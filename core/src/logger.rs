use std::sync::Arc;

use chrono::Local;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Info,
    Success,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogRecord {
    pub ts: String,
    pub module: String,
    pub tag: String,
    pub level: LogLevel,
    pub detail: String,
}

pub type LogSink = Arc<dyn Fn(LogRecord) + Send + Sync>;

#[derive(Clone)]
pub struct Logger {
    module: String,
    sink: LogSink,
}

impl Logger {
    pub fn new(module: impl Into<String>, sink: LogSink) -> Self {
        Self {
            module: module.into(),
            sink,
        }
    }

    pub fn info(&self, tag: impl Into<String>, detail: impl Into<String>) {
        self.emit(LogLevel::Info, tag.into(), detail.into());
    }

    pub fn success(&self, tag: impl Into<String>, detail: impl Into<String>) {
        self.emit(LogLevel::Success, tag.into(), detail.into());
    }

    pub fn warn(&self, tag: impl Into<String>, detail: impl Into<String>) {
        self.emit(LogLevel::Warn, tag.into(), detail.into());
    }

    pub fn error(&self, tag: impl Into<String>, detail: impl Into<String>) {
        self.emit(LogLevel::Error, tag.into(), detail.into());
    }

    fn emit(&self, level: LogLevel, tag: String, detail: String) {
        let ts = Local::now().format("%Y/%m/%d %H:%M:%S").to_string();
        (self.sink)(LogRecord {
            ts,
            module: self.module.as_str().to_string(),
            tag,
            level,
            detail,
        });
    }
}

pub struct Renderer {
    use_color: bool,
}

impl Renderer {
    pub fn new(use_color: bool) -> Self {
        Self { use_color }
    }

    pub fn render(&self, rec: &LogRecord) -> String {
        if !self.use_color {
            return format!("{} [{}] [{}] {}", rec.ts, rec.module, rec.tag, rec.detail);
        }

        let time = style(90, false, &rec.ts);
        let module = style(36, true, &format!("[{}]", rec.module));
        let tag_color = match rec.level {
            LogLevel::Info => (34, false),
            LogLevel::Success => (32, false),
            LogLevel::Warn => (33, false),
            LogLevel::Error => (31, true),
        };
        let tag = style(tag_color.0, tag_color.1, &format!("[{}]", rec.tag));
        let detail = highlight(&rec.detail);
        format!("{} {} {} {}", time, module, tag, detail)
    }
}

fn style(color: u8, bold: bool, s: &str) -> String {
    if bold {
        format!("\x1b[1;{}m{}\x1b[0m", color, s)
    } else {
        format!("\x1b[{}m{}\x1b[0m", color, s)
    }
}

static RE_QUOTED: Lazy<Result<Regex, regex::Error>> = Lazy::new(|| Regex::new(r"'([^']+)'"));
static RE_KV: Lazy<Result<Regex, regex::Error>> =
    Lazy::new(|| Regex::new(r"(?i)(Prefix|Tag|IDs?|found|error|status|interface):\s*([^\s,)]+)"));
static RE_SAFE_NUM: Lazy<Result<Regex, regex::Error>> =
    Lazy::new(|| Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]|\b\d+\b"));

fn highlight(s: &str) -> String {
    let mut out = s.to_string();

    if let Ok(re) = RE_QUOTED.as_ref() {
        out = re
            .replace_all(&out, |caps: &regex::Captures| {
                let matched = caps.get(0).map(|m| m.as_str()).unwrap_or("");
                style(93, true, matched)
            })
            .to_string();
    }

    if let Ok(re) = RE_KV.as_ref() {
        out = re
            .replace_all(&out, |caps: &regex::Captures| {
                let key = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let val = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                if val.contains("\x1b[") {
                    format!("{}: {}", key, val)
                } else {
                    format!("{}: {}", key, style(93, true, val))
                }
            })
            .to_string();
    }

    if let Ok(re) = RE_SAFE_NUM.as_ref() {
        out = re
            .replace_all(&out, |caps: &regex::Captures| {
                let m = caps.get(0).map(|m| m.as_str()).unwrap_or("");
                if m.starts_with("\x1b") {
                    m.to_string()
                } else {
                    style(95, false, m)
                }
            })
            .to_string();
    }

    out
}
