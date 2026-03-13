use std::path::PathBuf;
use std::sync::Arc;
use std::net::TcpListener;

use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::middleware::Next;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine;
use serde::{Deserialize, Serialize};
use tower_http::services::ServeDir;
use async_stream::stream;
use axum::response::sse::{Event, Sse};
use std::convert::Infallible;

use ikb_core::runtime::RuntimeService;

struct AppState {
    config_path: PathBuf,
    config: Arc<tokio::sync::Mutex<ikb_core::config::Config>>,
    runtime: Arc<RuntimeService>,
}

#[derive(Debug, Serialize)]
struct ConfigResponse {
    #[serde(flatten)]
    config: serde_json::Value,
    exe_path: String,
    conf_path: String,
    raw_yaml: String,
    top_level_comments: std::collections::BTreeMap<String, String>,
    item_comments: std::collections::BTreeMap<String, String>,
    webui_comments: std::collections::BTreeMap<String, String>,
    max_number_of_one_records_comments: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct SaveRequest {
    #[serde(flatten)]
    config: ikb_core::config::Config,
    with_comments: bool,
}

#[derive(Debug, Deserialize)]
struct SaveRawYamlRequest {
    yaml_text: String,
    with_comments: bool,
}

#[derive(Debug, Serialize)]
struct TestResult {
    ok: bool,
    message: String,
}

#[derive(Debug, Deserialize)]
struct TestIkuaiLoginRequest {
    #[serde(alias = "baseUrl")]
    base_url: String,
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct TestGithubProxyRequest {
    #[serde(alias = "githubProxy")]
    github_proxy: String,
}

pub fn start_web_server(
    config_path: PathBuf,
    config: Arc<tokio::sync::Mutex<ikb_core::config::Config>>,
    port: String,
) -> Result<(), String> {
    let default_cron = config.blocking_lock().cron.to_string();
    let runtime = Arc::new(RuntimeService::new(
        Arc::clone(&config),
        String::new(),
        default_cron,
        "ispdomain".to_string(),
    ));
    let state = Arc::new(AppState {
        config_path,
        config,
        runtime,
    });

    let api = Router::new()
        .route("/api/config", get(api_config))
        .route("/api/save", post(api_save))
        .route("/api/save-raw", post(api_save_raw_yaml))
        .route("/api/test/ikuai-login", post(api_test_ikuai_login))
        .route("/api/test/github-proxy", post(api_test_github_proxy))
        .route("/api/runtime/status", get(api_runtime_status))
        .route("/api/runtime/run-once", post(api_runtime_run_once))
        .route("/api/runtime/cron/start", post(api_runtime_cron_start))
        .route("/api/runtime/cron/stop", post(api_runtime_cron_stop))
        .route("/api/runtime/stop", post(api_runtime_stop))
        .route("/api/runtime/clean", post(api_runtime_clean))
        .route("/api/runtime/logs", get(api_runtime_logs))
        .route("/api/runtime/logs/stream", get(api_runtime_logs_stream));

    let dist = find_frontend_dist_dir();
    let app = if let Some(dist) = dist {
        Router::new()
            .merge(api)
            .fallback_service(ServeDir::new(dist))
            .layer(axum::middleware::from_fn_with_state(Arc::clone(&state), basic_auth))
            .with_state(Arc::clone(&state))
    } else {
        Router::new()
            .route("/", get(index))
            .merge(api)
            .layer(axum::middleware::from_fn_with_state(Arc::clone(&state), basic_auth))
            .with_state(Arc::clone(&state))
    };

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).map_err(|e| e.to_string())?;
    listener
        .set_nonblocking(true)
        .map_err(|e| e.to_string())?;

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[ERR:启动失败] Failed to create tokio runtime: {}", e);
                return;
            }
        };
        rt.block_on(async move {
            let listener = match tokio::net::TcpListener::from_std(listener) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("[ERR:启动失败] Failed to init tokio listener: {}", e);
                    return;
                }
            };
            let _ = axum::serve(listener, app).await;
        });
    });

    println!("[WEB:服务启动] WebUI is available at http://0.0.0.0:{}", port);
    Ok(())
}

