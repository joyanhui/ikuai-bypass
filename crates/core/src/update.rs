use std::sync::Arc;
use std::time::Duration;

use crate::config::Config;
use crate::ikuai;
use crate::logger::{LogSink, Logger};
use crate::session::{LoginParamsError, resolve_login_params};

#[derive(Debug, Clone, Default)]
pub struct UpdateOptions {
    /// Export directory for stream-domain rule lists (best-effort).
    /// 域名分流规则列表导出目录（尽力而为，不影响更新流程）。
    pub export_path: String,

    /// Whether to add a deterministic random-like suffix for ip-group/ipv6-group names.
    /// IP 分组名称是否增加“随机后缀”（确定性 hash token，用于降低截断碰撞概率）。
    pub ip_group_name_add_random_suffix: bool,
}

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

struct StreamDomainUpdate<'a> {
    iface: &'a str,
    tag: &'a str,
    src_addr_ipgroup: &'a str,
    src_addr: &'a str,
    url: &'a str,
}

struct StreamIpPortUpdate<'a> {
    forward_type: &'a str,
    tag: &'a str,
    iface: &'a str,
    nexthop: &'a str,
    src_addr: &'a str,
    src_addr_opt_ipgroup: &'a str,
    src_addr_inv: i64,
    ip_group_name: &'a str,
    dst_addr_inv: i64,
    mode: i64,
    ifaceband: i64,
}

pub async fn run_update_by_module(
    cfg: &Config,
    cli_login: &str,
    module: &str,
    opts: &UpdateOptions,
    sink: LogSink,
) -> Result<(), UpdateError> {
    let params = resolve_login_params(cfg, cli_login)?;
    let api = ikuai::IKuaiClient::new(params.base_url.to_string())?;

    let auth = Logger::new("AUTH:登录认证", Arc::clone(&sink));
    auth.info(
        "LOGIN:开始登录",
        format!("Logging in to iKuai: {}", params.base_url),
    );
    api.login(&params.username, &params.password).await?;
    auth.success("LOGIN:登录成功", "Login succeeded");

    let sys = Logger::new("SYS:系统组件", Arc::clone(&sink));
    match module {
        "ispdomain" => {
            sys.info("TASK:任务启动", "Starting ISP and Domain streaming mode");
            update_ispdomain(cfg, &api, opts, &sink).await;
        }
        "ipgroup" => {
            sys.info(
                "TASK:任务启动",
                "Starting IP group and Next-hop gateway mode",
            );
            update_ipgroup(cfg, &api, opts, &sink).await;
        }
        "ipv6group" => {
            sys.info("TASK:任务启动", "Starting IPv6 group mode");
            update_ipv6group(cfg, &api, opts, &sink).await;
        }
        "ii" => {
            sys.info(
                "TASK:任务启动",
                "Starting hybrid mode: ISP/Domain + IP group",
            );
            // stream-domain / stream-ipport can depend on freshly synced IP groups,
            // so combined modes must materialize IP groups before domain rules.
            // stream-domain / stream-ipport 可能依赖本轮刚同步出的 IP 分组，
            // 因此组合模式需要先落地 IP 分组，再处理域名类规则。
            update_ipgroup(cfg, &api, opts, &sink).await;
            update_ispdomain(cfg, &api, opts, &sink).await;
        }
        "ip" => {
            sys.info(
                "TASK:任务启动",
                "Starting hybrid mode: IPv4 group + IPv6 group",
            );
            update_ipgroup(cfg, &api, opts, &sink).await;
            update_ipv6group(cfg, &api, opts, &sink).await;
        }
        "iip" => {
            sys.info(
                "TASK:任务启动",
                "Starting full hybrid mode: ISP/Domain + IPv4/v6 group",
            );
            update_ipgroup(cfg, &api, opts, &sink).await;
            update_ispdomain(cfg, &api, opts, &sink).await;
            update_ipv6group(cfg, &api, opts, &sink).await;
        }
        other => return Err(UpdateError::InvalidModule(other.to_string())),
    }
    Ok(())
}

