use std::sync::Arc;
use std::time::Duration;

use crate::config::Config;
use crate::logger::{LogSink, Logger};
use crate::ikuai;
use crate::session::{resolve_login_params, LoginParamsError};

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("login params error: {0}")]
    LoginParams(#[from] LoginParamsError),
    #[error("ikuai error: {0}")]
    IKuai(#[from] ikuai::IKuaiError),
    #[error("download error: {0}")]
    Download(String),
    #[error("invalid -m parameter: {0}")]
    InvalidModule(String),
}

pub async fn run_update_by_module(
    cfg: &Config,
    cli_login: &str,
    module: &str,
    sink: LogSink,
) -> Result<(), UpdateError> {
    let params = resolve_login_params(cfg, cli_login)?;
    let api = ikuai::IKuaiClient::new(params.base_url.clone())?;
    api.login(&params.username, &params.password).await?;

    let sys = Logger::new("SYS:系统组件", Arc::clone(&sink));

    match module {
        "ispdomain" => {
            sys.info("TASK:任务启动", "Starting ISP and Domain streaming mode");
            update_ispdomain(cfg, &api, &sink).await;
        }
        "ipgroup" => {
            sys.info("TASK:任务启动", "Starting IP group and Next-hop gateway mode");
            update_ipgroup(cfg, &api, &sink).await;
        }
        "ipv6group" => {
            sys.info("TASK:任务启动", "Starting IPv6 group mode");
            update_ipv6group(cfg, &api, &sink).await;
        }
        "ii" => {
            sys.info("TASK:任务启动", "Starting hybrid mode: ISP/Domain + IP group");
            update_ispdomain(cfg, &api, &sink).await;
            update_ipgroup(cfg, &api, &sink).await;
        }
        "ip" => {
            sys.info("TASK:任务启动", "Starting hybrid mode: IPv4 group + IPv6 group");
            update_ipgroup(cfg, &api, &sink).await;
            update_ipv6group(cfg, &api, &sink).await;
        }
        "iip" => {
            sys.info("TASK:任务启动", "Starting full hybrid mode: ISP/Domain + IPv4/v6 group");
            update_ispdomain(cfg, &api, &sink).await;
            update_ipgroup(cfg, &api, &sink).await;
            update_ipv6group(cfg, &api, &sink).await;
        }
        other => return Err(UpdateError::InvalidModule(other.to_string())),
    }

    Ok(())
}

async fn update_ispdomain(cfg: &Config, api: &ikuai::IKuaiClient, sink: &LogSink) {
    let isp_logger = Logger::new("ISP:运营商分流", Arc::clone(sink));
    let domain_logger = Logger::new("DOMAIN:域名分流", Arc::clone(sink));
    let sys = Logger::new("SYS:系统组件", Arc::clone(sink));

    for item in &cfg.custom_isp {
        isp_logger.info("UPDATE:开始更新", format!("Updating {}...", item.tag));
        let res = update_custom_isp(cfg, api, sink, &item.tag, &item.url).await;
        if let Err(e) = res {
            isp_logger.error(
                "UPDATE:更新失败",
                format!("Failed to update custom ISP '{}': {}", item.tag, e),
            );
        } else {
            isp_logger.success(
                "UPDATE:更新成功",
                format!("Successfully updated custom ISP '{}'", item.tag),
            );
        }
    }

    for item in &cfg.stream_domain {
        domain_logger.info(
            "UPDATE:开始更新",
            format!(
                "Updating {} (Interface: {}, Tag: {})...",
                item.url, item.interface, item.tag
            ),
        );
        let res = update_stream_domain(
            cfg,
            api,
            sink,
            &item.interface,
            &item.tag,
            &item.src_addr_opt_ipgroup,
            &item.src_addr,
            &item.url,
        )
        .await;
        if let Err(e) = res {
            domain_logger.error(
                "UPDATE:更新失败",
                format!("Failed to update domain streaming for tag {}: {}", item.tag, e),
            );
        } else {
            domain_logger.success(
                "UPDATE:更新成功",
                format!("Successfully updated domain streaming for tag {}", item.tag),
            );
        }
    }

    sys.success(
        "DONE:任务完成",
        "ISP and Domain streaming update tasks completed",
    );
}

async fn update_ipgroup(cfg: &Config, api: &ikuai::IKuaiClient, sink: &LogSink) {
    let ip_logger = Logger::new("IP:IP分组", Arc::clone(sink));
    let stream_logger = Logger::new("STREAM:端口分流", Arc::clone(sink));

    for item in &cfg.ip_group {
        let res = update_ip_group(cfg, api, sink, &item.tag, &item.url).await;
        if let Err(e) = res {
            ip_logger.error(
                "UPDATE:更新失败",
                format!("Failed to add IP group '{}@{}': {}", item.tag, item.url, e),
            );
        } else {
            ip_logger.success(
                "UPDATE:更新成功",
                format!("Successfully updated IP group '{}@{}'", item.tag, item.url),
            );
        }
    }

    for item in &cfg.stream_ipport {
        let tag = if !item.opt_tagname.trim().is_empty() {
            item.opt_tagname.clone()
        } else {
            format!("{}{}", item.interface, item.nexthop)
        };
        if tag.trim().is_empty() {
            stream_logger.error(
                "VALID:参数校验",
                format!(
                    "Rule name and IpGroup cannot both be empty, skipping: interface='{}' nexthop='{}'",
                    item.interface, item.nexthop
                ),
            );
            continue;
        }

        stream_logger.info(
            "UPDATE:开始更新",
            format!("Updating port streaming for tag {}...", tag),
        );
        let res = update_stream_ipport(
            cfg,
            api,
            sink,
            &item.r#type,
            &tag,
            &item.interface,
            &item.nexthop,
            &item.src_addr,
            &item.src_addr_opt_ipgroup,
            &item.ip_group,
            item.mode,
            item.ifaceband,
        )
        .await;
        if let Err(e) = res {
            stream_logger.error(
                "UPDATE:更新失败",
                format!(
                    "Failed to update port streaming '{}@{}': {}",
                    format!("{}{}", item.interface, item.nexthop),
                    item.ip_group,
                    e
                ),
            );
        } else {
            stream_logger.success(
                "UPDATE:更新成功",
                format!(
                    "Successfully updated port streaming '{}@{}'",
                    format!("{}{}", item.interface, item.nexthop),
                    item.ip_group
                ),
            );
        }
    }
}

async fn update_ipv6group(cfg: &Config, api: &ikuai::IKuaiClient, sink: &LogSink) {
    let ipv6_logger = Logger::new("IPV6:IPv6分组", Arc::clone(sink));

    for item in &cfg.ipv6_group {
        let res = update_ipv6_group(cfg, api, sink, &item.tag, &item.url).await;
        if let Err(e) = res {
            ipv6_logger.error(
                "UPDATE:更新失败",
                format!("Failed to add IPv6 group '{}@{}': {}", item.tag, item.url, e),
            );
        } else {
            ipv6_logger.success(
                "UPDATE:更新成功",
                format!("Successfully updated IPv6 group '{}@{}'", item.tag, item.url),
            );
        }
    }
}

async fn update_custom_isp(
    cfg: &Config,
    api: &ikuai::IKuaiClient,
    sink: &LogSink,
    tag: &str,
    url: &str,
) -> Result<(), UpdateError> {
    let isp_logger = Logger::new("ISP:运营商分流", Arc::clone(sink));

    let body = http_get(cfg, sink, url).await?;
    let mut ips = split_lines(&body);
    ips = remove_ipv6_and_empty(sink, ips);
    isp_logger.info("STAT:规则统计", format!("Fetched {} IPs for {}", ips.len(), tag));

    let mut map = ikuai::custom_isp::get_custom_isp_map(api, tag).await?;
    let groups = group(ips, cfg.max_number_of_one_records.isp as usize);
    for (i, chunk) in groups.iter().enumerate() {
        let index = (i + 1) as i64;
        let ip_group = chunk.join(",");
        let res = if let Some(id) = map.remove(&index) {
            isp_logger.info(
                "EDIT:正在修改",
                format!(
                    "[{}/{}] {}: updating chunk {} (ID: {})...",
                    i + 1,
                    groups.len(),
                    tag,
                    index,
                    id
                ),
            );
            ikuai::custom_isp::edit_custom_isp(api, tag, &ip_group, i as i64, id).await
        } else {
            isp_logger.info(
                "ADD:正在添加",
                format!(
                    "[{}/{}] {}: adding chunk {}...",
                    i + 1,
                    groups.len(),
                    tag,
                    index
                ),
            );
            ikuai::custom_isp::add_custom_isp(api, tag, &ip_group, i as i64).await
        };

        if let Err(e) = res {
            isp_logger.error(
                "UPDATE:更新失败",
                format!(
                    "[{}/{}] {}: failed: {}",
                    i + 1,
                    groups.len(),
                    tag,
                    e
                ),
            );
            sleep(cfg.add_err_retry_wait).await;
        } else {
            sleep(cfg.add_wait).await;
        }
    }

    if !map.is_empty() {
        let mut extra = Vec::new();
        for (idx, id) in map {
            isp_logger.info(
                "CLEAN:冗余删除",
                format!(
                    "{}: chunk {} (ID: {}) is no longer needed, deleting...",
                    tag, idx, id
                ),
            );
            extra.push(id.to_string());
        }
        let joined = extra.join(",");
        let res = ikuai::custom_isp::del_custom_isp(api, &joined).await;
        if let Err(e) = res {
            isp_logger.error(
                "CLEAN:删除失败",
                format!("{}: failed to delete extra rules: {}", tag, e),
            );
        } else {
            isp_logger.success(
                "CLEAN:清理成功",
                format!("{}: deleted {} extra rules", tag, extra.len()),
            );
        }
    }

    Ok(())
}

async fn update_stream_domain(
    cfg: &Config,
    api: &ikuai::IKuaiClient,
    sink: &LogSink,
    iface: &str,
    tag: &str,
    src_addr_ipgroup: &str,
    src_addr: &str,
    url: &str,
) -> Result<(), UpdateError> {
    let domain_logger = Logger::new("DOMAIN:域名分流", Arc::clone(sink));

    let body = http_get(cfg, sink, url).await?;
    let mut domains = split_lines(&body);
    domains = filter_domains(sink, domains);
    domain_logger.success(
        "PARSE:解析成功",
        format!(
            "{} {}: obtained {} valid domains",
            iface,
            tag,
            domains.len()
        ),
    );

    let mut map = ikuai::stream_domain::get_stream_domain_map(api, tag).await?;
    let groups = group(domains, cfg.max_number_of_one_records.domain as usize);
    for (i, chunk) in groups.iter().enumerate() {
        let index = (i + 1) as i64;
        let name = ikuai::tag_name::build_indexed_tag_name(tag, i as i64);
        let joined = chunk.join(",");
        let res = if let Some(id) = map.remove(&index) {
            domain_logger.info(
                "EDIT:正在修改",
                format!(
                    "[{}/{}] {} {}: updating {} (ID: {})...",
                    i + 1,
                    groups.len(),
                    iface,
                    tag,
                    name,
                    id
                ),
            );
            ikuai::stream_domain::edit_stream_domain(
                api,
                iface,
                tag,
                src_addr,
                src_addr_ipgroup,
                &joined,
                i as i64,
                id,
            )
            .await
        } else {
            domain_logger.info(
                "ADD:正在添加",
                format!(
                    "[{}/{}] {} {}: adding {}...",
                    i + 1,
                    groups.len(),
                    iface,
                    tag,
                    name
                ),
            );
            ikuai::stream_domain::add_stream_domain(
                api,
                iface,
                tag,
                src_addr,
                src_addr_ipgroup,
                &joined,
                i as i64,
            )
            .await
        };

        if let Err(e) = res {
            domain_logger.error(
                "UPDATE:更新失败",
                format!(
                    "[{}/{}] {} {}: failed: {}",
                    i + 1,
                    groups.len(),
                    iface,
                    tag,
                    e
                ),
            );
            sleep(cfg.add_err_retry_wait).await;
        } else {
            sleep(cfg.add_wait).await;
        }
    }

    if !map.is_empty() {
        let mut extra = Vec::new();
        for (idx, id) in map {
            domain_logger.info(
                "CLEAN:冗余删除",
                format!(
                    "{}: chunk {} (ID: {}) is no longer needed, deleting...",
                    tag, idx, id
                ),
            );
            extra.push(id.to_string());
        }
        let res = ikuai::stream_domain::del_stream_domain(api, &extra.join(",")).await;
        if let Err(e) = res {
            domain_logger.error(
                "CLEAN:删除失败",
                format!("{}: failed to delete extra domain rules: {}", tag, e),
            );
        } else {
            domain_logger.success(
                "CLEAN:清理成功",
                format!("{}: deleted {} extra domain rules", tag, extra.len()),
            );
        }
    }

    Ok(())
}

async fn update_ip_group(
    cfg: &Config,
    api: &ikuai::IKuaiClient,
    sink: &LogSink,
    tag: &str,
    url: &str,
) -> Result<(), UpdateError> {
    let ip_logger = Logger::new("IP:IP分组", Arc::clone(sink));

    let body = http_get(cfg, sink, url).await?;
    let mut ips = split_lines(&body);
    ips = remove_ipv6_and_empty(sink, ips);
    let groups = group(ips, cfg.max_number_of_one_records.ipv4 as usize);
    ip_logger.success("PARSE:解析成功", format!("{}: obtained new data", tag));

    let mut map = ikuai::ip_group::get_ip_group_map(api, tag).await?;
    ip_logger.info(
        "QUERY:查询成功",
        format!("{}: found {} existing groups", tag, map.len()),
    );

    for (i, chunk) in groups.iter().enumerate() {
        let index = (i + 1) as i64;
        let name = ikuai::tag_name::build_indexed_tag_name(tag, i as i64);
        let joined = chunk.join(",");
        let res = if let Some(id) = map.remove(&index) {
            ip_logger.info(
                "EDIT:正在修改",
                format!(
                    "[{}/{}] {}: updating {} (ID: {})...",
                    i + 1,
                    groups.len(),
                    tag,
                    name,
                    id
                ),
            );
            ikuai::ip_group::edit_ip_group(api, tag, &joined, i as i64, id).await
        } else {
            ip_logger.info(
                "ADD:正在添加",
                format!(
                    "[{}/{}] {}: adding {}...",
                    i + 1,
                    groups.len(),
                    tag,
                    name
                ),
            );
            ikuai::ip_group::add_ip_group(api, tag, &joined, i as i64).await
        };

        if let Err(e) = res {
            ip_logger.error(
                "UPDATE:更新失败",
                format!(
                    "[{}/{}] {}: failed, error: {}",
                    i + 1,
                    groups.len(),
                    tag,
                    e
                ),
            );
            sleep(cfg.add_err_retry_wait).await;
        }
    }

    if !map.is_empty() {
        let extra: Vec<String> = map.values().map(|v| v.to_string()).collect();
        ip_logger.info(
            "CLEAN:冗余删除",
            format!(
                "{}: {} groups are no longer needed, deleting IDs: {}",
                tag,
                map.len(),
                extra.join(",")
            ),
        );
        let res = ikuai::ip_group::del_ip_group(api, &extra.join(",")).await;
        if let Err(e) = res {
            ip_logger.error(
                "CLEAN:删除失败",
                format!("{}: failed to delete extra groups: {}", tag, e),
            );
        } else {
            ip_logger.success(
                "CLEAN:清理成功",
                format!("{}: deleted {} extra groups", tag, extra.len()),
            );
        }
    }

    Ok(())
}

async fn update_ipv6_group(
    cfg: &Config,
    api: &ikuai::IKuaiClient,
    sink: &LogSink,
    tag: &str,
    url: &str,
) -> Result<(), UpdateError> {
    let ipv6_logger = Logger::new("IPV6:IPv6分组", Arc::clone(sink));

    let body = http_get(cfg, sink, url).await?;
    let mut ips = split_lines(&body);
    ips = remove_ipv4_and_empty(sink, ips);
    let groups = group(ips, cfg.max_number_of_one_records.ipv6 as usize);
    ipv6_logger.success("PARSE:解析成功", format!("{}: obtained new data", tag));

    let mut map = ikuai::ipv6_group::get_ipv6_group_map(api, tag).await?;
    ipv6_logger.info(
        "QUERY:查询成功",
        format!("{}: found {} existing IPv6 groups", tag, map.len()),
    );

    for (i, chunk) in groups.iter().enumerate() {
        let index = (i + 1) as i64;
        let name = ikuai::tag_name::build_indexed_tag_name(tag, i as i64);
        let joined = chunk.join(",");
        let res = if let Some(id) = map.remove(&index) {
            ipv6_logger.info(
                "EDIT:正在修改",
                format!(
                    "[{}/{}] {}: updating {} (ID: {})...",
                    i + 1,
                    groups.len(),
                    tag,
                    name,
                    id
                ),
            );
            ikuai::ipv6_group::edit_ipv6_group(api, tag, &joined, i as i64, id).await
        } else {
            ipv6_logger.info(
                "ADD:正在添加",
                format!(
                    "[{}/{}] {}: adding {}...",
                    i + 1,
                    groups.len(),
                    tag,
                    name
                ),
            );
            ikuai::ipv6_group::add_ipv6_group(api, tag, &joined, i as i64).await
        };

        if let Err(e) = res {
            ipv6_logger.error(
                "UPDATE:更新失败",
                format!(
                    "[{}/{}] {}: failed, error: {}",
                    i + 1,
                    groups.len(),
                    tag,
                    e
                ),
            );
            sleep(cfg.add_err_retry_wait).await;
        }
    }

    if !map.is_empty() {
        let extra: Vec<String> = map.values().map(|v| v.to_string()).collect();
        ipv6_logger.info(
            "CLEAN:冗余删除",
            format!(
                "{}: {} IPv6 groups are no longer needed, deleting IDs: {}",
                tag,
                map.len(),
                extra.join(",")
            ),
        );
        let res = ikuai::ipv6_group::del_ipv6_group(api, &extra.join(",")).await;
        if let Err(e) = res {
            ipv6_logger.error(
                "CLEAN:删除失败",
                format!("{}: failed to delete extra IPv6 groups: {}", tag, e),
            );
        } else {
            ipv6_logger.success(
                "CLEAN:清理成功",
                format!("{}: deleted {} extra IPv6 groups", tag, extra.len()),
            );
        }
    }

    Ok(())
}

async fn update_stream_ipport(
    cfg: &Config,
    api: &ikuai::IKuaiClient,
    sink: &LogSink,
    forward_type: &str,
    tag: &str,
    iface: &str,
    nexthop: &str,
    src_addr: &str,
    src_addr_opt_ipgroup: &str,
    ip_group_name: &str,
    mode: i64,
    ifaceband: i64,
) -> Result<(), UpdateError> {
    let stream_logger = Logger::new("STREAM:端口分流", Arc::clone(sink));

    let mut dst_addr = String::new();
    if ip_group_name.trim().is_empty() {
        stream_logger.info("CHECK:参数校验", "ip-group parameter is empty");
    } else {
        let mut dst_groups = Vec::new();
        for item in ip_group_name.split(',') {
            let item = item.trim();
            if item.is_empty() {
                continue;
            }
            let matches = ikuai::ip_group::get_all_ikuai_bypass_ip_group_names_by_name(api, item).await?;
            dst_groups.extend(matches);
        }
        if dst_groups.is_empty() {
            stream_logger.info(
                "SKIP:跳过操作",
                format!(
                    "No matching destination IP groups found, skipping port streaming rule addition. ip-group: {}",
                    ip_group_name
                ),
            );
            return Ok(());
        }
        dst_addr = dst_groups.join(",");
    }

    let mut src_addr = src_addr.to_string();
    if !src_addr_opt_ipgroup.trim().is_empty() {
        let mut src_groups = Vec::new();
        for item in src_addr_opt_ipgroup.split(',') {
            let item = item.trim();
            if item.is_empty() {
                continue;
            }
            let matches = ikuai::ip_group::get_all_ikuai_bypass_ip_group_names_by_name(api, item).await?;
            src_groups.extend(matches);
        }
        if !src_groups.is_empty() {
            src_addr = src_groups.join(",");
        } else {
            stream_logger.info(
                "SKIP:跳过操作",
                format!(
                    "No matching source IP groups found, skipping port streaming rule addition. srcAddrOptIpGroup: {}",
                    src_addr_opt_ipgroup
                ),
            );
            return Ok(());
        }
    }

    let stream_map = ikuai::stream_ipport::get_stream_ipport_map(api, tag).await?;
    let mut found: Option<(String, i64)> = None;
    for (name, id) in stream_map {
        found = Some((name, id));
        break;
    }

    let res = if let Some((name, id)) = found {
        stream_logger.info(
            "EDIT:正在修改",
            format!(
                "[1/1] {}: updating existing rule {} (ID: {})...",
                tag, name, id
            ),
        );
        ikuai::stream_ipport::edit_stream_ipport(
            api,
            forward_type,
            iface,
            &dst_addr,
            &src_addr,
            nexthop,
            tag,
            mode,
            ifaceband,
            id,
        )
        .await
    } else {
        stream_logger.info(
            "ADD:正在添加",
            format!("[1/1] {}: adding new rule...", tag),
        );
        ikuai::stream_ipport::add_stream_ipport(
            api,
            forward_type,
            iface,
            &dst_addr,
            &src_addr,
            nexthop,
            tag,
            mode,
            ifaceband,
        )
        .await
    };

    if let Err(e) = res {
        stream_logger.error(
            "UPDATE:更新失败",
            format!("[1/1] {}: failed: {}", tag, e),
        );
        sleep(cfg.add_err_retry_wait).await;
    } else {
        stream_logger.success(
            "UPDATE:更新成功",
            format!("[1/1] {}: updated successfully", tag),
        );
    }

    Ok(())
}

async fn http_get(cfg: &Config, sink: &LogSink, original_url: &str) -> Result<Vec<u8>, UpdateError> {
    let (full, proxy) = get_full_url(&cfg.github_proxy, original_url);
    let http_logger = Logger::new("HTTP:资源下载", Arc::clone(sink));
    if proxy.is_empty() {
        http_logger.info("HTTP:资源下载", format!("http.get '{}'", original_url));
    } else {
        http_logger.info(
            "HTTP:资源下载",
            format!("http.get '{}' proxy '{}'", original_url, proxy),
        );
    }

    let resp = reqwest::get(full)
        .await
        .map_err(|e| UpdateError::Download(e.to_string()))?;
    if !resp.status().is_success() {
        return Err(UpdateError::Download(resp.status().to_string()));
    }
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| UpdateError::Download(e.to_string()))?;
    Ok(bytes.to_vec())
}