fn find_frontend_dist_dir() -> Option<std::path::PathBuf> {
    fn valid(p: &std::path::Path) -> bool {
        p.join("index.html").is_file()
    }

    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("app/frontend/dist"));
        candidates.push(cwd.join("./app/frontend/dist"));
    }

    if let Ok(exe) = std::env::current_exe()
        && let Some(exe_dir) = exe.parent()
    {
        candidates.push(exe_dir.join("../app/frontend/dist"));
        candidates.push(exe_dir.join("../../app/frontend/dist"));

        for ancestor in exe_dir.ancestors().take(8) {
            candidates.push(ancestor.join("app/frontend/dist"));
        }
    }

    candidates.push(std::path::PathBuf::from("app/frontend/dist"));
    candidates.push(std::path::PathBuf::from("./app/frontend/dist"));

    candidates.into_iter().find(|p| valid(p))
}

async fn index() -> Html<&'static str> {
    Html("<html><body><h1>iKuai Bypass WebUI (Rust)</h1></body></html>")
}

async fn api_config(State(state): State<Arc<AppState>>) -> Response {
    let exe_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_default();

    let conf_path = if state.config_path.is_absolute() {
        state.config_path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join(&state.config_path)
    };
    let conf_path = conf_path.to_string_lossy().to_string();

    let cfg_guard = state.config.lock().await;
    let config = match serde_json::to_value(&*cfg_guard) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to encode config: {}", e),
            )
                .into_response();
        }
    };
    let raw_yaml = std::fs::read_to_string(&state.config_path).unwrap_or_default();
    let resp = ConfigResponse {
        config,
        exe_path,
        conf_path,
        raw_yaml,
        top_level_comments: ikb_core::config::top_level_comments(),
        item_comments: ikb_core::config::item_comments(),
        webui_comments: ikb_core::config::webui_comments(),
        max_number_of_one_records_comments: ikb_core::config::max_number_of_one_records_comments(),
    };
    (StatusCode::OK, Json(resp)).into_response()
}

async fn api_save(State(state): State<Arc<AppState>>, Json(req): Json<SaveRequest>) -> Response {
    let allow = state.config.lock().await.webui.enable;
    if !allow {
        return (
            StatusCode::FORBIDDEN,
            "Forbidden: WebUI is disabled in configuration",
        )
            .into_response();
    }

    if let Err(e) = req
        .config
        .save_to_path_with_comments(&state.config_path, req.with_comments)
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save config: {}", e),
        )
            .into_response();
    }

    let new_cron = req.config.cron.to_string();
    {
        let mut current = state.config.lock().await;
        *current = req.config;
    }
    state.runtime.set_defaults(None, Some(new_cron)).await;

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        r#"{"status":"success","message":"Configuration saved successfully"}"#,
    )
        .into_response()
}

async fn api_save_raw_yaml(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SaveRawYamlRequest>,
) -> Response {
    let allow = state.config.lock().await.webui.enable;
    if !allow {
        return (
            StatusCode::FORBIDDEN,
            "Forbidden: WebUI is disabled in configuration",
        )
            .into_response();
    }

    let cfg = match ikb_core::config::Config::validate_and_save_raw_yaml(
        &req.yaml_text,
        &state.config_path,
        req.with_comments,
    ) {
        Ok(cfg) => cfg,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                format!("Failed to save config: {}", e),
            )
                .into_response()
        }
    };

    let new_cron = cfg.cron.to_string();
    {
        let mut current = state.config.lock().await;
        *current = cfg;
    }
    state.runtime.set_defaults(None, Some(new_cron)).await;

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        r#"{"status":"success","message":"Raw YAML saved successfully"}"#,
    )
        .into_response()
}

fn normalize_base_url(input: &str) -> String {
    let raw = input.trim();
    if raw.is_empty() {
        return String::new();
    }
    if raw.contains("://") {
        return raw.to_string();
    }
    format!("http://{}", raw)
}

fn normalize_url_prefix(input: &str) -> String {
    let raw = input.trim();
    if raw.is_empty() {
        return String::new();
    }
    if raw.contains("://") {
        return raw.to_string();
    }
    format!("https://{}", raw)
}

