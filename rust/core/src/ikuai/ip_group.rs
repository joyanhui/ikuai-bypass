use serde::{Deserialize, Serialize};

use super::clean::match_clean_tag;
use super::tag_name::{build_indexed_tag_name, build_tag_name, match_tag_name_filter};
use super::types::{IpGroupData, IKuaiClient, IKuaiError, FUNC_NAME_ROUTE_OBJECT};

#[derive(Debug, Deserialize)]
struct RouteObject4 {
    pub id: i64,
    #[serde(rename = "group_name")]
    pub group_name: String,
    pub r#type: i64,
    #[serde(rename = "group_value")]
    pub group_value: Vec<std::collections::HashMap<String, String>>,
    pub comment: String,
}

#[derive(Debug, Serialize)]
struct ShowParam {
    #[serde(rename = "TYPE")]
    r#type: String,
    limit: String,
    #[serde(rename = "FILTER1")]
    filter1: String,
}

#[derive(Debug, Serialize)]
struct DelParam {
    id: String,
}

pub async fn show_ip_group_by_tag_name(api: &IKuaiClient, tag_name: &str) -> Result<Vec<IpGroupData>, IKuaiError> {
    let param = ShowParam {
        r#type: "total,data".to_string(),
        limit: "0,1000".to_string(),
        filter1: "type,=,0".to_string(),
    };
    let resp = api
        .call::<_, Vec<RouteObject4>>(FUNC_NAME_ROUTE_OBJECT, "show", &param)
        .await?;
    let data = resp.results.ok_or(IKuaiError::InvalidResponse)?.data;

    let mut out = Vec::new();
    for d in data {
        if !match_tag_name_filter(tag_name, &d.group_name, &d.comment) {
            continue;
        }
        let mut ips = Vec::new();
        for v in &d.group_value {
            if let Some(ip) = v.get("ip") {
                ips.push(ip.clone());
            }
        }
        out.push(IpGroupData {
            id: d.id,
            group_name: d.group_name,
            addr_pool: ips.join(","),
            comment: d.comment,
            r#type: d.r#type,
        });
    }
    Ok(out)
}

pub async fn show_ip_group_by_name(api: &IKuaiClient, name: &str) -> Result<Vec<IpGroupData>, IKuaiError> {
    show_ip_group_by_tag_name(api, name).await
}

pub async fn add_ip_group(api: &IKuaiClient, tag: &str, addr_pool: &str, index: i64) -> Result<(), IKuaiError> {
    let ips: Vec<&str> = addr_pool
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let group_value: Vec<serde_json::Value> = ips
        .into_iter()
        .map(|ip| serde_json::json!({"ip": ip, "comment": ""}))
        .collect();
    let param = serde_json::json!({
        "group_name": build_indexed_tag_name(tag, index),
        "type": 0,
        "group_value": group_value,
        "comment": "",
    });
    let _ = api
        .call::<_, serde_json::Value>(FUNC_NAME_ROUTE_OBJECT, "add", &param)
        .await?;
    Ok(())
}

pub async fn edit_ip_group(api: &IKuaiClient, tag: &str, addr_pool: &str, index: i64, id: i64) -> Result<(), IKuaiError> {
    let ips: Vec<&str> = addr_pool
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let group_value: Vec<serde_json::Value> = ips
        .into_iter()
        .map(|ip| serde_json::json!({"ip": ip, "comment": ""}))
        .collect();
    let param = serde_json::json!({
        "group_name": build_indexed_tag_name(tag, index),
        "type": 0,
        "group_value": group_value,
        "comment": "",
        "id": id,
    });
    let _ = api
        .call::<_, serde_json::Value>(FUNC_NAME_ROUTE_OBJECT, "edit", &param)
        .await?;
    Ok(())
}

pub async fn del_ip_group(api: &IKuaiClient, id_csv: &str) -> Result<(), IKuaiError> {
    let param = DelParam {
        id: id_csv.to_string(),
    };
    let _ = api
        .call::<_, serde_json::Value>(FUNC_NAME_ROUTE_OBJECT, "del", &param)
        .await?;
    Ok(())
}

pub async fn get_ip_group_map(api: &IKuaiClient, tag: &str) -> Result<std::collections::HashMap<i64, i64>, IKuaiError> {
    let data = show_ip_group_by_tag_name(api, "").await?;
    let base = build_tag_name(tag);
    let mut out = std::collections::HashMap::new();
    for d in data {
        if !match_tag_name_filter(tag, &d.group_name, &d.comment) {
            continue;
        }
        let suffix = d.group_name.trim().trim_start_matches(&base);
        if let Ok(idx) = suffix.parse::<i64>() {
            out.insert(idx, d.id);
        }
    }
    Ok(out)
}

pub async fn del_ikuai_bypass_ip_group(api: &IKuaiClient, clean_tag: &str) -> Result<(), IKuaiError> {
    loop {
        let data = show_ip_group_by_tag_name(api, "").await?;
        let ids: Vec<String> = data
            .into_iter()
            .filter(|d| match_clean_tag(clean_tag, &d.comment, &d.group_name))
            .map(|d| d.id.to_string())
            .collect();
        if ids.is_empty() {
            return Ok(());
        }
        del_ip_group(api, &ids.join(",")).await?;
    }
}

pub async fn get_all_ikuai_bypass_ip_group_names_by_name(
    api: &IKuaiClient,
    name: &str,
) -> Result<Vec<String>, IKuaiError> {
    let data = show_ip_group_by_name(api, name).await?;
    Ok(data
        .into_iter()
        .filter(|d| match_tag_name_filter(name, &d.group_name, &d.comment))
        .map(|d| d.group_name)
        .collect())
}