fn get_full_url(proxy: &str, original: &str) -> (String, String) {
    let proxy = proxy.trim();
    if proxy.is_empty() {
        return (original.to_string(), String::new());
    }
    if !original.starts_with("https://raw.githubusercontent.com/")
        && !original.starts_with("https://github.com/")
    {
        return (original.to_string(), String::new());
    }
    let mut p = proxy.to_string();
    if !p.ends_with('/') {
        p.push('/');
    }
    (format!("{}{}", p, original), proxy.to_string())
}

fn split_lines(body: &[u8]) -> Vec<String> {
    let s = String::from_utf8_lossy(body);
    s.lines().map(|l| l.to_string()).collect()
}

fn remove_ipv6_and_empty(sink: &LogSink, mut lines: Vec<String>) -> Vec<String> {
    Logger::new("IP:IP分组", Arc::clone(sink)).info(
        "IP:v6规则清洗",
        "Removing IPv6 addresses, empty lines and comments...",
    );
    let mut out = Vec::new();
    for mut ip in lines.drain(..) {
        if let Some(idx) = ip.find('#') {
            ip.truncate(idx);
        }
        let ip = ip.trim();
        if ip.is_empty() {
            continue;
        }
        if !ip.contains(':') {
            out.push(ip.to_string());
        }
    }
    out
}

