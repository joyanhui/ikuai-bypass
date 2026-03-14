use std::time::Duration;

use crate::config::ProxyConfig;

pub async fn fetch_remote_config(
    url: &str,
    github_proxy: &str,
    proxy_cfg: &ProxyConfig,
) -> Result<String, String> {
    let url = url.trim();
    if url.is_empty() {
        return Err("Remote URL is empty".to_string());
    }

    let mut final_url = url.to_string();
    let ghproxy = github_proxy.trim();
    if !ghproxy.is_empty()
        && (url.starts_with("https://raw.githubusercontent.com/")
            || url.starts_with("https://github.com/"))
    {
        final_url = if ghproxy.ends_with('/') {
            format!("{}{}", ghproxy, url)
        } else {
            format!("{}/{}", ghproxy, url)
        };
    }

    let builder = reqwest::Client::builder()
        .user_agent("ikb-core")
        .timeout(Duration::from_secs(15))
        ;
    let client = crate::net::apply_proxy(builder, proxy_cfg)
        .map_err(|e| e.to_string())?
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get(final_url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("HTTP {}", status));
    }
    resp.text().await.map_err(|e| e.to_string())
}
