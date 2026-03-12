use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::time::Duration;

use std::collections::BTreeMap;

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

    #[serde(rename = "AddErrRetryWait", with = "duration_compat", default)]
    pub add_err_retry_wait: Duration,
    #[serde(rename = "AddWait", with = "duration_compat", default)]
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

    pub fn save_to_path_with_comments(
        &self,
        path: impl AsRef<Path>,
        with_comments: bool,
    ) -> Result<(), ConfigError> {
        validate_save_path(path.as_ref())?;
        let data = if with_comments {
            self.to_commented_yaml()
        } else {
            serde_yaml::to_string(self)?
        };

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

    pub fn validate_and_save_raw_yaml(
        raw: &str,
        path: impl AsRef<Path>,
        with_comments: bool,
    ) -> Result<Self, ConfigError> {
        let cfg = Self::load_from_yaml_str(raw)?;
        validate_save_path(path.as_ref())?;
        let data = if with_comments {
            raw.to_string()
        } else {
            serde_yaml::to_string(&cfg)?
        };

        #[cfg(unix)]
        {
            use std::io::Write;
            use std::os::unix::fs::OpenOptionsExt;
            let mut options = fs::OpenOptions::new();
            options.create(true).truncate(true).write(true).mode(0o600);
            let mut f = options.open(path)?;
            f.write_all(data.as_bytes())?;
            return Ok(cfg);
        }

        #[cfg(not(unix))]
        {
            fs::write(path, data)?;
            Ok(cfg)
        }
    }

    fn to_commented_yaml(&self) -> String {
        let mut out = String::new();
        out.push_str("#  iKuai Bypass 配置文件 大部分时候请使用默认设置即可\n");
        out.push_str("#  详情参考: https://github.com/joyanhui/ikuai-bypass\n");
        out.push_str("#\n");
        out.push_str("#  【重要】tag 字段长度限制说明 / Important: tag field length limitation\n");
        out.push_str("#  爱快固件 4.0.101 对规则名称(tagname)有 15 字符的长度限制\n");
        out.push_str("#  系统会自动添加 \"IKB\" 前缀，因此 tag 字段建议不超过 11 个字符\n");
        out.push_str("#  超过限制的 tag 会被自动截断并打印警告日志\n");
        out.push_str("#  iKuai firmware 4.0.101 has a 15-character limit on tagname length\n");
        out.push_str(
            "#  System auto-adds \"IKB\" prefix, so tag field should not exceed 11 characters\n",
        );
        out.push_str("#  Tags exceeding the limit will be auto-truncated with a warning log\n");

        let top = top_level_comments();
        let item = item_comments();
        let webui = webui_comments();
        let maxn = max_number_of_one_records_comments();

        fn push_kv(out: &mut String, key: &str, value: &str, comment: Option<&str>) {
            if let Some(c) = comment {
                out.push_str("\n# ");
                out.push_str(c);
                out.push('\n');
            } else {
                out.push('\n');
            }
            out.push_str(key);
            out.push_str(": ");
            out.push_str(value);
            out.push('\n');
        }

        fn yaml_quote(s: &str) -> String {
            if s.is_empty() {
                return "\"\"".to_string();
            }
            let need_quote = s
                .chars()
                .any(|ch| matches!(ch, ':' | '#' | '\n' | '\r' | '\t'))
                || s.starts_with(' ')
                || s.ends_with(' ');
            if !need_quote {
                return s.to_string();
            }
            let mut v = String::new();
            v.push('"');
            for ch in s.chars() {
                match ch {
                    '\\' => v.push_str("\\\\"),
                    '"' => v.push_str("\\\""),
                    _ => v.push(ch),
                }
            }
            v.push('"');
            v
        }

        let ikuai_url = yaml_quote(self.ikuai_url.trim());
        let username = yaml_quote(self.username.trim());
        let password = yaml_quote(self.password.trim());
        let cron = yaml_quote(self.cron.trim());
        let add_err_retry_wait =
            yaml_quote(&humantime::format_duration(self.add_err_retry_wait).to_string());
        let add_wait = yaml_quote(&humantime::format_duration(self.add_wait).to_string());
        let github_proxy = yaml_quote(self.github_proxy.trim());

        push_kv(
            &mut out,
            "ikuai-url",
            &ikuai_url,
            top.get("ikuai-url").map(|s| s.as_str()),
        );
        push_kv(
            &mut out,
            "username",
            &username,
            top.get("username").map(|s| s.as_str()),
        );
        push_kv(
            &mut out,
            "password",
            &password,
            top.get("password").map(|s| s.as_str()),
        );
        push_kv(&mut out, "cron", &cron, top.get("cron").map(|s| s.as_str()));
        push_kv(
            &mut out,
            "AddErrRetryWait",
            &add_err_retry_wait,
            top.get("AddErrRetryWait").map(|s| s.as_str()),
        );
        push_kv(
            &mut out,
            "AddWait",
            &add_wait,
            top.get("AddWait").map(|s| s.as_str()),
        );
        push_kv(
            &mut out,
            "github-proxy",
            &github_proxy,
            top.get("github-proxy").map(|s| s.as_str()),
        );

        out.push_str("\n# ");
        out.push_str(
            top.get("custom-isp")
                .map(|s| s.as_str())
                .unwrap_or("custom-isp"),
        );
        out.push('\n');
        out.push_str("custom-isp:\n");
        for it in &self.custom_isp {
            out.push_str("  - ");
            out.push_str("tag: ");
            out.push_str(&yaml_quote(&it.tag));
            out.push('\n');
            if let Some(c) = item.get("tag") {
                out.push_str("    # ");
                out.push_str(c);
                out.push('\n');
            }
            out.push_str("    url: ");
            out.push_str(&yaml_quote(&it.url));
            out.push('\n');
        }

        out.push_str("\n# ");
        out.push_str(
            top.get("stream-domain")
                .map(|s| s.as_str())
                .unwrap_or("stream-domain"),
        );
        out.push('\n');
        out.push_str("stream-domain:\n");
        for it in &self.stream_domain {
            out.push_str("  - interface: ");
            out.push_str(&yaml_quote(&it.interface));
            out.push('\n');
            for k in ["interface", "src-addr", "src-addr-opt-ipgroup", "tag"].iter() {
                if let Some(c) = item.get(*k) {
                    out.push_str("    # ");
                    out.push_str(c);
                    out.push('\n');
                }
                match *k {
                    "src-addr" => {
                        out.push_str("    src-addr: ");
                        out.push_str(&yaml_quote(&it.src_addr));
                        out.push('\n');
                    }
                    "src-addr-opt-ipgroup" => {
                        out.push_str("    src-addr-opt-ipgroup: ");
                        out.push_str(&yaml_quote(&it.src_addr_opt_ipgroup));
                        out.push('\n');
                    }
                    "tag" => {
                        out.push_str("    tag: ");
                        out.push_str(&yaml_quote(&it.tag));
                        out.push('\n');
                    }
                    _ => {}
                }
            }
            out.push_str("    url: ");
            out.push_str(&yaml_quote(&it.url));
            out.push('\n');
        }

        out.push_str("\n# ");
        out.push_str(
            top.get("ip-group")
                .map(|s| s.as_str())
                .unwrap_or("ip-group"),
        );
        out.push('\n');
        out.push_str("ip-group:\n");
        for it in &self.ip_group {
            out.push_str("  - tag: ");
            out.push_str(&yaml_quote(&it.tag));
            out.push('\n');
            out.push_str("    url: ");
            out.push_str(&yaml_quote(&it.url));
            out.push('\n');
        }

        out.push_str("\n# ");
        out.push_str(
            top.get("ipv6-group")
                .map(|s| s.as_str())
                .unwrap_or("ipv6-group"),
        );
        out.push('\n');
        out.push_str("ipv6-group:\n");
        for it in &self.ipv6_group {
            out.push_str("  - tag: ");
            out.push_str(&yaml_quote(&it.tag));
            out.push('\n');
            out.push_str("    url: ");
            out.push_str(&yaml_quote(&it.url));
            out.push('\n');
        }

        out.push_str("\n# ");
        out.push_str(
            top.get("stream-ipport")
                .map(|s| s.as_str())
                .unwrap_or("stream-ipport"),
        );
        out.push('\n');
        out.push_str("stream-ipport:\n");
        for it in &self.stream_ipport {
            out.push_str("  - type: ");
            out.push_str(&yaml_quote(&it.r#type));
            out.push('\n');
            if let Some(c) = item.get("type") {
                out.push_str("    # ");
                out.push_str(c);
                out.push('\n');
            }
            out.push_str("    opt-tagname: ");
            out.push_str(&yaml_quote(&it.opt_tagname));
            out.push('\n');
            if let Some(c) = item.get("opt-tagname") {
                out.push_str("    # ");
                out.push_str(c);
                out.push('\n');
            }
            out.push_str("    interface: ");
            out.push_str(&yaml_quote(&it.interface));
            out.push('\n');
            out.push_str("    nexthop: ");
            out.push_str(&yaml_quote(&it.nexthop));
            out.push('\n');
            out.push_str("    src-addr: ");
            out.push_str(&yaml_quote(&it.src_addr));
            out.push('\n');
            out.push_str("    src-addr-opt-ipgroup: ");
            out.push_str(&yaml_quote(&it.src_addr_opt_ipgroup));
            out.push('\n');
            out.push_str("    ip-group: ");
            out.push_str(&yaml_quote(&it.ip_group));
            out.push('\n');
            out.push_str("    mode: ");
            out.push_str(&it.mode.to_string());
            out.push('\n');
            out.push_str("    ifaceband: ");
            out.push_str(&it.ifaceband.to_string());
            out.push('\n');
        }

        out.push_str("\n# ");
        out.push_str(top.get("webui").map(|s| s.as_str()).unwrap_or("webui"));
        out.push('\n');
        out.push_str("webui:\n");
        out.push_str("  port: ");
        out.push_str(&yaml_quote(&self.webui.port));
        out.push('\n');
        if let Some(c) = webui.get("port") {
            out.push_str("  # ");
            out.push_str(c);
            out.push('\n');
        }
        out.push_str("  user: ");
        out.push_str(&yaml_quote(&self.webui.user));
        out.push('\n');
        out.push_str("  pass: ");
        out.push_str(&yaml_quote(&self.webui.pass));
        out.push('\n');
        out.push_str("  enable: ");
        out.push_str(if self.webui.enable { "true" } else { "false" });
        out.push('\n');
        out.push_str("  cdn-prefix: ");
        out.push_str(&yaml_quote(&self.webui.cdn_prefix));
        out.push('\n');

        out.push_str("\n# ");
        out.push_str(
            top.get("MaxNumberOfOneRecords")
                .map(|s| s.as_str())
                .unwrap_or("MaxNumberOfOneRecords"),
        );
        out.push('\n');
        out.push_str("MaxNumberOfOneRecords:\n");
        out.push_str("  Isp: ");
        out.push_str(&self.max_number_of_one_records.isp.to_string());
        out.push('\n');
        if let Some(c) = maxn.get("Isp") {
            out.push_str("  # ");
            out.push_str(c);
            out.push('\n');
        }
        out.push_str("  Ipv4: ");
        out.push_str(&self.max_number_of_one_records.ipv4.to_string());
        out.push('\n');
        out.push_str("  Ipv6: ");
        out.push_str(&self.max_number_of_one_records.ipv6.to_string());
        out.push('\n');
        out.push_str("  Domain: ");
        out.push_str(&self.max_number_of_one_records.domain.to_string());
        out.push('\n');

        out
    }
}

pub fn top_level_comments() -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "ikuai-url".to_string(),
            "爱快控制台地址，结尾不要加 \"/\"".to_string(),
        ),
        ("username".to_string(), "爱快登陆用户名".to_string()),
        ("password".to_string(), "爱快登陆密码".to_string()),
        (
            "cron".to_string(),
            "更新周期cron表达式，例如 0 7 * * *".to_string(),
        ),
        (
            "AddErrRetryWait".to_string(),
            "自动重试时间间隔 (10s, 120s)".to_string(),
        ),
        (
            "AddWait".to_string(),
            "规则添加后的反应等待时间".to_string(),
        ),
        (
            "github-proxy".to_string(),
            "Github代理加速地址，例如 https://gh-proxy.com/ (留空不使用)".to_string(),
        ),
        ("webui".to_string(), "WebUI 管理服务设置".to_string()),
        (
            "custom-isp".to_string(),
            "自定义运营商分流 (IP分流)".to_string(),
        ),
        (
            "stream-domain".to_string(),
            "域名分流 (优先级高于IP分流)".to_string(),
        ),
        (
            "ip-group".to_string(),
            "IP分组 (与端口分流配合使用)".to_string(),
        ),
        (
            "ipv6-group".to_string(),
            "IPv6分组 (与端口分流配合使用)".to_string(),
        ),
        (
            "stream-ipport".to_string(),
            "端口分流 (下一跳网关/外网线路)".to_string(),
        ),
        (
            "MaxNumberOfOneRecords".to_string(),
            "分组和分流规则单条记录最大写入数据量设置".to_string(),
        ),
    ])
}