/// Export stream-domain rule lists into plain TXT files.
/// 将 stream-domain 规则列表导出为纯文本 TXT（便于调试/人工导入）。
pub async fn export_stream_domain_to_txt(
    cfg: &Config,
    export_path: &str,
    sink: LogSink,
) -> Result<(), UpdateError> {
    let export_path = export_path.trim();
    if export_path.is_empty() {
        return Err(UpdateError::Download("exportPath is empty".to_string()));
    }

    let domain = Logger::new("DOMAIN:域名分流", Arc::clone(&sink));
    domain.info(
        "EXPORT:开始导出",
        format!(
            "exportPath='{}' items={} (stream-domain)",
            export_path,
            cfg.stream_domain.len()
        ),
    );

    if cfg.stream_domain.is_empty() {
        domain.warn("EXPORT:无可导出项", "stream-domain is empty".to_string());
        return Ok(());
    }

    let mut failed = 0usize;
    for item in &cfg.stream_domain {
        let iface = item.interface.trim();
        let tag = item.tag.trim();
        let url = item.url.trim();
        if url.is_empty() {
            failed += 1;
            domain.error(
                "EXPORT:导出失败",
                format!("interface='{}' tag='{}' error=empty_url", iface, tag),
            );
            continue;
        }

        domain.info(
            "EXPORT:开始导出",
            format!("interface='{}' tag='{}' url='{}'", iface, tag, url),
        );
        let body = match http_get(cfg, &sink, url).await {
            Ok(v) => v,
            Err(e) => {
                failed += 1;
                domain.error(
                    "EXPORT:导出失败",
                    format!("interface='{}' tag='{}' error={}", iface, tag, e),
                );
                continue;
            }
        };

        let domains = filter_domains(split_lines(&body));
        match export_stream_domains(export_path, iface, tag, &domains) {
            Ok(p) => {
                domain.success(
                    "EXPORT:导出成功",
                    format!("domains={} path='{}'", domains.len(), p.to_string_lossy()),
                );
            }
            Err(e) => {
                failed += 1;
                domain.error(
                    "EXPORT:导出失败",
                    format!("interface='{}' tag='{}' error={}", iface, tag, e),
                );
            }
        }
    }

    if failed > 0 {
        domain.error("EXPORT:导出完成", format!("failed={}", failed));
        return Err(UpdateError::Download(format!(
            "export finished with {} failures",
            failed
        )));
    }

    domain.success("EXPORT:导出完成", "OK".to_string());
    Ok(())
}

