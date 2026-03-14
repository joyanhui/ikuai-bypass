use serde::Serialize;

use super::clean::match_clean_tag;
use super::tag_name::{build_tag_name, match_tag_name_filter};
use super::types::{CustomIspData, IKuaiClient, IKuaiError, FUNC_NAME_CUSTOM_ISP, NEW_COMMENT};

#[derive(Debug, Serialize)]
struct ShowParam {
    #[serde(rename = "TYPE")]
    r#type: String,
    limit: String,
}

#[derive(Debug, Serialize)]
struct DelParam {
    id: String,
}

pub async fn show_custom_isp_by_tag_name(
    api: &IKuaiClient,
    tag_name: &str,
) -> Result<Vec<CustomIspData>, IKuaiError> {
    let param = ShowParam {
        r#type: "total,data".to_string(),
        limit: "0,1000".to_string(),
    };
    let resp = api
        .call::<_, Vec<CustomIspData>>(FUNC_NAME_CUSTOM_ISP, "show", &param)
        .await?;
    let data = resp
        .results
        .ok_or_else(|| IKuaiError::InvalidResponse("missing results".to_string()))?
        .data;
    Ok(data
        .into_iter()
        .filter(|d| match_tag_name_filter(tag_name, &d.name, &d.comment))
        .collect())
}

pub async fn add_custom_isp(api: &IKuaiClient, tag: &str, ipgroup: &str, index: i64) -> Result<(), IKuaiError> {
    let param = serde_json::json!({
        "name": build_tag_name(tag),
        "ipgroup": ipgroup.trim(),
        "comment": build_custom_isp_chunk_comment(index),
    });
    let _ = api
        .call::<_, serde_json::Value>(FUNC_NAME_CUSTOM_ISP, "add", &param)
        .await?;
    Ok(())
}

pub async fn edit_custom_isp(
    api: &IKuaiClient,
    tag: &str,
    ipgroup: &str,
    index: i64,
    id: i64,
) -> Result<(), IKuaiError> {
    let param = serde_json::json!({
        "name": build_tag_name(tag),
        "ipgroup": ipgroup.trim(),
        "comment": build_custom_isp_chunk_comment(index),
        "id": id,
    });
    let _ = api
        .call::<_, serde_json::Value>(FUNC_NAME_CUSTOM_ISP, "edit", &param)
        .await?;
    Ok(())
}

pub async fn del_custom_isp(api: &IKuaiClient, id_csv: &str) -> Result<(), IKuaiError> {
    let param = DelParam {
        id: id_csv.to_string(),
    };
    let _ = api
        .call::<_, serde_json::Value>(FUNC_NAME_CUSTOM_ISP, "del", &param)
        .await?;
    Ok(())
}

pub async fn get_custom_isp_map(api: &IKuaiClient, tag: &str) -> Result<std::collections::HashMap<i64, i64>, IKuaiError> {
    let data = show_custom_isp_by_tag_name(api, "").await?;
    let mut out = std::collections::HashMap::new();
    for d in data {
        if !match_tag_name_filter(tag, &d.name, &d.comment) {
            continue;
        }
        let idx = parse_custom_isp_chunk_index_from_comment(&d.comment)
            .or_else(|| parse_custom_isp_chunk_index_from_name(&d.name, tag));
        let Some(idx) = idx else { continue };
        out.entry(idx).or_insert(d.id);
    }
    Ok(out)
}

pub async fn del_custom_isp_all(api: &IKuaiClient, clean_tag: &str) -> Result<(), IKuaiError> {
    loop {
        let data = show_custom_isp_by_tag_name(api, "").await?;
        let ids: Vec<String> = data
            .into_iter()
            .filter(|d| match_clean_tag(clean_tag, &d.comment, &d.name))
            .map(|d| d.id.to_string())
            .collect();
        if ids.is_empty() {
            return Ok(());
        }
        del_custom_isp(api, &ids.join(",")).await?;
    }
}

fn build_custom_isp_chunk_comment(index: i64) -> String {
    let chunk = index + 1;
    if chunk <= 1 {
        return NEW_COMMENT.to_string();
    }
    format!("{}-{}", NEW_COMMENT, chunk)
}

fn parse_custom_isp_chunk_index_from_comment(comment: &str) -> Option<i64> {
    let c = comment.trim();
    if c.is_empty() {
        return None;
    }
    for prefix in [NEW_COMMENT, super::types::COMMENT_IKUAI_BYPASS] {
        if c == prefix {
            return Some(1);
        }
        if !c.starts_with(prefix) {
            continue;
        }
        let mut suffix = c.trim_start_matches(prefix);
        suffix = suffix.trim_start_matches('-');
        suffix = suffix.trim_start_matches('_');
        let suffix = suffix.trim();
        if suffix.is_empty() {
            return Some(1);
        }
        if let Ok(v) = suffix.parse::<i64>() && v > 0 {
            return Some(v);
        }
    }
    None
}

fn parse_custom_isp_chunk_index_from_name(name: &str, tag: &str) -> Option<i64> {
    let base = build_tag_name(tag);
    let suffix = name.trim().trim_start_matches(&base);
    if suffix.is_empty() {
        return None;
    }
    let v = suffix.parse::<i64>().ok()?;
    if v <= 0 {
        return None;
    }
    Some(v)
}
