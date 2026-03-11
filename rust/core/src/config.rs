use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;

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
    #[serde(rename = "ip-group", default)]
    pub ip_group: String,
    #[serde(rename = "mode")]
    pub mode: i64,
    #[serde(rename = "ifaceband")]
    pub ifaceband: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WebUiConfig {
    #[serde(rename = "port", default)]
    pub port: String,
    #[serde(rename = "user", default)]
    pub user: String,
    #[serde(rename = "pass", default)]
    pub pass: String,
    #[serde(rename = "enable", default)]
    pub enable: bool,
    #[serde(rename = "enable-update", default)]
    pub enable_update: bool,
    #[serde(rename = "cdn-prefix", default)]
    pub cdn_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

    #[serde(rename = "AddErrRetryWait", with = "humantime_serde", default)]
    pub add_err_retry_wait: Duration,
    #[serde(rename = "AddWait", with = "humantime_serde", default)]
    pub add_wait: Duration,

    #[serde(rename = "github-proxy", default)]
    pub github_proxy: String,

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

impl Default for WebUiConfig {
    fn default() -> Self {
        Self {
            port: String::new(),
            user: String::new(),
            pass: String::new(),
            enable: false,
            enable_update: false,
            cdn_prefix: String::new(),
        }
    }
}

impl Default for MaxNumberOfOneRecordsConfig {
    fn default() -> Self {
        Self {
            isp: 0,
            ipv4: 0,
            ipv6: 0,
            domain: 0,
        }
    }
}

impl Config {
    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let raw = fs::read_to_string(path)?;
        let mut cfg: Config = serde_yaml::from_str(&raw)?;
        cfg.apply_defaults();
        Ok(cfg)
    }

    pub fn apply_defaults(&mut self) {
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
                item.tag = item.interface.clone();
            }
        }
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
            return Ok(());
        }

        #[cfg(not(unix))]
        {
            fs::write(path, data)?;
            Ok(())
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