async fn update_ispdomain(
    cfg: &Config,
    api: &ikuai::IKuaiClient,
    opts: &UpdateOptions,
    sink: &LogSink,
) {
    let isp = Logger::new("ISP:运营商分流", Arc::clone(sink));
    let domain = Logger::new("DOMAIN:域名分流", Arc::clone(sink));
    let sys = Logger::new("SYS:系统组件", Arc::clone(sink));

    for item in &cfg.custom_isp {
        isp.info("UPDATE:开始更新", format!("Updating {}...", item.tag));
        let res = update_custom_isp(cfg, api, sink, &item.tag, &item.url).await;
        if let Err(e) = res {
            isp.error(
                "UPDATE:更新失败",
                format!("Failed to update custom ISP '{}': {}", item.tag, e),
            );
        } else {
            isp.success(
                "UPDATE:更新成功",
                format!("Successfully updated custom ISP '{}'", item.tag),
            );
        }
    }

    for item in &cfg.stream_domain {
        domain.info(
            "UPDATE:开始更新",
            format!(
                "Updating {} (Interface: {}, Tag: {})...",
                item.url, item.interface, item.tag
            ),
        );
        let input = StreamDomainUpdate {
            iface: &item.interface,
            tag: &item.tag,
            src_addr_ipgroup: &item.src_addr_opt_ipgroup,
            src_addr: &item.src_addr,
            url: &item.url,
        };
        let res = update_stream_domain(cfg, api, opts, sink, input).await;
        if let Err(e) = res {
            domain.error(
                "UPDATE:更新失败",
                format!(
                    "Failed to update domain streaming for tag {}: {}",
                    item.tag, e
                ),
            );
        } else {
            domain.success(
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

async fn update_ipgroup(
    cfg: &Config,
    api: &ikuai::IKuaiClient,
    opts: &UpdateOptions,
    sink: &LogSink,
) {
    let ip_logger = Logger::new("IP:IP分组", Arc::clone(sink));
    let stream_logger = Logger::new("STREAM:端口分流", Arc::clone(sink));

    for item in &cfg.ip_group {
        let res = update_ip_group(cfg, api, opts, sink, &item.tag, &item.url).await;
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
            item.opt_tagname.to_string()
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
        let input = StreamIpPortUpdate {
            forward_type: &item.r#type,
            tag: &tag,
            iface: &item.interface,
            nexthop: &item.nexthop,
            src_addr: &item.src_addr,
            src_addr_opt_ipgroup: &item.src_addr_opt_ipgroup,
            src_addr_inv: item.src_addr_inv,
            ip_group_name: &item.ip_group,
            dst_addr_inv: item.dst_addr_inv,
            mode: item.mode,
            ifaceband: item.ifaceband,
        };
        let res = update_stream_ipport(cfg, api, sink, input).await;
        if let Err(e) = res {
            let route_name = format!("{}{}", item.interface, item.nexthop);
            stream_logger.error(
                "UPDATE:更新失败",
                format!(
                    "Failed to update port streaming '{}@{}': {}",
                    route_name, item.ip_group, e
                ),
            );
        } else {
            let route_name = format!("{}{}", item.interface, item.nexthop);
            stream_logger.success(
                "UPDATE:更新成功",
                format!(
                    "Successfully updated port streaming '{}@{}'",
                    route_name, item.ip_group
                ),
            );
        }
    }
}

async fn update_ipv6group(
    cfg: &Config,
    api: &ikuai::IKuaiClient,
    opts: &UpdateOptions,
    sink: &LogSink,
) {
    let ipv6_logger = Logger::new("IPV6:IPv6分组", Arc::clone(sink));
    for item in &cfg.ipv6_group {
        let res = update_ipv6_group(cfg, api, opts, sink, &item.tag, &item.url).await;
        if let Err(e) = res {
            ipv6_logger.error(
                "UPDATE:更新失败",
                format!(
                    "Failed to add IPv6 group '{}@{}': {}",
                    item.tag, item.url, e
                ),
            );
        } else {
            ipv6_logger.success(
                "UPDATE:更新成功",
                format!(
                    "Successfully updated IPv6 group '{}@{}'",
                    item.tag, item.url
                ),
            );
        }
    }
}

async fn http_get(
    cfg: &Config,
    sink: &LogSink,
    original_url: &str,
) -> Result<Vec<u8>, UpdateError> {
    let net = crate::net::NetConfig::from_config(cfg);
    let plan = crate::net::plan_rule_fetch(&net, original_url);
    let http_logger = Logger::new("HTTP:资源下载", Arc::clone(sink));
    let via = match plan.proxy {
        crate::net::ProxyChoice::Direct => "直连",
        crate::net::ProxyChoice::System => "系统代理",
        crate::net::ProxyChoice::Custom => "自定义代理",
    };
    let gh = if plan.used_github_proxy {
        " (ghproxy)"
    } else {
        ""
    };
    http_logger.info(
        "HTTP:资源下载",
        format!("http.get '{}' via={}{}", original_url, via, gh),
    );

    // 避免远程资源不可达时无限期等待。
    // Avoid hanging forever on remote resources.
    let builder = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(120));
    let client = crate::net::apply_proxy_choice(builder, &net, plan.proxy)
        .map_err(|e| UpdateError::Download(e.to_string()))?
        .build()
        .map_err(|e| UpdateError::Download(e.to_string()))?;

    let resp = client
        .get(plan.url)
        .send()
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

fn split_lines(body: &[u8]) -> Vec<String> {
    let s = String::from_utf8_lossy(body);
    s.lines().map(|l| l.to_string()).collect()
}

fn remove_ipv6_and_empty(lines: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for mut ip in lines {
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

fn remove_ipv4_and_empty(lines: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for mut ip in lines {
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

fn filter_domains(lines: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for mut d in lines {
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
            continue;
        }
        out.push(d);
    }
    out
}

fn export_stream_domains(
    export_path: &str,
    iface: &str,
    tag: &str,
    domains: &[String],
) -> Result<std::path::PathBuf, std::io::Error> {
    let dir = std::path::PathBuf::from(export_path.trim());
    if dir.as_os_str().is_empty() {
        return Ok(dir);
    }
    std::fs::create_dir_all(&dir)?;

    let iface_token = ikuai::tag_name::sanitize_tag_name(iface);
    let tag_token = ikuai::tag_name::sanitize_tag_name(tag);
    let iface_token = if iface_token.is_empty() {
        "iface"
    } else {
        &iface_token
    };
    let tag_token = if tag_token.is_empty() {
        "tag"
    } else {
        &tag_token
    };

    let filename = format!("stream-domain_{}_{}.txt", iface_token, tag_token);
    let path = dir.join(filename);
    let mut content = String::new();
    for d in domains {
        content.push_str(d);
        content.push('\n');
    }
    std::fs::write(&path, content.as_bytes())?;
    Ok(path)
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

async fn update_custom_isp(
    cfg: &Config,
    api: &ikuai::IKuaiClient,
    sink: &LogSink,
    tag: &str,
    url: &str,
) -> Result<(), UpdateError> {
    let isp_logger = Logger::new("ISP:运营商分流", Arc::clone(sink));
    let body = http_get(cfg, sink, url).await?;
    let ips = remove_ipv6_and_empty(split_lines(&body));
    isp_logger.info(
        "STAT:规则统计",
        format!("Fetched {} IPs for {}", ips.len(), tag),
    );

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
                format!("[{}/{}] {}: failed: {}", i + 1, groups.len(), tag, e),
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
        let res = ikuai::custom_isp::del_custom_isp(api, &extra.join(",")).await;
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
    opts: &UpdateOptions,
    sink: &LogSink,
    input: StreamDomainUpdate<'_>,
) -> Result<(), UpdateError> {
    let domain_logger = Logger::new("DOMAIN:域名分流", Arc::clone(sink));
    let body = http_get(cfg, sink, input.url).await?;
    let domains = filter_domains(split_lines(&body));
    domain_logger.success(
        "PARSE:解析成功",
        format!(
            "{} {}: obtained {} valid domains",
            input.iface,
            input.tag,
            domains.len()
        ),
    );

    // Best-effort export for debugging/manual inspection.
    // 规则导出（尽力而为，用于调试/人工检查，不影响更新流程）。
    if !opts.export_path.trim().is_empty() {
        match export_stream_domains(&opts.export_path, input.iface, input.tag, &domains) {
            Ok(p) => {
                if !p.as_os_str().is_empty() {
                    domain_logger
                        .info("EXPORT:导出成功", format!("path='{}'", p.to_string_lossy()));
                }
            }
            Err(e) => {
                domain_logger.warn(
                    "EXPORT:导出失败",
                    format!("exportPath='{}' error={}", opts.export_path, e),
                );
            }
        }
    }

    let mut map = ikuai::stream_domain::get_stream_domain_map(api, input.tag).await?;
    let groups = group(domains, cfg.max_number_of_one_records.domain as usize);
    for (i, chunk) in groups.iter().enumerate() {
        let index = (i + 1) as i64;
        let name = ikuai::tag_name::build_indexed_tag_name(input.tag, i as i64);
        let joined = chunk.join(",");
        let spec = ikuai::stream_domain::StreamDomainSpec {
            iface: input.iface,
            tag: input.tag,
            src_addr: input.src_addr,
            src_addr_opt_ipgroup: input.src_addr_ipgroup,
            domains: &joined,
            index: i as i64,
        };
        let res = if let Some(id) = map.remove(&index) {
            domain_logger.info(
                "EDIT:正在修改",
                format!(
                    "[{}/{}] {} {}: updating {} (ID: {})...",
                    i + 1,
                    groups.len(),
                    input.iface,
                    input.tag,
                    name,
                    id
                ),
            );
            ikuai::stream_domain::edit_stream_domain(api, spec, id).await
        } else {
            domain_logger.info(
                "ADD:正在添加",
                format!(
                    "[{}/{}] {} {}: adding {}...",
                    i + 1,
                    groups.len(),
                    input.iface,
                    input.tag,
                    name
                ),
            );
            ikuai::stream_domain::add_stream_domain(api, spec).await
        };

        if let Err(e) = res {
            domain_logger.error(
                "UPDATE:更新失败",
                format!(
                    "[{}/{}] {} {}: failed: {}",
                    i + 1,
                    groups.len(),
                    input.iface,
                    input.tag,
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
                    input.tag, idx, id
                ),
            );
            extra.push(id.to_string());
        }
        let res = ikuai::stream_domain::del_stream_domain(api, &extra.join(",")).await;
        if let Err(e) = res {
            domain_logger.error(
                "CLEAN:删除失败",
                format!("{}: failed to delete extra domain rules: {}", input.tag, e),
            );
        } else {
            domain_logger.success(
                "CLEAN:清理成功",
                format!("{}: deleted {} extra domain rules", input.tag, extra.len()),
            );
        }
    }

    Ok(())
}

async fn update_ip_group(
    cfg: &Config,
    api: &ikuai::IKuaiClient,
    opts: &UpdateOptions,
    sink: &LogSink,
    tag: &str,
    url: &str,
) -> Result<(), UpdateError> {
    let ip_logger = Logger::new("IP:IP分组", Arc::clone(sink));
    let body = http_get(cfg, sink, url).await?;
    let ips = remove_ipv6_and_empty(split_lines(&body));
    let groups = group(ips, cfg.max_number_of_one_records.ipv4 as usize);
    ip_logger.success("PARSE:解析成功", format!("{}: obtained new data", tag));

    let mut map = ikuai::ip_group::get_ip_group_map_with_name(api, tag).await?;
    ip_logger.info(
        "QUERY:查询成功",
        format!("{}: found {} existing groups", tag, map.len()),
    );

    for (i, chunk) in groups.iter().enumerate() {
        let index = (i + 1) as i64;
        let name = if opts.ip_group_name_add_random_suffix {
            ikuai::tag_name::build_indexed_ip_group_tag_name(tag, i as i64)
        } else {
            ikuai::tag_name::build_indexed_tag_name(tag, i as i64)
        };
        let joined = chunk.join(",");
        let res = if let Some((id, existing_name)) = map.remove(&index) {
            ip_logger.info(
                "EDIT:正在修改",
                format!(
                    "[{}/{}] {}: updating {} (ID: {})...",
                    i + 1,
                    groups.len(),
                    tag,
                    existing_name,
                    id
                ),
            );
            ikuai::ip_group::edit_ip_group_named(api, &existing_name, &joined, id).await
        } else {
            ip_logger.info(
                "ADD:正在添加",
                format!("[{}/{}] {}: adding {}...", i + 1, groups.len(), tag, name),
            );
            ikuai::ip_group::add_ip_group_named(api, &name, &joined).await
        };
        if let Err(e) = res {
            ip_logger.error(
                "UPDATE:更新失败",
                format!("[{}/{}] {}: failed, error: {}", i + 1, groups.len(), tag, e),
            );
            sleep(cfg.add_err_retry_wait).await;
        }
    }

    if !map.is_empty() {
        let extra: Vec<String> = map.values().map(|(id, _)| id.to_string()).collect();
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
    opts: &UpdateOptions,
    sink: &LogSink,
    tag: &str,
    url: &str,
) -> Result<(), UpdateError> {
    let ipv6_logger = Logger::new("IPV6:IPv6分组", Arc::clone(sink));
    let body = http_get(cfg, sink, url).await?;
    let ips = remove_ipv4_and_empty(split_lines(&body));
    let groups = group(ips, cfg.max_number_of_one_records.ipv6 as usize);
    ipv6_logger.success("PARSE:解析成功", format!("{}: obtained new data", tag));

    let mut map = ikuai::ipv6_group::get_ipv6_group_map_with_name(api, tag).await?;
    ipv6_logger.info(
        "QUERY:查询成功",
        format!("{}: found {} existing IPv6 groups", tag, map.len()),
    );

    for (i, chunk) in groups.iter().enumerate() {
        let index = (i + 1) as i64;
        let name = if opts.ip_group_name_add_random_suffix {
            ikuai::tag_name::build_indexed_ip_group_tag_name(tag, i as i64)
        } else {
            ikuai::tag_name::build_indexed_tag_name(tag, i as i64)
        };
        let joined = chunk.join(",");
        let res = if let Some((id, existing_name)) = map.remove(&index) {
            ipv6_logger.info(
                "EDIT:正在修改",
                format!(
                    "[{}/{}] {}: updating {} (ID: {})...",
                    i + 1,
                    groups.len(),
                    tag,
                    existing_name,
                    id
                ),
            );
            ikuai::ipv6_group::edit_ipv6_group_named(api, &existing_name, &joined, id).await
        } else {
            ipv6_logger.info(
                "ADD:正在添加",
                format!("[{}/{}] {}: adding {}...", i + 1, groups.len(), tag, name),
            );
            ikuai::ipv6_group::add_ipv6_group_named(api, &name, &joined).await
        };
        if let Err(e) = res {
            ipv6_logger.error(
                "UPDATE:更新失败",
                format!("[{}/{}] {}: failed, error: {}", i + 1, groups.len(), tag, e),
            );
            sleep(cfg.add_err_retry_wait).await;
        }
    }

    if !map.is_empty() {
        let extra: Vec<String> = map.values().map(|(id, _)| id.to_string()).collect();
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
    input: StreamIpPortUpdate<'_>,
) -> Result<(), UpdateError> {
    let stream_logger = Logger::new("STREAM:端口分流", Arc::clone(sink));

    let mut dst_addr = String::new();
    if input.ip_group_name.trim().is_empty() {
        stream_logger.info("CHECK:参数校验", "ip-group parameter is empty");
    } else {
        let mut dst_groups = Vec::new();
        for item in input.ip_group_name.split(',') {
            let item = item.trim();
            if item.is_empty() {
                continue;
            }
            let matches = ikuai::ip_group::resolve_rule_reference_ip_group_names(api, item).await?;
            dst_groups.extend(matches);
        }
        if dst_groups.is_empty() {
            stream_logger.info(
                "SKIP:跳过操作",
                format!(
                    "No matching destination IP groups found, skipping port streaming rule addition. ip-group: {}",
                    input.ip_group_name
                ),
            );
            return Ok(());
        }
        dst_addr = dst_groups.join(",");
    }

    let mut src_addr = input.src_addr.to_string();
    if !input.src_addr_opt_ipgroup.trim().is_empty() {
        let mut src_groups = Vec::new();
        for item in input.src_addr_opt_ipgroup.split(',') {
            let item = item.trim();
            if item.is_empty() {
                continue;
            }
            let matches = ikuai::ip_group::resolve_rule_reference_ip_group_names(api, item).await?;
            src_groups.extend(matches);
        }
        if !src_groups.is_empty() {
            src_addr = src_groups.join(",");
        } else {
            stream_logger.info(
                "SKIP:跳过操作",
                format!(
                    "No matching source IP groups found, skipping port streaming rule addition. srcAddrOptIpGroup: {}",
                    input.src_addr_opt_ipgroup
                ),
            );
            return Ok(());
        }
    }

    let stream_map = ikuai::stream_ipport::get_stream_ipport_map(api, input.tag).await?;
    let found = stream_map.into_iter().next();
    let spec = ikuai::stream_ipport::StreamIpPortSpec {
        forward_type: input.forward_type,
        iface: input.iface,
        dst_addr: &dst_addr,
        src_addr: &src_addr,
        src_addr_inv: input.src_addr_inv,
        nexthop: input.nexthop,
        tag: input.tag,
        dst_addr_inv: input.dst_addr_inv,
        mode: input.mode,
        iface_band: input.ifaceband,
    };

    let res = if let Some((name, id)) = found {
        stream_logger.info(
            "EDIT:正在修改",
            format!(
                "[1/1] {}: updating existing rule {} (ID: {})...",
                input.tag, name, id
            ),
        );
        ikuai::stream_ipport::edit_stream_ipport(api, spec, id).await
    } else {
        stream_logger.info(
            "ADD:正在添加",
            format!("[1/1] {}: adding new rule...", input.tag),
        );
        ikuai::stream_ipport::add_stream_ipport(api, spec).await
    };

    if let Err(e) = res {
        stream_logger.error(
            "UPDATE:更新失败",
            format!("[1/1] {}: failed: {}", input.tag, e),
        );
        sleep(cfg.add_err_retry_wait).await;
        return Err(e.into());
    }

    stream_logger.success(
        "UPDATE:更新成功",
        format!("[1/1] {}: updated successfully", input.tag),
    );

    Ok(())
}