fn remove_ipv4_and_empty(sink: &LogSink, mut lines: Vec<String>) -> Vec<String> {
    Logger::new("IP:IP分组", Arc::clone(sink)).info(
        "IP:v4规则清洗",
        "Removing IPv4 addresses, empty lines and comments...",
    );
    let mut out = Vec::new();
    for mut ip in lines.drain(..) {
        if let Some(idx) = ip.find('#') {
            ip.truncate(idx);
        }
        let ip = ip.trim();
        if ip.is_empty() {
            continue;
        }
        if ip.contains(':') {
            out.push(ip.to_string());
        }
    }
    out
}

fn filter_domains(sink: &LogSink, mut lines: Vec<String>) -> Vec<String> {
    Logger::new("DOMAIN:域名分流", Arc::clone(sink)).info(
        "DOMAIN:域名清洗",
        "Cleaning invalid domains (underscores, comments, etc.)...",
    );
    let mut out = Vec::new();
    for mut d in lines.drain(..) {
        d = d.trim().to_string();
        if d.is_empty() {
            continue;
        }
        if let Some(idx) = d.find('#') {
            d.truncate(idx);
            d = d.trim().to_string();
        }
        if d.is_empty() {
            continue;
        }
        if d.contains('_') {
            Logger::new("DOMAIN:域名分流", Arc::clone(sink)).info(
                "DOMAIN:域名清洗",
                format!("Excluding invalid domain (contains underscore): {}", d),
            );
            continue;
        }
        out.push(d);
    }
    out
}