pub fn item_comments() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("type".to_string(), "分流方式：0-外网线路，1-下一跳网关".to_string()),
        ("mode".to_string(), "负载模式：0-新建连接数, 1-源IP, 2-源IP+源端口, 3-源IP+目的IP, 4-源IP+目的IP+目的端口, 5-主备模式".to_string()),
        ("ifaceband".to_string(), "线路绑定：0-不勾选，1-勾选".to_string()),
        ("interface".to_string(), "分流线路 (如 wan1)".to_string()),
        ("nexthop".to_string(), "下一跳网关地址".to_string()),
        ("tag".to_string(), "规则标识名称 (支持中文，系统自动添加 IKB 前缀)".to_string()),
        ("src-addr".to_string(), "分流源地址 (IP或范围)".to_string()),
        ("src-addr-opt-ipgroup".to_string(), "分流源地址标签(匹配爱快IP分组)，设置后 src-addr 会被忽略；多个名字用逗号分隔".to_string()),
        ("ip-group".to_string(), "关联的IP分组名称，多个名字可以逗号".to_string()),
        ("opt-tagname".to_string(), "该条规则的 TagName (可选)".to_string()),
    ])
}

pub fn webui_comments() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("port".to_string(), "WebUI 服务端口".to_string()),
        (
            "user".to_string(),
            "WebUI 用户名 (留空禁用认证)".to_string(),
        ),
        ("pass".to_string(), "WebUI 密码".to_string()),
        (
            "enable".to_string(),
            "是否启用 WebUI 服务；启用后即可通过当前 WebUI 修改配置".to_string(),
        ),
        (
            "cdn-prefix".to_string(),
            "CDN 前缀 (例如: https://cdn.jsdelivr.net/npm | https://cdn.jsdmirror.com/npm)"
                .to_string(),
        ),
    ])
}

pub fn max_number_of_one_records_comments() -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "Isp".to_string(),
            "自定义运营商IP最大单条写入数".to_string(),
        ),
        ("Ipv4".to_string(), "IPv4分组最大单条写入数".to_string()),
        ("Ipv6".to_string(), "IPv6分组最大单条写入数".to_string()),
        ("Domain".to_string(), "域名分流最大单条写入数".to_string()),
    ])
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
