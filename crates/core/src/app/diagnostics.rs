use std::time::Duration;
use std::path::Path;

use crate::ikuai::IKuaiClient;
use crate::{config::Config, net::ProxyChoice};
use chrono::Local;
use cron::Schedule;
use std::str::FromStr;

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

pub async fn test_ikuai_login(req: TestIkuaiLoginRequest) -> TestResult {
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

    let api = match IKuaiClient::new(base_url) {
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

pub async fn test_github_proxy(req: TestGithubProxyRequest) -> TestResult {
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
    // ghproxy 测试必须直连，避免被其他代理干扰。
    // ghproxy test should be direct to avoid interference.
    let client = match builder.no_proxy().build() {
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

#[derive(Debug, Clone, serde::Serialize)]
pub struct DiagnosticsReport {
    pub generated_at: String,
    pub text: String,
}

fn mask_secret(s: &str) -> String {
    let s = s.trim();
    if s.is_empty() {
        return "(empty)".to_string();
    }
    // Keep length only; never return raw secrets.
    // 仅保留长度信息；不要回传明文。
    format!("(set,len={})", s.chars().count())
}

fn normalize_cron_expr_for_report(expr: &str) -> Result<String, String> {
    let raw = expr.trim();
    if raw.is_empty() {
        return Err("cron expression is empty".to_string());
    }
    let parts: Vec<&str> = raw.split_whitespace().collect();
    let mut candidates = Vec::new();
    candidates.push(raw.to_string());
    if parts.len() == 5 {
        candidates.push(format!("0 {}", raw));
        candidates.push(format!("0 {} *", raw));
    }
    if parts.len() == 6 {
        candidates.push(format!("{} *", raw));
    }
    for c in candidates {
        if Schedule::from_str(&c).is_ok() {
            return Ok(c);
        }
    }
    Err("Invalid cron expression".to_string())
}

#[derive(Debug, Clone)]
struct UrlProbe {
    label: String,
    url: String,
}

#[derive(Debug, Clone)]
struct UrlProbeResult {
    ok: bool,
    status: String,
    via: String,
    used_github_proxy: bool,
    bytes: usize,
    error: String,
}

async fn probe_rule_url(cfg: &Config, original_url: &str) -> UrlProbeResult {
    let net = crate::net::NetConfig::from_config(cfg);
    let plan = crate::net::plan_rule_fetch(&net, original_url);
    let via = match plan.proxy {
        ProxyChoice::Direct => "direct",
        ProxyChoice::System => "system",
        ProxyChoice::Custom => "custom",
    };

    let builder = reqwest::Client::builder()
        .user_agent("ikb-core")
        .connect_timeout(Duration::from_secs(8))
        .timeout(Duration::from_secs(15));

    let client = match crate::net::apply_proxy_choice(builder, &net, plan.proxy)
        .map_err(|e| e.to_string())
        .and_then(|b| b.build().map_err(|e| e.to_string()))
    {
        Ok(v) => v,
        Err(e) => {
            return UrlProbeResult {
                ok: false,
                status: "".to_string(),
                via: via.to_string(),
                used_github_proxy: plan.used_github_proxy,
                bytes: 0,
                error: format!("build client failed: {}", e),
            }
        }
    };

    let resp = match client
        .get(plan.url)
        .header("Range", "bytes=0-2047")
        .send()
        .await
    {
        Ok(v) => v,
        Err(e) => {
            return UrlProbeResult {
                ok: false,
                status: "".to_string(),
                via: via.to_string(),
                used_github_proxy: plan.used_github_proxy,
                bytes: 0,
                error: e.to_string(),
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
            let mut out = trimmed.chars().take(160).collect::<String>();
            if trimmed.chars().count() > 160 {
                out.push_str("...");
            }
            format!(" body='{}'", out.replace('\n', " "))
        };
        return UrlProbeResult {
            ok: false,
            status: status.to_string(),
            via: via.to_string(),
            used_github_proxy: plan.used_github_proxy,
            bytes: 0,
            error: format!("HTTP {}{}", status, hint),
        };
    }

    let bytes = match resp.bytes().await {
        Ok(v) => v,
        Err(e) => {
            return UrlProbeResult {
                ok: false,
                status: status.to_string(),
                via: via.to_string(),
                used_github_proxy: plan.used_github_proxy,
                bytes: 0,
                error: e.to_string(),
            }
        }
    };

    UrlProbeResult {
        ok: true,
        status: status.to_string(),
        via: via.to_string(),
        used_github_proxy: plan.used_github_proxy,
        bytes: bytes.len(),
        error: String::new(),
    }
}

pub async fn build_diagnostics_report(
    cfg: &Config,
    config_path: &Path,
    runtime_status: Option<crate::runtime::RuntimeStatus>,
    cli_login: &str,
) -> DiagnosticsReport {
    let now = Local::now().to_rfc3339();

    let mut out = String::new();
    out.push_str("IKB Diagnostics Report\n");
    out.push_str(&format!("generated_at: {}\n", now));
    out.push_str(&format!("ikb-core: {}\n", env!("CARGO_PKG_VERSION")));
    out.push_str("\n");

    out.push_str("[config]\n");
    out.push_str(&format!("path: {}\n", config_path.to_string_lossy()));
    out.push_str(&format!("ikuai_url: {}\n", cfg.ikuai_url.trim()));
    out.push_str(&format!("username: {}\n", cfg.username.trim()));
    out.push_str(&format!("password: {}\n", mask_secret(&cfg.password)));
    out.push_str(&format!("cron: {}\n", cfg.cron.trim()));
    out.push_str(&format!("proxy.mode: {:?}\n", cfg.proxy.mode));
    out.push_str(&format!("proxy.url: {}\n", cfg.proxy.url.trim()));
    out.push_str(&format!("proxy.user: {}\n", cfg.proxy.user.trim()));
    out.push_str(&format!("proxy.pass: {}\n", mask_secret(&cfg.proxy.pass)));
    out.push_str(&format!("github-proxy: {}\n", cfg.github_proxy.trim()));
    out.push_str(&format!("webui.enable: {}\n", cfg.webui.enable));
    out.push_str(&format!("webui.port: {}\n", cfg.webui.port.trim()));
    out.push_str(&format!("webui.user: {}\n", cfg.webui.user.trim()));
    out.push_str(&format!("webui.pass: {}\n", mask_secret(&cfg.webui.pass)));
    out.push_str("\n");

    out.push_str("[rules]\n");
    out.push_str(&format!("custom-isp: {}\n", cfg.custom_isp.len()));
    out.push_str(&format!("stream-domain: {}\n", cfg.stream_domain.len()));
    out.push_str(&format!("ip-group: {}\n", cfg.ip_group.len()));
    out.push_str(&format!("ipv6-group: {}\n", cfg.ipv6_group.len()));
    out.push_str(&format!("stream-ipport: {}\n", cfg.stream_ipport.len()));
    out.push_str("\n");

    out.push_str("[cron]\n");
    match normalize_cron_expr_for_report(&cfg.cron) {
        Ok(norm) => {
            out.push_str(&format!("normalized: {}\n", norm));
            if let Ok(sched) = Schedule::from_str(&norm) {
                if let Some(next) = sched.upcoming(Local).next() {
                    out.push_str(&format!("next: {}\n", next.to_rfc3339()));
                }
            }
        }
        Err(e) => {
            out.push_str(&format!("error: {}\n", e));
        }
    }
    out.push_str("\n");

    if let Some(st) = runtime_status {
        out.push_str("[runtime]\n");
        out.push_str(&format!("running: {}\n", st.running));
        out.push_str(&format!("cron_running: {}\n", st.cron_running));
        out.push_str(&format!("module: {}\n", st.module.trim()));
        out.push_str(&format!("cron_expr: {}\n", st.cron_expr.trim()));
        out.push_str(&format!("last_run_at: {}\n", st.last_run_at.trim()));
        out.push_str(&format!("next_run_at: {}\n", st.next_run_at.trim()));
        out.push_str("\n");
    }

    out.push_str("[checks]\n");
    let login_params = crate::session::resolve_login_params(cfg, cli_login);
    match login_params {
        Ok(p) => {
            out.push_str(&format!("login.source: {:?}\n", p.source));
            out.push_str(&format!("login.base_url: {}\n", p.base_url.trim()));
            out.push_str(&format!("login.username: {}\n", p.username.trim()));
            out.push_str(&format!("login.password: {}\n", mask_secret(&p.password)));

            let r = test_ikuai_login(TestIkuaiLoginRequest {
                base_url: p.base_url,
                username: p.username,
                password: p.password,
            })
            .await;
            out.push_str(&format!("ikuai.login: {}\n", if r.ok { "OK" } else { "FAIL" }));
            if !r.ok {
                out.push_str(&format!("ikuai.login.error: {}\n", r.message));
            }
        }
        Err(e) => {
            out.push_str(&format!("login.resolve: FAIL ({})\n", e));
        }
    }

    if cfg.proxy.mode == crate::config::ProxyMode::Smart && !cfg.github_proxy.trim().is_empty() {
        let r = test_github_proxy(TestGithubProxyRequest {
            github_proxy: cfg.github_proxy.to_string(),
        })
        .await;
        out.push_str(&format!("github-proxy.test: {}\n", if r.ok { "OK" } else { "FAIL" }));
        if !r.ok {
            out.push_str(&format!("github-proxy.error: {}\n", r.message));
        }
    }

    // Probe a small set of URLs to validate proxy/ghproxy behavior.
    // 抽样探测少量 URL，用于验证 proxy/ghproxy 行为。
    let mut probes: Vec<UrlProbe> = Vec::new();
    for it in cfg.custom_isp.iter().take(2) {
        probes.push(UrlProbe {
            label: format!("custom-isp:{}", it.tag.trim()),
            url: it.url.to_string(),
        });
    }
    for it in cfg.stream_domain.iter().take(2) {
        probes.push(UrlProbe {
            label: format!("stream-domain:{}", it.tag.trim()),
            url: it.url.to_string(),
        });
    }
    for it in cfg.ip_group.iter().take(1) {
        probes.push(UrlProbe {
            label: format!("ip-group:{}", it.tag.trim()),
            url: it.url.to_string(),
        });
    }
    for it in cfg.ipv6_group.iter().take(1) {
        probes.push(UrlProbe {
            label: format!("ipv6-group:{}", it.tag.trim()),
            url: it.url.to_string(),
        });
    }

    if !probes.is_empty() {
        out.push_str("\n[url-probe]\n");
    }
    for p in probes {
        let res = probe_rule_url(cfg, &p.url).await;
        if res.ok {
            out.push_str(&format!(
                "{}: OK status={} via={} ghproxy={} bytes={} url='{}'\n",
                p.label,
                res.status,
                res.via,
                if res.used_github_proxy { "1" } else { "0" },
                res.bytes,
                p.url
            ));
        } else {
            out.push_str(&format!(
                "{}: FAIL via={} ghproxy={} url='{}' error='{}'\n",
                p.label,
                res.via,
                if res.used_github_proxy { "1" } else { "0" },
                p.url,
                res.error.replace('\n', " ")
            ));
        }
    }

    DiagnosticsReport {
        generated_at: now,
        text: out,
    }
}
