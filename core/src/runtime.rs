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
        let stored = LogRecord {
            ts: rec.ts.as_str().to_string(),
            module: rec.module.as_str().to_string(),
            tag: rec.tag.as_str().to_string(),
            level: rec.level,
            detail: rec.detail.as_str().to_string(),
        };
        self.lines.push_back(stored);
        while self.lines.len() > self.max_lines {
            self.lines.pop_front();
        }
        let _ = self.tx.send(rec);
    }

    fn tail(&self, n: usize) -> Vec<LogRecord> {
        let n = n.max(1).min(self.lines.len());
        self.lines
            .iter()
            .skip(self.lines.len() - n)
            .map(|rec| LogRecord {
                ts: rec.ts.as_str().to_string(),
                module: rec.module.as_str().to_string(),
                tag: rec.tag.as_str().to_string(),
                level: rec.level,
                detail: rec.detail.as_str().to_string(),
            })
            .collect()
    }

    fn subscribe(&self) -> broadcast::Receiver<LogRecord> {
        self.tx.subscribe()
    }
}

#[derive(Debug)]
struct Inner {
    module: String,
    cron_expr: String,
    run_task: Option<tokio::task::JoinHandle<()>>,
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
                run_task: None,
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
                cron_expr: i.cron_expr.as_str().to_string(),
                module: i.module.as_str().to_string(),
                last_run_at: i
                    .last_run_at
                    .as_deref()
                    .unwrap_or_default()
                    .to_string(),
                next_run_at: i
                    .next_run_at
                    .as_deref()
                    .unwrap_or_default()
                    .to_string(),
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
        let input_module = module.trim().to_string();
        let module = if input_module.is_empty() {
            self.inner.lock().await.module.to_string()
        } else {
            input_module
        };

        let started = self
            .running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok();
        if !started {
            return Ok(false);
        }

        // 将本次成功启动的 module 写回默认值，便于状态展示与后续“继续/重复执行”。
        // Persist module as default only when we actually started.
        {
            let mut inner = self.inner.lock().await;
            inner.module = module.to_string();
        }

        self.append_sys(LogLevel::Info, "TASK:任务启动", format!("module={}", module))
            .await;

        let this = Arc::clone(&self);
        let handle = tokio::spawn(async move {
            // 任务执行期间避免长时间持有配置锁：配置只读，拷贝一份用于本次任务。
            // Avoid holding config lock across awaits: clone config for this run.
            let cfg = { this.config.lock().await.clone() };

            this.append_sys(LogLevel::Info, "TASK:任务执行", format!("module={}", module))
                .await;

            let plan = match module.as_str() {
                "ispdomain" => format!(
                    "custom_isp={} stream_domain={}",
                    cfg.custom_isp.len(),
                    cfg.stream_domain.len()
                ),
                "ipgroup" => format!(
                    "ip_group={} stream_ipport={}",
                    cfg.ip_group.len(),
                    cfg.stream_ipport.len()
                ),
                "ipv6group" => format!("ipv6_group={}", cfg.ipv6_group.len()),
                "ii" => format!(
                    "custom_isp={} stream_domain={} ip_group={} stream_ipport={}",
                    cfg.custom_isp.len(),
                    cfg.stream_domain.len(),
                    cfg.ip_group.len(),
                    cfg.stream_ipport.len()
                ),
                "ip" => format!("ip_group={} ipv6_group={}", cfg.ip_group.len(), cfg.ipv6_group.len()),
                "iip" => format!(
                    "custom_isp={} stream_domain={} ip_group={} ipv6_group={} stream_ipport={}",
                    cfg.custom_isp.len(),
                    cfg.stream_domain.len(),
                    cfg.ip_group.len(),
                    cfg.ipv6_group.len(),
                    cfg.stream_ipport.len()
                ),
                other => format!("module={}", other),
            };
            this.append_sys(LogLevel::Info, "TASK:任务计划", plan).await;

            let sink: LogSink = {
                let svc = Arc::clone(&this);
                Arc::new(move |rec| {
                    let svc = Arc::clone(&svc);
                    tokio::spawn(async move {
                        svc.logs.lock().await.append(rec);
                    });
                })
            };

            let res = crate::update::run_update_by_module(&cfg, &this.cli_login, &module, sink).await;
            match res {
                Ok(()) => {
                    this.append_sys(LogLevel::Success, "DONE:任务完成", format!("module={}", module))
                        .await;
                }
                Err(e) => {
                    this.append_sys(
                        LogLevel::Error,
                        "TASK:任务失败",
                        format!("module={} error={}", module, e),
                    )
                    .await;
                }
            }
            {
                let mut inner = this.inner.lock().await;
                inner.last_run_at = Some(Local::now().to_rfc3339());
                inner.run_task = None;
            }
            this.running.store(false, Ordering::SeqCst);
        });