async fn api_test_ikuai_login(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<TestIkuaiLoginRequest>,
) -> impl IntoResponse {
    let base_url = normalize_base_url(&req.base_url);
    let username = req.username.trim().to_string();
    if base_url.is_empty() {
        return (StatusCode::OK, Json(TestResult { ok: false, message: "Empty iKuai URL".to_string() }));
    }
    if username.is_empty() {
        return (StatusCode::OK, Json(TestResult { ok: false, message: "Empty username".to_string() }));
    }

    let api = match ikb_core::ikuai::IKuaiClient::new(base_url) {
        Ok(v) => v,
        Err(e) => {
            return (StatusCode::OK, Json(TestResult { ok: false, message: e.to_string() }));
        }
    };

    match api.login(&username, &req.password).await {
        Ok(()) => (StatusCode::OK, Json(TestResult { ok: true, message: "OK".to_string() })),
        Err(e) => (StatusCode::OK, Json(TestResult { ok: false, message: e.to_string() })),
    }
}

async fn api_test_github_proxy(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<TestGithubProxyRequest>,
) -> impl IntoResponse {
    const URL: &str = "https://raw.githubusercontent.com/joyanhui/ikuai-bypass/refs/heads/main/.gitignore";

    let proxy = normalize_url_prefix(&req.github_proxy);
    if proxy.is_empty() {
        return (StatusCode::OK, Json(TestResult { ok: false, message: "Empty github proxy".to_string() }));
    }

    let final_url = if proxy.ends_with('/') {
        format!("{}{}", proxy, URL)
    } else {
        format!("{}/{}", proxy, URL)
    };

    let client = match reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(15))
        .build()
    {
        Ok(v) => v,
        Err(e) => {
            return (StatusCode::OK, Json(TestResult { ok: false, message: e.to_string() }));
        }
    };

    let resp = match client.get(&final_url).send().await {
        Ok(v) => v,
        Err(e) => {
            return (StatusCode::OK, Json(TestResult { ok: false, message: e.to_string() }));
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
        return (StatusCode::OK, Json(TestResult { ok: false, message: format!("HTTP {} url='{}'{}", status, final_url, hint) }));
    }
    let text = match resp.text().await {
        Ok(v) => v,
        Err(e) => {
            return (StatusCode::OK, Json(TestResult { ok: false, message: e.to_string() }));
        }
    };
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return (
            StatusCode::OK,
            Json(TestResult { ok: false, message: format!("Empty response url='{}'", final_url) }),
        );
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.contains("<html") || lower.contains("<!doctype") {
        let mut out = trimmed.chars().take(200).collect::<String>();
        if trimmed.chars().count() > 200 {
            out.push_str("...");
        }
        return (
            StatusCode::OK,
            Json(TestResult {
                ok: false,
                message: format!("Unexpected HTML url='{}' body='{}'", final_url, out.replace('\n', " ")),
            }),
        );
    }
    (StatusCode::OK, Json(TestResult { ok: true, message: format!("OK url='{}'", final_url) }))
}

async fn api_runtime_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    (StatusCode::OK, Json(state.runtime.status()))
}

#[derive(Debug, Deserialize)]
struct RunOnceRequest {
    module: String,
}

async fn api_runtime_run_once(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RunOnceRequest>,
) -> Response {
    match Arc::clone(&state.runtime).start_run_once(req.module).await {
        Ok(started) => (StatusCode::OK, Json(serde_json::json!({"started": started}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to start run: {}", e),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct CronStartRequest {
    expr: String,
    module: String,
}

async fn api_runtime_cron_start(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CronStartRequest>,
) -> Response {
    if let Err(e) = Arc::clone(&state.runtime)
        .start_cron(req.expr, req.module)
        .await
    {
        return (
            StatusCode::BAD_REQUEST,
            format!("Failed to start cron: {}", e),
        )
            .into_response();
    }
    (StatusCode::OK, Json(serde_json::json!({"status": "success"}))).into_response()
}

async fn api_runtime_cron_stop(State(state): State<Arc<AppState>>) -> Response {
    if let Err(e) = Arc::clone(&state.runtime).stop_cron().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to stop cron: {}", e),
        )
            .into_response();
    }
    (StatusCode::OK, Json(serde_json::json!({"status": "success"}))).into_response()
}

async fn api_runtime_stop(State(state): State<Arc<AppState>>) -> Response {
    if let Err(e) = Arc::clone(&state.runtime).stop_all().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to stop runtime: {}", e),
        )
            .into_response();
    }
    (StatusCode::OK, Json(serde_json::json!({"status": "success"}))).into_response()
}

#[derive(Debug, Deserialize)]
struct CleanRequest {
    clean_tag: String,
}