fn group(arr: Vec<String>, sub_len: usize) -> Vec<Vec<String>> {
    let sub_len = sub_len.max(1);
    let mut out = Vec::new();
    let mut i = 0;
    while i < arr.len() {
        let end = (i + sub_len).min(arr.len());
        out.push(arr[i..end].to_vec());
        i = end;
    }
    out
}

async fn sleep(d: Duration) {
    if d > Duration::from_millis(0) {
        tokio::time::sleep(d).await;
    }
}

#[cfg(test)]
mod tests {
    use super::{filter_domains, group, remove_ipv4_and_empty, remove_ipv6_and_empty, LogSink};
    use std::sync::Arc;

    fn sink() -> LogSink {
        Arc::new(|_s| {})
    }

    #[test]
    fn group_splits() {
        let v = (0..5).map(|i| i.to_string()).collect();
        let g = group(v, 2);
        assert_eq!(g.len(), 3);
        assert_eq!(g[0].len(), 2);
        assert_eq!(g[2].len(), 1);
    }

    #[test]
    fn ipv4_ipv6_filters() {
        let s = sink();
        let ips = vec!["1.1.1.1".into(), "::1".into(), "#c".into(), " 2.2.2.2 #x".into()];
        let v4 = remove_ipv6_and_empty(&s, ips.clone());
        assert_eq!(v4, vec!["1.1.1.1", "2.2.2.2"]);
        let v6 = remove_ipv4_and_empty(&s, ips);
        assert_eq!(v6, vec!["::1"]);
    }

    #[test]
    fn domain_filter_excludes_underscore() {
        let s = sink();
        let d = vec!["a.com".into(), "a_b.com".into(), "b.com #x".into()];
        let out = filter_domains(&s, d);
        assert_eq!(out, vec!["a.com", "b.com"]);
    }
}