        {
            let mut inner = self.inner.lock().await;
            inner.run_task = Some(handle);
        }

        Ok(true)
    }

    pub async fn start_cron(self: Arc<Self>, expr: String, module: String) -> Result<(), String> {
        let expr = expr.trim().to_string();
        let module = module.trim().to_string();
        if expr.is_empty() {
            return Err("Cron expression is empty in config file".to_string());
        }
        let schedule_expr = normalize_cron_expr_for_parser(&expr)?;
        let schedule = Schedule::from_str(&schedule_expr).map_err(|e| e.to_string())?;

        let module = if module.is_empty() {
            self.inner.lock().await.module.to_string()
        } else {
            module
        };

        {
            // 启动成功前不修改状态，避免无效表达式污染 status()。
            // Do not mutate state before we know the schedule is valid.
            let mut inner = self.inner.lock().await;
            if inner.cron_task.is_some() {
                return Err("cron is already running".to_string());
            }
            inner.cron_expr = expr.to_string();
            inner.module = module.to_string();
        }

        let this = Arc::clone(&self);
        let module_for_cron = module.to_string();
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

                let _ = Arc::clone(&this)
                    .start_run_once(module_for_cron.to_string())
                    .await;
            }
        });

        {
            let mut inner = self.inner.lock().await;
            inner.cron_task = Some(handle);
        }

        self.append_sys(
            LogLevel::Info,
            "CRON:定时任务启动",
            format!("cron={} module={}", expr, module),
        )
        .await;
        Ok(())
    }

    pub async fn stop_cron(self: Arc<Self>) -> Result<(), String> {
        let (handle, expr, module) = {
            let mut inner = self.inner.lock().await;
            inner.next_run_at = None;
            (inner.cron_task.take(), inner.cron_expr.to_string(), inner.module.to_string())
        };
        if let Some(h) = handle {
            h.abort();

            self.append_sys(
                LogLevel::Warn,
                "CRON:定时任务停止",
                format!("cron={} module={}", expr, module),
            )
            .await;
        }
        Ok(())
    }

    pub async fn stop_all(self: Arc<Self>) -> Result<(), String> {
        let (run_handle, cron_handle) = {
            let mut inner = self.inner.lock().await;
            inner.next_run_at = None;
            (inner.run_task.take(), inner.cron_task.take())
        };

        if let Some(h) = run_handle {
            h.abort();
        }
        if let Some(h) = cron_handle {
            h.abort();
        }

        self.running.store(false, Ordering::SeqCst);
        self.append_sys(LogLevel::Warn, "TASK:任务停止", "runtime stop requested".to_string())
            .await;
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

    // 兼容 Go 版常见 5 段 crontab（分/时/日/月/周），并尽量适配 cron crate 的解析规则。
    // Compatibility: accept common 5-field crontab and normalize for the `cron` parser.
    let parts: Vec<&str> = raw.split_whitespace().collect();
    let mut candidates = Vec::new();
    candidates.push(raw.to_string());
    if parts.len() == 5 {
        candidates.push(format!("0 {}", raw));
        candidates.push(format!("0 {} *", raw));
    }
    if parts.len() == 6 {
        candidates.push(format!("{} *", raw));
    }

    for c in candidates {
        if Schedule::from_str(&c).is_ok() {
            return Ok(c);
        }
    }
    Err("Invalid cron expression".to_string())
}
