use crate::config::{Config, ProxyConfig, ProxyMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyChoice {
    Direct,
    System,
    Custom,
}

#[derive(Debug, Clone)]
pub struct HttpPlan {
    pub url: String,
    pub proxy: ProxyChoice,
    pub used_github_proxy: bool,
}

/// A lightweight view of network-related settings.
/// 轻量级网络配置视图。
#[derive(Debug, Clone, Copy)]
pub struct NetConfig<'a> {
    pub mode: ProxyMode,
    pub proxy_url: &'a str,
    pub proxy_user: &'a str,
    pub proxy_pass: &'a str,
    pub github_proxy: &'a str,
}

impl<'a> NetConfig<'a> {
    pub fn from_config(cfg: &'a Config) -> Self {
        Self::from_parts(&cfg.proxy, &cfg.github_proxy)
    }

    pub fn from_parts(proxy: &'a ProxyConfig, github_proxy: &'a str) -> Self {
        Self {
            mode: proxy.mode,
            proxy_url: &proxy.url,
            proxy_user: &proxy.user,
            proxy_pass: &proxy.pass,
            github_proxy,
        }
    }
}

pub fn is_github_url_for_ghproxy(url: &str) -> bool {
    let u = url.trim();
    u.starts_with("https://raw.githubusercontent.com/") || u.starts_with("https://github.com/")
}

pub fn normalize_url_prefix(input: &str) -> String {
    let raw = input.trim();
    if raw.is_empty() {
        return String::new();
    }
    if raw.contains("://") {
        return raw.to_string();
    }
    format!("https://{}", raw)
}

fn join_prefix(prefix: &str, url: &str) -> String {
    let mut p = prefix.trim().to_string();
    if !p.ends_with('/') {
        p.push('/');
    }
    format!("{}{}", p, url.trim())
}

/// Plan a request used for downloading remote rule resources (lists/config).
/// 规划用于下载远程规则资源（列表/配置）的请求。
pub fn plan_rule_fetch(net: &NetConfig<'_>, original_url: &str) -> HttpPlan {
    let url = original_url.trim();
    let mode = net.mode;
    let ghproxy = net.github_proxy.trim();

    let can_use_ghproxy =
        matches!(mode, ProxyMode::Smart) && !ghproxy.is_empty() && is_github_url_for_ghproxy(url);
    let used_github_proxy = can_use_ghproxy;
    let final_url = if can_use_ghproxy {
        join_prefix(&normalize_url_prefix(ghproxy), url)
    } else {
        url.to_string()
    };

    let proxy = match mode {
        ProxyMode::Custom => ProxyChoice::Custom,
        ProxyMode::System => ProxyChoice::System,
        ProxyMode::Smart => {
            if used_github_proxy {
                // When using ghproxy, the request must be direct.
                // 使用 ghproxy 时必须直连。
                ProxyChoice::Direct
            } else if net.proxy_url.trim().is_empty() {
                // No custom proxy configured -> fall back to system proxy.
                // 未配置自定义代理 -> 回退到系统代理。
                ProxyChoice::System
            } else {
                ProxyChoice::Custom
            }
        }
    };

    HttpPlan {
        url: final_url,
        proxy,
        used_github_proxy,
    }
}

/// Plan a request used for GitHub API (releases).
/// 规划用于 GitHub API（releases）的请求。
pub fn plan_github_api(net: &NetConfig<'_>, url: &str) -> HttpPlan {
    let url = url.trim();
    let mode = net.mode;

    let proxy = match mode {
        ProxyMode::Custom => ProxyChoice::Custom,
        ProxyMode::System => ProxyChoice::System,
        ProxyMode::Smart => {
            if net.proxy_url.trim().is_empty() {
                ProxyChoice::System
            } else {
                ProxyChoice::Custom
            }
        }
    };

    HttpPlan {
        url: url.to_string(),
        proxy,
        used_github_proxy: false,
    }
}

/// Apply a proxy choice to a reqwest ClientBuilder.
/// 将代理策略应用到 reqwest ClientBuilder。
pub fn apply_proxy_choice(
    mut builder: reqwest::ClientBuilder,
    net: &NetConfig<'_>,
    choice: ProxyChoice,
) -> Result<reqwest::ClientBuilder, reqwest::Error> {
    match choice {
        ProxyChoice::Direct => {
            builder = builder.no_proxy();
        }
        ProxyChoice::System => {
            // Keep reqwest default behavior (honor env proxies).
            // 使用 reqwest 默认行为（读取系统/环境代理变量）。
        }
        ProxyChoice::Custom => {
            let proxy_url = net.proxy_url.trim();
            let proxy_url = if proxy_url.is_empty() {
                // Keep a sensible default for custom mode.
                // custom 模式保留一个合理默认值。
                "http://127.0.0.1:7890"
            } else {
                proxy_url
            };

            let mut p = reqwest::Proxy::all(proxy_url)?;
            let user = net.proxy_user.trim();
            let pass = net.proxy_pass.trim();
            if !user.is_empty() {
                // reqwest Proxy basic_auth does not accept Option.
                // reqwest 的 Proxy basic_auth 不支持 Option。
                p = p.basic_auth(user, pass);
            }

            // Disable system proxies when using a custom proxy.
            // 使用自定义代理时禁用系统代理。
            builder = builder.no_proxy();
            builder = builder.proxy(p);
        }
    }
    Ok(builder)
}
