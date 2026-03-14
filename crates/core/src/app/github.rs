use std::time::Duration;

use crate::config::ProxyConfig;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GithubRelease {
    pub tag_name: String,
    #[serde(default)]
    pub name: Option<String>,
    pub prerelease: bool,
    pub draft: bool,
    pub html_url: String,
    #[serde(default)]
    pub published_at: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

pub async fn fetch_github_releases(proxy: &ProxyConfig) -> Result<Vec<GithubRelease>, String> {
    const URL: &str =
        "https://api.github.com/repos/joyanhui/ikuai-bypass/releases?per_page=30";

    let builder = reqwest::Client::builder()
        .user_agent("ikb-core")
        .connect_timeout(Duration::from_secs(8))
        .timeout(Duration::from_secs(15));

    let client = crate::net::apply_proxy_for(builder, proxy, crate::net::ProxyTarget::GithubApi)
        .map_err(|e| format!("Failed to apply proxy: {}", e))?
        .build()
        .map_err(|e| format!("Failed to build http client: {}", e))?;

    let resp = client
        .get(URL)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        let trimmed = body.trim();
        let mut hint = String::new();
        if !trimmed.is_empty() {
            let mut out = trimmed.chars().take(200).collect::<String>();
            if trimmed.chars().count() > 200 {
                out.push_str("...");
            }
            hint = format!(" body='{}'", out.replace('\n', " "));
        }
        return Err(format!("HTTP {} url='{}'{}", status, URL, hint));
    }

    resp.json::<Vec<GithubRelease>>()
        .await
        .map_err(|e| format!("Failed to decode response: {}", e))
}
