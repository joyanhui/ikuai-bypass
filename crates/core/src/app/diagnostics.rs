use std::time::Duration;

use crate::ikuai::IKuaiClient;
use crate::config::ProxyConfig;

use super::{normalize_base_url, normalize_url_prefix};

#[derive(Debug, Clone, serde::Serialize)]
pub struct TestResult {
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TestIkuaiLoginRequest {
    #[serde(alias = "baseUrl")]
    pub base_url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TestGithubProxyRequest {
    #[serde(alias = "githubProxy")]
    pub github_proxy: String,
}

pub async fn test_ikuai_login(req: TestIkuaiLoginRequest, proxy_cfg: &ProxyConfig) -> TestResult {
    let base_url = normalize_base_url(&req.base_url);
    let username = req.username.trim().to_string();
    if base_url.is_empty() {
        return TestResult {
            ok: false,
            message: "Empty iKuai URL".to_string(),
        };
    }
    if username.is_empty() {
        return TestResult {
            ok: false,
            message: "Empty username".to_string(),
        };
    }

    let api = match IKuaiClient::new(base_url, proxy_cfg) {
        Ok(v) => v,
        Err(e) => {
            return TestResult {
                ok: false,
                message: e.to_string(),
            }
        }
    };

    match api.login(&username, &req.password).await {
        Ok(()) => TestResult {
            ok: true,
            message: "OK".to_string(),
        },
        Err(e) => TestResult {
            ok: false,
            message: e.to_string(),
        },
    }
}

pub async fn test_github_proxy(req: TestGithubProxyRequest, proxy_cfg: &ProxyConfig) -> TestResult {
    const URL: &str = "https://raw.githubusercontent.com/joyanhui/ikuai-bypass/refs/heads/main/.gitignore";

    let ghproxy = normalize_url_prefix(&req.github_proxy);
    if ghproxy.is_empty() {
        return TestResult {
            ok: false,
            message: "Empty github proxy".to_string(),
        };
    }

    let final_url = if ghproxy.ends_with('/') {
        format!("{}{}", ghproxy, URL)
    } else {
        format!("{}/{}", ghproxy, URL)
    };

    let builder = reqwest::Client::builder()
        // Some ghproxy sites may restrict uncommon user agents.
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(Duration::from_secs(15))
        ;
    let client = match crate::net::apply_proxy(builder, proxy_cfg).and_then(|b| b.build()) {
        Ok(v) => v,
        Err(e) => {
            return TestResult {
                ok: false,
                message: e.to_string(),
            }
        }
    };

    let resp = match client.get(&final_url).send().await {
        Ok(v) => v,
        Err(e) => {
            return TestResult {
                ok: false,
                message: e.to_string(),
            }
        }
    };
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        let trimmed = body.trim();
        let hint = if trimmed.is_empty() {
            String::new()
        } else {
            let mut out = trimmed.chars().take(200).collect::<String>();
            if trimmed.chars().count() > 200 {
                out.push_str("...");
            }
            format!(" body='{}'", out.replace('\n', " "))
        };
        return TestResult {
            ok: false,
            message: format!("HTTP {} url='{}'{}", status, final_url, hint),
        };
    }

    let text = match resp.text().await {
        Ok(v) => v,
        Err(e) => {
            return TestResult {
                ok: false,
                message: e.to_string(),
            }
        }
    };
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return TestResult {
            ok: false,
            message: format!("Empty response url='{}'", final_url),
        };
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.contains("<html") || lower.contains("<!doctype") {
        let mut out = trimmed.chars().take(200).collect::<String>();
        if trimmed.chars().count() > 200 {
            out.push_str("...");
        }
        return TestResult {
            ok: false,
            message: format!(
                "Unexpected HTML url='{}' body='{}'",
                final_url,
                out.replace('\n', " ")
            ),
        };
    }
    TestResult {
        ok: true,
        message: format!("OK url='{}'", final_url),
    }
}
