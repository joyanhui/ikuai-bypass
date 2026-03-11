use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::clean::match_clean_tag;
use super::tag_name::{build_tag_name, match_tag_name_filter};
use super::types::{IKuaiClient, IKuaiError, StreamIpPortData, FUNC_NAME_STREAM_IPPORT, NEW_COMMENT};
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
    #[serde(rename = "object")]
    _object: Value,
}

#[derive(Debug, Deserialize)]
struct StreamIpPort4 {
    id: i64,
    enabled: String,
    #[serde(rename = "tagname")]
    tagname: String,
    interface: String,
    nexthop: String,
    comment: String,
    #[serde(rename = "iface_band")]
    iface_band: i64,
    mode: i64,
    protocol: String,
    r#type: i64,
    #[serde(rename = "src_addr")]
    src_addr: AddrBlock,
    #[serde(rename = "dst_addr")]
    dst_addr: AddrBlock,
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

pub async fn show_stream_ipport_by_tag_name(api: &IKuaiClient, tag_name: &str) -> Result<Vec<StreamIpPortData>, IKuaiError> {
    let param = ShowParam {
        r#type: "total,data".to_string(),
        limit: "0,1000".to_string(),
    };
    let resp = api
        .call::<_, Vec<StreamIpPort4>>(FUNC_NAME_STREAM_IPPORT, "show", &param)
        .await?;
    let data = resp.results.ok_or(IKuaiError::InvalidResponse)?.data;
    let mut out = Vec::new();
    for d in data {
        if !match_tag_name_filter(tag_name, &d.tagname, &d.comment) {
            continue;
        }
        let mut srcs = to_string_list(&d.src_addr.custom);
        srcs.extend(to_string_list(&d.src_addr.object));
        let mut dsts = to_string_list(&d.dst_addr.custom);
        dsts.extend(to_string_list(&d.dst_addr.object));
        let mut item = StreamIpPortData {
            id: d.id,
            enabled: d.enabled,
            comment: d.comment,
            tag_name: d.tagname,
            interface: d.interface,
            nexthop: d.nexthop,
            iface_band: d.iface_band,
            mode: d.mode,
            protocol: d.protocol,
            r#type: d.r#type,
            src_addr: srcs.join(","),
            dst_addr: dsts.join(","),
            src_port: String::new(),
            dst_port: String::new(),
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

pub async fn get_stream_ipport_map(api: &IKuaiClient, tag: &str) -> Result<std::collections::HashMap<String, i64>, IKuaiError> {
    let data = show_stream_ipport_by_tag_name(api, "").await?;
    let mut out = std::collections::HashMap::new();
    for d in data {
        if match_tag_name_filter(tag, &d.tag_name, &d.comment) {
            out.insert(d.tag_name.clone(), d.id);
        }
    }
    Ok(out)
}

pub async fn add_stream_ipport(
    api: &IKuaiClient,
    forward_type: &str,
    iface: &str,
    dst_addr: &str,
    src_addr: &str,
    nexthop: &str,
    tag: &str,
    mode: i64,
    iface_band: i64,
) -> Result<(), IKuaiError> {
    let f_type: i64 = forward_type.parse().unwrap_or(0);
    let src_list: Vec<String> = src_addr
        .trim()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let dst_list: Vec<String> = dst_addr
        .trim()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let (src_custom, src_obj) = categorize_addrs(&src_list);
    let (dst_custom, dst_obj) = categorize_addrs(&dst_list);
    let src_objects = resolve_ip_group_objects(api, &src_obj).await?;
    let dst_objects = resolve_ip_group_objects(api, &dst_obj).await?;
    let param = serde_json::json!({
        "enabled": "yes",
        "tagname": build_tag_name(tag),
        "interface": iface,
        "nexthop": nexthop,
        "iface_band": iface_band,
        "comment": NEW_COMMENT,
        "type": f_type,
        "mode": mode,
        "protocol": "tcp+udp",
        "src_addr": {"custom": src_custom, "object": src_objects},
        "dst_addr": {"custom": dst_custom, "object": dst_objects},
        "src_port": {"custom": [], "object": []},
        "dst_port": {"custom": [], "object": []},
        "time": {"custom": [{"type":"weekly","weekdays":"1234567","start_time":"00:00","end_time":"23:59","comment":""}], "object": []},
        "prio": 0,
        "area_code": "",
        "dst_type": ""
    });
    let _ = api
        .call::<_, serde_json::Value>(FUNC_NAME_STREAM_IPPORT, "add", &param)
        .await?;
    Ok(())
}

pub async fn edit_stream_ipport(
    api: &IKuaiClient,
    forward_type: &str,
    iface: &str,
    dst_addr: &str,
    src_addr: &str,
    nexthop: &str,
    tag: &str,
    mode: i64,
    iface_band: i64,
    id: i64,
) -> Result<(), IKuaiError> {
    let f_type: i64 = forward_type.parse().unwrap_or(0);
    let src_list: Vec<String> = src_addr
        .trim()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let dst_list: Vec<String> = dst_addr
        .trim()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let (src_custom, src_obj) = categorize_addrs(&src_list);
    let (dst_custom, dst_obj) = categorize_addrs(&dst_list);
    let src_objects = resolve_ip_group_objects(api, &src_obj).await?;
    let dst_objects = resolve_ip_group_objects(api, &dst_obj).await?;
    let param = serde_json::json!({
        "enabled": "yes",
        "tagname": build_tag_name(tag),
        "interface": iface,
        "nexthop": nexthop,
        "iface_band": iface_band,
        "comment": NEW_COMMENT,
        "type": f_type,
        "mode": mode,
        "protocol": "tcp+udp",
        "src_addr": {"custom": src_custom, "object": src_objects},
        "dst_addr": {"custom": dst_custom, "object": dst_objects},
        "src_port": {"custom": [], "object": []},
        "dst_port": {"custom": [], "object": []},
        "time": {"custom": [{"type":"weekly","weekdays":"1234567","start_time":"00:00","end_time":"23:59","comment":""}], "object": []},
        "prio": 0,
        "area_code": "",
        "dst_type": "",
        "id": id
    });
    let _ = api
        .call::<_, serde_json::Value>(FUNC_NAME_STREAM_IPPORT, "edit", &param)
        .await?;
    Ok(())
}

pub async fn del_stream_ipport(api: &IKuaiClient, id_csv: &str) -> Result<(), IKuaiError> {
    let param = DelParam {
        id: id_csv.to_string(),
    };
    let _ = api
        .call::<_, serde_json::Value>(FUNC_NAME_STREAM_IPPORT, "del", &param)
        .await?;
    Ok(())
}

pub async fn del_ikuai_bypass_stream_ipport(api: &IKuaiClient, clean_tag: &str) -> Result<(), IKuaiError> {
    loop {
        let data = show_stream_ipport_by_tag_name(api, "").await?;
        let ids: Vec<String> = data
            .into_iter()
            .filter(|d| match_clean_tag(clean_tag, &d.comment, &d.tag_name))
            .map(|d| d.id.to_string())
            .collect();
        if ids.is_empty() {
            return Ok(());
        }
        del_stream_ipport(api, &ids.join(",")).await?;
    }
}

async fn resolve_ip_group_objects(api: &IKuaiClient, names: &[String]) -> Result<Vec<Value>, IKuaiError> {
    if names.is_empty() {
        return Ok(Vec::new());
    }
    let groups = super::ip_group::show_ip_group_by_tag_name(api, "").await?;
    let mut map = std::collections::HashMap::new();
    for g in groups {
        map.insert(g.group_name.clone(), g.id);
    }
    let mut out = Vec::new();
    for name in names {
        if let Some(id) = map.get(name) {
            out.push(serde_json::json!({"type": 0, "gid": format!("IPGP{}", id), "gp_name": name}));
        }
    }
    Ok(out)
}
