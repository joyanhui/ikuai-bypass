use std::time::Duration;

pub async fn fetch_remote_config(url: &str, github_proxy: &str) -> Result<String, String> {
    let url = url.trim();
    if url.is_empty() {
        return Err("Remote URL is empty".to_string());
    }

    let mut final_url = url.to_string();
    let proxy = github_proxy.trim();
    if !proxy.is_empty()
        && (url.starts_with("https://raw.githubusercontent.com/")
            || url.starts_with("https://github.com/"))
    {
        final_url = if proxy.ends_with('/') {
            format!("{}{}", proxy, url)
        } else {
            format!("{}/{}", proxy, url)
        };
    }

    let client = reqwest::Client::builder()
        .user_agent("ikb-core")
        .timeout(Duration::from_secs(15))
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
