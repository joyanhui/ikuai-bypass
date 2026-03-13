use std::time::Duration;

use base64::Engine;
use serde::{Deserialize, Serialize};

pub const NAME_PREFIX_IKB: &str = "IKB";
pub const COMMENT_IKUAI_BYPASS: &str = "IKUAI_BYPASS";
pub const NEW_COMMENT: &str = "joyanhui/ikuai-bypass";
pub const CLEAN_MODE_ALL: &str = "cleanAll";

pub const FUNC_NAME_ROUTE_OBJECT: &str = "route_object";
pub const FUNC_NAME_CUSTOM_ISP: &str = "custom_isp";
pub const FUNC_NAME_STREAM_DOMAIN: &str = "stream_domain";
pub const FUNC_NAME_STREAM_IPPORT: &str = "stream_ipport";

#[derive(Debug, Clone)]
pub struct IKuaiClient {
    base_url: String,
    client: reqwest::Client,
}

#[derive(Debug, thiserror::Error)]
pub enum IKuaiError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("api error: {0}")]
    Api(String),
    #[error("invalid response: {0}")]
    InvalidResponse(String),
}

#[derive(Debug, Serialize)]
pub struct CallReq<T> {
    pub func_name: String,
    pub action: String,
    pub param: T,
}

#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
pub struct CallResp<T> {
    pub code: i64,
    pub message: String,
    #[serde(default)]
    pub results: Option<CallRespData<T>>,
    #[serde(default)]
    pub rowid: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
pub struct CallRespData<T> {
    #[serde(default)]
    pub total: Option<i64>,
    pub data: T,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CustomIspData {
    pub ipgroup: String,
    pub time: String,
    pub id: i64,
    pub comment: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct StreamDomainData {
    pub week: String,
    pub comment: String,
    #[serde(rename = "tagname")]
    pub tag_name: String,
    pub domain: String,
    #[serde(rename = "src_addr")]
    pub src_addr: String,
    pub interface: String,
    pub time: String,
    pub id: i64,
    pub enabled: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct IpGroupData {
    #[serde(rename = "addr_pool")]
    pub addr_pool: String,
    pub comment: String,
    #[serde(rename = "group_name")]
    pub group_name: String,
    pub id: i64,
    pub r#type: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Ipv6GroupData {
    #[serde(rename = "addr_pool")]
    pub addr_pool: String,
    pub comment: String,
    #[serde(rename = "group_name")]
    pub group_name: String,
    pub id: i64,
    pub r#type: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct StreamIpPortData {
    pub protocol: String,
    #[serde(rename = "tagname")]
    pub tag_name: String,
    #[serde(rename = "src_port")]
    pub src_port: String,
    pub id: i64,
    pub enabled: String,
    pub week: String,
    pub comment: String,
    pub time: String,
    pub nexthop: String,
    #[serde(rename = "iface_band")]
    pub iface_band: i64,
    pub interface: String,
    pub mode: i64,
    #[serde(rename = "src_addr")]
    pub src_addr: String,
    #[serde(rename = "dst_port")]
    pub dst_port: String,
    #[serde(rename = "dst_addr")]
    pub dst_addr: String,
    pub r#type: i64,
}

impl IKuaiClient {
    pub fn new(base_url: String) -> Result<Self, IKuaiError> {
        // 避免网络异常时无限期卡住（比如爱快地址不可达）。
        // Avoid hanging forever when iKuai is unreachable.
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self { base_url, client })
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<(), IKuaiError> {
        let passwd = crate::ikuai::utils::md5_hex(password);
        let pass = base64::engine::general_purpose::STANDARD
            .encode(format!("salt_11{}", password));

        let req = serde_json::json!({
            "passwd": passwd,
            "pass": pass,
            "remember_password": "",
            "username": username,
        });
        let url = format!("{}/Action/login", self.base_url.trim_end_matches('/'));
        let text = self.post_json_text(&url, &req).await?;
        let resp: CallResp<serde_json::Value> = parse_call_response(&text)?;
        if resp.code != 0 {
            return Err(IKuaiError::Api(resp.message));
        }
        Ok(())
    }

    pub async fn call<TParam: Serialize, TData: for<'de> Deserialize<'de>>(
        &self,
        func_name: &str,
        action: &str,
        param: &TParam,
    ) -> Result<CallResp<TData>, IKuaiError> {
        let url = format!("{}/Action/call", self.base_url.trim_end_matches('/'));
        let req = CallReq {
            func_name: func_name.to_string(),
            action: action.to_string(),
            param,
        };
        let text = self.post_json_text(&url, &req).await?;
        let resp: CallResp<TData> = parse_call_response(&text)?;
        if resp.code != 0 {
            return Err(IKuaiError::Api(resp.message.to_string()));
        }
        Ok(resp)
    }

    async fn post_json_text<T: Serialize>(&self, url: &str, body: &T) -> Result<String, IKuaiError> {
        let resp = self.client.post(url).json(body).send().await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            return Err(IKuaiError::Api(format!(
                "http status {}: {}",
                status,
                trim_body(&text)
            )));
        }
        Ok(text)
    }
}

fn parse_call_response<T: for<'de> Deserialize<'de>>(text: &str) -> Result<CallResp<T>, IKuaiError> {
    serde_json::from_str(text).map_err(|e| {
        IKuaiError::InvalidResponse(format!("decode error: {} body: {}", e, trim_body(text)))
    })
}

fn trim_body(text: &str) -> String {
    let trimmed = text.trim();
    const LIMIT: usize = 200;
    if trimmed.len() <= LIMIT {
        return trimmed.to_string();
    }
    let mut end = 0usize;
    for (i, ch) in trimmed.char_indices() {
        let next = i + ch.len_utf8();
        if next > LIMIT {
            break;
        }
        end = next;
    }
    let mut out = trimmed[..end].to_string();
    out.push_str("...");
    out
}
