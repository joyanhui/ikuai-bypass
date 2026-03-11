use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::clean::match_clean_tag;
use super::ip_group;
use super::tag_name::{build_indexed_tag_name, match_tag_name_filter};
use super::types::{IKuaiClient, IKuaiError, StreamDomainData, FUNC_NAME_STREAM_DOMAIN, NEW_COMMENT};
use super::utils::{categorize_addrs, to_string_list};

#[derive(Debug, Deserialize)]
struct AddrBlock {
    custom: Value,
    object: Value,
}

#[derive(Debug, Deserialize)]
struct TimeCustom {
    weekdays: String,
    start_time: String,
    end_time: String,
}

#[derive(Debug, Deserialize)]
struct TimeBlock {
    custom: Vec<TimeCustom>,
    object: Value,
}

#[derive(Debug, Deserialize)]
struct StreamDomain4 {
    id: i64,
    enabled: String,
    #[serde(rename = "tagname")]
    tagname: String,
    interface: String,
    comment: String,
    prio: i64,
    #[serde(rename = "src_addr")]
    src_addr: AddrBlock,
    domain: AddrBlock,
    time: TimeBlock,
}

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

pub async fn show_stream_domain_by_tag_name(
    api: &IKuaiClient,
    tag_name: &str,
) -> Result<Vec<StreamDomainData>, IKuaiError> {
    let param = ShowParam {
        r#type: "total,data".to_string(),
        limit: "0,1000".to_string(),
    };
    let resp = api
        .call::<_, Vec<StreamDomain4>>(FUNC_NAME_STREAM_DOMAIN, "show", &param)
        .await?;
    let data = resp.results.ok_or(IKuaiError::InvalidResponse)?.data;
    let mut out = Vec::new();
    for d in data {
        if !match_tag_name_filter(tag_name, &d.tagname, &d.comment) {
            continue;
        }
        let srcs = {
            let mut v = to_string_list(&d.src_addr.custom);
            v.extend(to_string_list(&d.src_addr.object));
            v
        };
        let domains = {
            let mut v = to_string_list(&d.domain.custom);
            v.extend(to_string_list(&d.domain.object));
            v
        };
        let mut item = StreamDomainData {
            id: d.id,
            enabled: d.enabled,
            comment: d.comment,
            tag_name: d.tagname,
            interface: d.interface,
            domain: domains.join(","),
            src_addr: srcs.join(","),
            week: String::new(),
            time: String::new(),
        };
        if let Some(t) = d.time.custom.first() {
            item.week = t.weekdays.clone();
            item.time = format!("{}-{}", t.start_time, t.end_time);
        }
        out.push(item);
    }
    Ok(out)
}

pub async fn add_stream_domain(
    api: &IKuaiClient,
    iface: &str,
    tag: &str,
    src_addr: &str,
    src_addr_opt_ipgroup: &str,
    domains: &str,
    index: i64,
) -> Result<(), IKuaiError> {
    let domain_list: Vec<String> = domains
        .trim()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let (src_custom, src_objects) = resolve_src_addrs(api, src_addr, src_addr_opt_ipgroup).await?;
    let unique_tagname = build_indexed_tag_name(tag, index);

    let param = serde_json::json!({
        "enabled": "yes",
        "tagname": unique_tagname,
        "interface": iface,
        "src_addr": {"custom": src_custom, "object": src_objects},
        "domain": {"custom": domain_list, "object": []},
        "comment": NEW_COMMENT,
        "time": {
            "custom": [{"type":"weekly","weekdays":"1234567","start_time":"00:00","end_time":"23:59","comment":""}],
            "object": []
        },
        "prio": 31
    });
    let _ = api
        .call::<_, serde_json::Value>(FUNC_NAME_STREAM_DOMAIN, "add", &param)
        .await?;
    Ok(())
}

