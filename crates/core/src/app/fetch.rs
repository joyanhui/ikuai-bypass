use std::time::Duration;

use crate::config::ProxyConfig;

pub async fn fetch_remote_config(
    url: &str,
    proxy: &ProxyConfig,
    github_proxy: &str,
) -> Result<String, String> {
    let url = url.trim();
    if url.is_empty() {
        return Err("Remote URL is empty".to_string());
    }

    let net = crate::net::NetConfig::from_parts(proxy, github_proxy);
    let plan = crate::net::plan_rule_fetch(&net, url);

    let builder = reqwest::Client::builder()
        .user_agent("ikb-core")
        .timeout(Duration::from_secs(15))
        ;
    let client = crate::net::apply_proxy_choice(builder, &net, plan.proxy)
        .map_err(|e| e.to_string())?
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get(plan.url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("HTTP {}", status));
    }
    resp.text().await.map_err(|e| e.to_string())
}
