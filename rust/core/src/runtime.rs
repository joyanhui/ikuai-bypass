use std::collections::VecDeque;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::DateTime;
use chrono::Local;
use cron::Schedule;
use tokio::sync::{broadcast, Mutex};

use crate::config::Config;
use crate::logger::{LogLevel, LogRecord, LogSink};

#[derive(Debug, Clone, serde::Serialize)]
pub struct RuntimeStatus {
    pub running: bool,
    pub cron_running: bool,
    pub cron_expr: String,
    pub module: String,
    pub last_run_at: String,
    pub next_run_at: String,
}

#[derive(Debug)]
struct LogBroker {
    max_lines: usize,
    lines: VecDeque<LogRecord>,
    tx: broadcast::Sender<LogRecord>,
}

impl LogBroker {
    fn new(max_lines: usize) -> Self {
        let (tx, _) = broadcast::channel(512);
        Self {
            max_lines: max_lines.max(1),
            lines: VecDeque::with_capacity(max_lines.max(1)),
            tx,
        }
    }

    fn append(&mut self, rec: LogRecord) {
        self.lines.push_back(rec.clone());
        while self.lines.len() > self.max_lines {
            self.lines.pop_front();
        }
        let _ = self.tx.send(rec);
    }

    fn tail(&self, n: usize) -> Vec<LogRecord> {
        let n = n.max(1).min(self.lines.len());
        self.lines.iter().skip(self.lines.len() - n).cloned().collect()
    }

    fn subscribe(&self) -> broadcast::Receiver<LogRecord> {
        self.tx.subscribe()
    }
}

#[derive(Debug)]
struct Inner {
    module: String,
    cron_expr: String,
    cron_task: Option<tokio::task::JoinHandle<()>>,
    next_run_at: Option<String>,
    last_run_at: Option<String>,
}

#[derive(Debug)]
pub struct RuntimeService {
    inner: Mutex<Inner>,
    running: AtomicBool,
    logs: Mutex<LogBroker>,
    config: Arc<tokio::sync::Mutex<Config>>,
    cli_login: String,
}

impl RuntimeService {
    pub fn new(
        config: Arc<tokio::sync::Mutex<Config>>,
        cli_login: String,
        default_cron: String,
        default_module: String,
    ) -> Self {
        Self {
            inner: Mutex::new(Inner {
                module: default_module,
                cron_expr: default_cron,
                cron_task: None,
                next_run_at: None,
                last_run_at: None,
            }),
            running: AtomicBool::new(false),
            logs: Mutex::new(LogBroker::new(5000)),
            config,
            cli_login,
        }
    }

    pub async fn set_defaults(&self, module: Option<String>, cron_expr: Option<String>) {
        let module = module.filter(|s| !s.trim().is_empty());
        let cron_expr = cron_expr.filter(|s| !s.trim().is_empty());
        let mut inner = self.inner.lock().await;
        if let Some(m) = module {
            inner.module = m;
        }
        if let Some(c) = cron_expr {
            inner.cron_expr = c;
        }
    }

    pub async fn tail_logs(&self, n: usize) -> Vec<LogRecord> {
        self.logs.lock().await.tail(n)
    }

    pub async fn subscribe_logs(&self) -> broadcast::Receiver<LogRecord> {
        self.logs.lock().await.subscribe()
    }

    pub fn status(&self) -> RuntimeStatus {
        let inner = self.inner.try_lock();
        if let Ok(i) = inner {
            return RuntimeStatus {
                running: self.running.load(Ordering::SeqCst),
                cron_running: i.cron_task.is_some(),
                cron_expr: i.cron_expr.clone(),
                module: i.module.clone(),
                last_run_at: i.last_run_at.clone().unwrap_or_default(),
                next_run_at: i.next_run_at.clone().unwrap_or_default(),
            };
        }
        RuntimeStatus {
            running: self.running.load(Ordering::SeqCst),
            cron_running: false,
            cron_expr: String::new(),
            module: String::new(),
            last_run_at: String::new(),
            next_run_at: String::new(),
        }
    }