pub async fn edit_stream_domain(
    api: &IKuaiClient,
    iface: &str,
    tag: &str,
    src_addr: &str,
    src_addr_opt_ipgroup: &str,
    domains: &str,
    index: i64,
    id: i64,
) -> Result<(), IKuaiError> {
    let domain_list: Vec<String> = domains
        .trim()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let (src_custom, src_objects) = resolve_src_addrs(api, src_addr, src_addr_opt_ipgroup).await?;
    let unique_tagname = build_indexed_tag_name(tag, index);

    let param = serde_json::json!({
        "enabled": "yes",
        "tagname": unique_tagname,
        "interface": iface,
        "src_addr": {"custom": src_custom, "object": src_objects},
        "domain": {"custom": domain_list, "object": []},
        "comment": NEW_COMMENT,
        "time": {
            "custom": [{"type":"weekly","weekdays":"1234567","start_time":"00:00","end_time":"23:59","comment":""}],
            "object": []
        },
        "prio": 31,
        "id": id
    });
    let _ = api
        .call::<_, serde_json::Value>(FUNC_NAME_STREAM_DOMAIN, "edit", &param)
        .await?;
    Ok(())
}

pub async fn del_stream_domain(api: &IKuaiClient, id_csv: &str) -> Result<(), IKuaiError> {
    let param = DelParam {
        id: id_csv.to_string(),
    };
    let _ = api
        .call::<_, serde_json::Value>(FUNC_NAME_STREAM_DOMAIN, "del", &param)
        .await?;
    Ok(())
}

pub async fn get_stream_domain_map(
    api: &IKuaiClient,
    tag: &str,
) -> Result<std::collections::HashMap<i64, i64>, IKuaiError> {
    let data = show_stream_domain_by_tag_name(api, "").await?;
    let base = super::tag_name::build_tag_name(tag);
    let mut out = std::collections::HashMap::new();
    for d in data {
        if !match_tag_name_filter(tag, &d.tag_name, &d.comment) {
            continue;
        }
        let suffix = d.tag_name.trim().trim_start_matches(&base);
        if let Ok(idx) = suffix.parse::<i64>() {
            out.insert(idx, d.id);
        }
    }
    Ok(out)
}

pub async fn del_stream_domain_all(api: &IKuaiClient, clean_tag: &str) -> Result<(), IKuaiError> {
    loop {
        let data = show_stream_domain_by_tag_name(api, "").await?;
        let ids: Vec<String> = data
            .into_iter()
            .filter(|d| match_clean_tag(clean_tag, &d.comment, &d.tag_name))
            .map(|d| d.id.to_string())
            .collect();
        if ids.is_empty() {
            return Ok(());
        }
        del_stream_domain(api, &ids.join(",")).await?;
    }
}

async fn resolve_src_addrs(
    api: &IKuaiClient,
    src_addr: &str,
    src_addr_opt_ipgroup: &str,
) -> Result<(Vec<String>, Vec<Value>), IKuaiError> {
    if !src_addr_opt_ipgroup.trim().is_empty() {
        let mut resolved = Vec::new();
        for name in src_addr_opt_ipgroup.split(',') {
            let name = name.trim();
            if name.is_empty() {
                continue;
            }
            if let Ok(matches) = ip_group::get_all_ikuai_bypass_ip_group_names_by_name(api, name).await {
                resolved.extend(matches);
            }
        }
        let objects = resolve_ip_group_objects(api, &resolved).await?;
        return Ok((Vec::new(), objects));
    }

    let mut src_list = Vec::new();
    let src_addr = src_addr.trim();
    if !src_addr.is_empty() {
        src_list.extend(src_addr.split(',').map(|s| s.trim().to_string()));
    }
    let (custom, obj_names) = categorize_addrs(&src_list);
    let objects = resolve_ip_group_objects(api, &obj_names).await?;
    Ok((custom, objects))
}

async fn resolve_ip_group_objects(api: &IKuaiClient, names: &[String]) -> Result<Vec<Value>, IKuaiError> {
    if names.is_empty() {
        return Ok(Vec::new());
    }
    let groups = ip_group::show_ip_group_by_tag_name(api, "").await?;
    let mut map = std::collections::HashMap::new();
    for g in groups {
        map.insert(g.group_name.clone(), g.id);
    }
    let mut out = Vec::new();
    for name in names {
        if let Some(id) = map.get(name) {
            out.push(serde_json::json!({
                "type": 0,
                "gid": format!("IPGP{}", id),
                "gp_name": name
            }));
        }
    }
    Ok(out)
}
