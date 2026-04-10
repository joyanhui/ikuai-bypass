use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;

mod duration_compat {
    use std::fmt;
    use std::time::Duration;

    use serde::de;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(d: &Duration, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(&humantime::format_duration(*d).to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct V;

        impl<'de> de::Visitor<'de> for V {
            type Value = Duration;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "duration as string or nanoseconds")
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Duration::from_nanos(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if v < 0 {
                    return Err(E::custom("duration nanoseconds must be >= 0"));
                }
                Ok(Duration::from_nanos(v as u64))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                humantime::parse_duration(v).map_err(E::custom)
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(&v)
            }
        }

        deserializer.deserialize_any(V)
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("read config failed: {0}")]
    ReadFailed(#[from] std::io::Error),
    #[error("parse yaml failed: {0}")]
    ParseFailed(#[from] serde_yaml::Error),
    #[error("security violation: file extension must be .yml or .yaml")]
    InvalidExtension,
    #[error("security violation: cannot write to a symbolic link")]
    SymlinkDenied,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomIspItem {
    #[serde(rename = "tag")]
    pub tag: String,
    #[serde(rename = "url")]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StreamDomainItem {
    #[serde(rename = "interface")]
    pub interface: String,
    #[serde(rename = "src-addr", default)]
    pub src_addr: String,
    #[serde(rename = "src-addr-opt-ipgroup", default)]
    pub src_addr_opt_ipgroup: String,
    #[serde(rename = "url")]
    pub url: String,
    #[serde(rename = "tag", default)]
    pub tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IpGroupItem {
    #[serde(rename = "tag")]
    pub tag: String,
    #[serde(rename = "url")]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Ipv6GroupItem {
    #[serde(rename = "tag")]
    pub tag: String,
    #[serde(rename = "url")]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StreamIpPortItem {
    #[serde(rename = "opt-tagname", default)]
    pub opt_tagname: String,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(rename = "interface", default)]
    pub interface: String,
    #[serde(rename = "nexthop", default)]
    pub nexthop: String,
    #[serde(rename = "src-addr", default)]
    pub src_addr: String,
    #[serde(rename = "src-addr-opt-ipgroup", default)]
    pub src_addr_opt_ipgroup: String,
    #[serde(rename = "src-addr-inv", default)]
    pub src_addr_inv: i64,
    #[serde(rename = "ip-group", default)]
    pub ip_group: String,
    #[serde(rename = "dst-addr-inv", default)]
    pub dst_addr_inv: i64,
    #[serde(rename = "mode")]
    pub mode: i64,
    #[serde(rename = "ifaceband")]
    pub ifaceband: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct WebUiConfig {
    #[serde(rename = "port", default)]
    pub port: String,
    #[serde(rename = "user", default)]
    pub user: String,
    #[serde(rename = "pass", default)]
    pub pass: String,
    #[serde(rename = "enable", default)]
    pub enable: bool,
    #[serde(rename = "cdn-prefix", default)]
    pub cdn_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct MaxNumberOfOneRecordsConfig {
    #[serde(rename = "Isp", default)]
    pub isp: i64,
    #[serde(rename = "Ipv4", default)]
    pub ipv4: i64,
    #[serde(rename = "Ipv6", default)]
    pub ipv6: i64,
    #[serde(rename = "Domain", default)]
    pub domain: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ProxyMode {
    #[serde(rename = "custom")]
    Custom,
    #[serde(rename = "system", alias = "disabled")]
    System,
    #[serde(
        rename = "smart",
        alias = "onlyGithubApi",
        alias = "only-github-api",
        alias = "only_github_api"
    )]
    #[default]
    Smart,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProxyConfig {
    #[serde(default)]
    pub mode: ProxyMode,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub user: String,
    #[serde(default)]
    pub pass: String,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            mode: ProxyMode::Smart,
            url: String::new(),
            user: String::new(),
            pass: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    #[serde(rename = "ikuai-url")]
    pub ikuai_url: String,
    #[serde(rename = "username")]
    pub username: String,
    #[serde(rename = "password")]
    pub password: String,
    #[serde(rename = "cron")]
    pub cron: String,

    #[serde(rename = "AddErrRetryWait", with = "duration_compat", default)]
    pub add_err_retry_wait: Duration,
    #[serde(rename = "AddWait", with = "duration_compat", default)]
    pub add_wait: Duration,

    #[serde(rename = "github-proxy", default)]
    pub github_proxy: String,

    /// Global HTTP proxy configuration.
    /// 全局 HTTP 代理配置。
    #[serde(rename = "proxy", default)]
    pub proxy: ProxyConfig,

    #[serde(rename = "custom-isp", default)]
    pub custom_isp: Vec<CustomIspItem>,
    #[serde(rename = "stream-domain", default)]
    pub stream_domain: Vec<StreamDomainItem>,
    #[serde(rename = "ip-group", default)]
    pub ip_group: Vec<IpGroupItem>,
    #[serde(rename = "ipv6-group", default)]
    pub ipv6_group: Vec<Ipv6GroupItem>,
    #[serde(rename = "stream-ipport", default)]
    pub stream_ipport: Vec<StreamIpPortItem>,

    #[serde(rename = "webui", default)]
    pub webui: WebUiConfig,
    #[serde(rename = "MaxNumberOfOneRecords", default)]
    pub max_number_of_one_records: MaxNumberOfOneRecordsConfig,
}

impl Config {
    pub fn load_from_yaml_str(raw: &str) -> Result<Self, ConfigError> {
        let mut cfg: Config = serde_yaml::from_str(raw)?;
        cfg.apply_defaults();
        Ok(cfg)
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let raw = fs::read_to_string(path)?;
        Self::load_from_yaml_str(&raw)
    }

    pub fn apply_defaults(&mut self) {
        fn normalize_binary_flag(value: i64) -> i64 {
            if value == 1 { 1 } else { 0 }
        }

        // WebUI defaults.
        // WebUI 默认值。
        if self.webui.port.trim().is_empty() {
            // Keep consistent with docs and frontend defaults.
            // 与文档/前端默认值保持一致。
            self.webui.port = "19001".to_string();
        } else {
            self.webui.port = self.webui.port.trim().to_string();
        }

        if self.webui.cdn_prefix.is_empty() {
            self.webui.cdn_prefix = "https://cdn.jsdelivr.net/npm".to_string();
        }

        if self.max_number_of_one_records.isp == 0 {
            self.max_number_of_one_records.isp = 5000;
        }
        if self.max_number_of_one_records.ipv4 == 0 {
            self.max_number_of_one_records.ipv4 = 1000;
        }
        if self.max_number_of_one_records.ipv6 == 0 {
            self.max_number_of_one_records.ipv6 = 1000;
        }
        if self.max_number_of_one_records.domain == 0 {
            self.max_number_of_one_records.domain = 5000;
        }

        for item in &mut self.stream_domain {
            if item.tag.is_empty() {
                item.tag = item.interface.to_string();
            }
        }

        for item in &mut self.stream_ipport {
            item.src_addr_inv = normalize_binary_flag(item.src_addr_inv);
            item.dst_addr_inv = normalize_binary_flag(item.dst_addr_inv);
        }

        // Proxy defaults & normalization.
        // 代理默认值与标准化处理。
        self.proxy.url = self.proxy.url.trim().to_string();
        self.proxy.user = self.proxy.user.trim().to_string();
        self.proxy.pass = self.proxy.pass.trim().to_string();
        if matches!(self.proxy.mode, ProxyMode::Custom) && self.proxy.url.is_empty() {
            self.proxy.url = "http://127.0.0.1:7890".to_string();
        }
        /* 配置文件应该做为唯一的配置来源
        // Environment variable overrides (for Docker / ipkg deployment).
        // Only override when the env var is set and non-empty, so that
        // config-file-only users are not affected.
        // 环境变量覆盖（用于 Docker / ipkg 部署场景）。
        // 仅当环境变量已设置且非空时才覆盖，确保纯配置文件用户不受影响。
        if let Ok(v) = std::env::var("IKUAI_URL") {
            if !v.trim().is_empty() {
                self.ikuai_url = v.trim().to_string();
            }
        }
        if let Ok(v) = std::env::var("IKUAI_USERNAME") {
            if !v.trim().is_empty() {
                self.username = v.trim().to_string();
            }
        }
        if let Ok(v) = std::env::var("IKUAI_PASSWORD") {
            if !v.trim().is_empty() {
                self.password = v.trim().to_string();
            }
        }
        if let Ok(v) = std::env::var("WEBUI_USER") {
            if !v.trim().is_empty() {
                self.webui.user = v.trim().to_string();
            }
        }
        if let Ok(v) = std::env::var("WEBUI_PASS") {
            if !v.trim().is_empty() {
                self.webui.pass = v.trim().to_string();
            }
        }
        */
    }

    pub fn save_to_path(&self, path: impl AsRef<Path>) -> Result<(), ConfigError> {
        validate_save_path(path.as_ref())?;
        let data = serde_yaml::to_string(self)?;

        #[cfg(unix)]
        {
            use std::io::Write;
            use std::os::unix::fs::OpenOptionsExt;
            let mut options = fs::OpenOptions::new();
            options.create(true).truncate(true).write(true).mode(0o600);
            let mut f = options.open(path)?;
            f.write_all(data.as_bytes())?;
            Ok(())
        }

        #[cfg(not(unix))]
        {
            fs::write(path, data)?;
            Ok(())
        }
    }

    pub fn validate_and_save_raw_yaml(
        raw: &str,
        path: impl AsRef<Path>,
    ) -> Result<Self, ConfigError> {
        let cfg = Self::load_from_yaml_str(raw)?;
        validate_save_path(path.as_ref())?;
        let data = raw.to_string();

        #[cfg(unix)]
        {
            use std::io::Write;
            use std::os::unix::fs::OpenOptionsExt;
            let mut options = fs::OpenOptions::new();
            options.create(true).truncate(true).write(true).mode(0o600);
            let mut f = options.open(path)?;
            f.write_all(data.as_bytes())?;
            Ok(cfg)
        }

        #[cfg(not(unix))]
        {
            fs::write(path, data)?;
            Ok(cfg)
        }
    }
}

pub fn validate_save_path(path: &Path) -> Result<(), ConfigError> {
    let ext = path.extension().and_then(OsStr::to_str).unwrap_or_default();
    let ext = ext.to_ascii_lowercase();
    if ext != "yml" && ext != "yaml" {
        return Err(ConfigError::InvalidExtension);
    }

    match fs::symlink_metadata(path) {
        Ok(meta) => {
            if meta.file_type().is_symlink() {
                return Err(ConfigError::SymlinkDenied);
            }
        }
        Err(e) => {
            if e.kind() != std::io::ErrorKind::NotFound {
                return Err(ConfigError::ReadFailed(e));
            }
        }
    }

    Ok(())
}
