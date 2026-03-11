use serde::{Deserialize, Serialize};

use base64::Engine;

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
    #[error("invalid response")]
    InvalidResponse,
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
        let client = reqwest::Client::builder()
            .cookie_store(true)
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
        let resp: CallResp<serde_json::Value> = self
            .client
            .post(url)
            .json(&req)
            .send()
            .await?
            .json()
            .await?;
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
        let resp: CallResp<TData> = self
            .client
            .post(url)
            .json(&req)
            .send()
            .await?
            .json()
            .await?;
        if resp.code != 0 {
            return Err(IKuaiError::Api(resp.message.clone()));
        }
        Ok(resp)
    }
}