    pub async fn start_run_once(self: Arc<Self>, module: String) -> Result<bool, String> {
        let module = if module.trim().is_empty() {
            self.inner.lock().await.module.clone()
        } else {
            module
        };

        let started = self
            .running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok();
        if !started {
            return Ok(false);
        }

        self.append_sys(LogLevel::Info, "TASK:任务启动", format!("module={}", module))
            .await;

        let this = Arc::clone(&self);
        tokio::spawn(async move {
            let cfg = this.config.lock().await.clone();
            this.append_sys(LogLevel::Info, "TASK:任务执行", format!("module={}", module))
                .await;

            let sink: LogSink = {
                let svc = Arc::clone(&this);
                Arc::new(move |rec| {
                    let svc = Arc::clone(&svc);
                    tokio::spawn(async move {
                        svc.logs.lock().await.append(rec);
                    });
                })
            };

            let _ = crate::update::run_update_by_module(&cfg, &this.cli_login, &module, sink).await;
            {
                let mut inner = this.inner.lock().await;
                inner.last_run_at = Some(Local::now().to_rfc3339());
            }
            this.running.store(false, Ordering::SeqCst);
        });

        Ok(true)
    }

    pub async fn start_cron(self: Arc<Self>, expr: String, module: String) -> Result<(), String> {
        if expr.trim().is_empty() {
            return Err("Cron expression is empty in config file".to_string());
        }
        {
            let inner = self.inner.lock().await;
            if !inner.cron_expr.trim().is_empty() && expr != inner.cron_expr {
                return Err("Cron expression must match config file".to_string());
            }
            if inner.cron_task.is_some() {
                return Err("cron is already running".to_string());
            }
        }

        let schedule_expr = normalize_cron_expr_for_parser(&expr)?;
        let schedule = Schedule::from_str(&schedule_expr).map_err(|e| e.to_string())?;
        let module = if module.trim().is_empty() {
            self.inner.lock().await.module.clone()
        } else {
            module
        };

        let this = Arc::clone(&self);
        let handle = tokio::spawn(async move {
            let mut upcoming = schedule.upcoming(Local);
            loop {
                let next: DateTime<Local> = match upcoming.next() {
                    Some(t) => t,
                    None => return,
                };

                {
                    let mut inner = this.inner.lock().await;
                    inner.next_run_at = Some(next.to_rfc3339());
                }

                loop {
                    let now = Local::now();
                    if next <= now {
                        break;
                    }
                    let wait = (next - now)
                        .to_std()
                        .unwrap_or(Duration::from_millis(200));
                    tokio::time::sleep(wait.min(Duration::from_secs(1))).await;
                }

                let _ = Arc::clone(&this).start_run_once(module.clone()).await;
            }
        });

        {
            let mut inner = self.inner.lock().await;
            inner.cron_task = Some(handle);
        }
        Ok(())
    }

    pub async fn stop_cron(self: Arc<Self>) -> Result<(), String> {
        let handle = {
            let mut inner = self.inner.lock().await;
            inner.next_run_at = None;
            inner.cron_task.take()
        };
        if let Some(h) = handle {
            h.abort();
        }
        Ok(())
    }

    async fn append_sys(&self, level: LogLevel, tag: &str, detail: String) {
        let mut b = self.logs.lock().await;
        b.append(LogRecord {
            ts: Local::now().format("%Y/%m/%d %H:%M:%S").to_string(),
            module: "SYS:系统组件".to_string(),
            tag: tag.to_string(),
            level,
            detail,
        });
    }
}

fn normalize_cron_expr_for_parser(expr: &str) -> Result<String, String> {
    let raw = expr.trim();
    if raw.is_empty() {
        return Err("cron expression is empty".to_string());
    }
    let parts: Vec<&str> = raw.split_whitespace().collect();
    match parts.len() {
        5 => Ok(format!("0 {}", raw)),
        6 => Ok(raw.to_string()),
        _ => Err("Invalid cron expression".to_string()),
    }
}
