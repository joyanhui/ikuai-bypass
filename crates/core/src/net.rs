use crate::config::{ProxyConfig, ProxyMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyTarget {
    General,
    GithubApi,
}

/// Apply global proxy settings to a reqwest ClientBuilder.
/// 将全局代理设置应用到 reqwest ClientBuilder。
pub fn apply_proxy(
    builder: reqwest::ClientBuilder,
    proxy: &ProxyConfig,
) -> Result<reqwest::ClientBuilder, reqwest::Error> {
    apply_proxy_for(builder, proxy, ProxyTarget::General)
}

/// Apply proxy settings for a specific target.
/// 对特定用途应用代理设置。
pub fn apply_proxy_for(
    mut builder: reqwest::ClientBuilder,
    proxy: &ProxyConfig,
    target: ProxyTarget,
) -> Result<reqwest::ClientBuilder, reqwest::Error> {
    match proxy.mode {
        ProxyMode::Disabled => {
            // Disable all proxies and ignore environment variables.
            // 禁用所有代理并忽略环境变量。
            builder = builder.no_proxy();
        }
        ProxyMode::System => {
            // Keep reqwest defaults (honor environment proxy variables).
            // 使用 reqwest 默认行为（读取系统/环境代理变量）。
        }
        ProxyMode::Custom => {
            // Prefer explicitly configured proxy.
            // 优先使用显式配置的代理。
            let url = proxy.url.trim();
            if !url.is_empty() {
                // Try to avoid environment proxy interference.
                // 尽量避免环境变量代理干扰。
                builder = builder.no_proxy();
                builder = builder.proxy(reqwest::Proxy::all(url)?);
            }
        }
        ProxyMode::OnlyGithubApi => {
            match target {
                ProxyTarget::GithubApi => {
                    // Only use proxy for GitHub API.
                    // - If url is set: use the configured proxy URL.
                    // - Otherwise: fall back to system/env proxy.
                    // 仅对 GitHub API 使用代理：
                    // - 若 url 非空：使用配置的代理地址
                    // - 否则：回退到系统/环境代理
                    let url = proxy.url.trim();
                    if !url.is_empty() {
                        builder = builder.no_proxy();
                        builder = builder.proxy(reqwest::Proxy::all(url)?);
                    }
                }
                ProxyTarget::General => {
                    // Everything else goes direct.
                    // 其他请求一律直连。
                    builder = builder.no_proxy();
                }
            }
        }
    }

    Ok(builder)
}