async fn run_clean(config: &ikb_core::config::Config, clean_tag: String) -> Result<(), String> {
    let tag = clean_tag.trim().to_string();
    if tag.is_empty() {
        return Err("Clean mode requires clean_tag".to_string());
    }

    let params = ikb_core::session::resolve_login_params(config, "")
        .map_err(|_| "Invalid login parameters".to_string())?;
    let api = ikb_core::ikuai::IKuaiClient::new(params.base_url.to_string())
        .map_err(|e| e.to_string())?;
    api.login(&params.username, &params.password)
        .await
        .map_err(|e| e.to_string())?;

    ikb_core::ikuai::custom_isp::del_custom_isp_all(&api, &tag)
        .await
        .map_err(|e| e.to_string())?;
    ikb_core::ikuai::stream_domain::del_stream_domain_all(&api, &tag)
        .await
        .map_err(|e| e.to_string())?;
    ikb_core::ikuai::ip_group::del_ikuai_bypass_ip_group(&api, &tag)
        .await
        .map_err(|e| e.to_string())?;
    ikb_core::ikuai::ipv6_group::del_ikuai_bypass_ipv6_group(&api, &tag)
        .await
        .map_err(|e| e.to_string())?;
    ikb_core::ikuai::stream_ipport::del_ikuai_bypass_stream_ipport(&api, &tag)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

async fn api_runtime_clean(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CleanRequest>,
) -> Response {
    let cfg_guard = state.config.lock().await;
    match run_clean(&cfg_guard, req.clean_tag).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"status": "success"}))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

async fn api_runtime_logs(State(state): State<Arc<AppState>>, req: axum::http::Request<axum::body::Body>) -> Response {
    let query = req.uri().query().unwrap_or("");
    let tail = parse_tail_query(query).unwrap_or(200);
    let logs = state.runtime.tail_logs(tail).await;
    (StatusCode::OK, Json(logs)).into_response()
}

fn parse_tail_query(query: &str) -> Option<usize> {
    for part in query.split('&') {
        let mut kv = part.splitn(2, '=');
        let k = kv.next().unwrap_or("");
        let v = kv.next().unwrap_or("");
        if k == "tail" && let Ok(n) = v.parse::<usize>() && n > 0 {
            return Some(n);
        }
    }
    None
}

async fn api_runtime_logs_stream(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let runtime = Arc::clone(&state.runtime);
    let out = stream! {
        let mut rx = runtime.subscribe_logs().await;
        loop {
            let msg = rx.recv().await;
            if let Ok(entry) = msg {
                let data = serde_json::to_string(&entry).unwrap_or_default();
                yield Ok::<Event, Infallible>(Event::default().data(data));
            }
        }
    };
    Sse::new(out).keep_alive(axum::response::sse::KeepAlive::default())
}

async fn basic_auth(
    State(state): State<Arc<AppState>>,
    req: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Response {
    let (user, pass) = {
        let cfg = state.config.lock().await;
        (cfg.webui.user.to_string(), cfg.webui.pass.to_string())
    };
    if user.is_empty() {
        return next.run(req).await;
    }

    let auth = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();
    if !auth.starts_with("Basic ") {
        return unauthorized();
    }
    let encoded = &auth[6..];
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(encoded.as_bytes())
        .ok();
    let decoded = match decoded {
        Some(v) => v,
        None => return unauthorized(),
    };
    let decoded = String::from_utf8(decoded).ok();
    let decoded = match decoded {
        Some(v) => v,
        None => return unauthorized(),
    };
    let mut parts = decoded.splitn(2, ':');
    let u = parts.next().unwrap_or("");
    let p = parts.next().unwrap_or("");
    if !constant_time_eq(u.as_bytes(), user.as_bytes())
        || !constant_time_eq(p.as_bytes(), pass.as_bytes())
    {
        return unauthorized();
    }
    next.run(req).await
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    let max = a.len().max(b.len());
    let mut diff: u8 = (a.len() ^ b.len()) as u8;
    for i in 0..max {
        let aa = a.get(i).copied().unwrap_or(0);
        let bb = b.get(i).copied().unwrap_or(0);
        diff |= aa ^ bb;
    }
    diff == 0
}

fn unauthorized() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, "Basic realm=\"Restricted\"")],
        "Unauthorized",
    )
        .into_response()
}
