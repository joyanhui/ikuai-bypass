use std::path::PathBuf;
use std::sync::Arc;

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

fn display_conf_path(p: &PathBuf) -> String {
    if p.is_absolute() {
        return p.to_string_lossy().to_string();
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    cwd.join(p).to_string_lossy().to_string()
}

fn print_webui_banner(
    port: &str,
    conf_path: &str,
    auth_user: &str,
) {
    let listen_addr = format!("0.0.0.0:{}", port);
    let open_url = format!("http://127.0.0.1:{}", port);
    let auth_user = auth_user.trim();
    let auth_enabled = !auth_user.is_empty();

    println!();
    println!("===========================================================");
    println!("[WEB:服务启动] iKuai Bypass WebUI");
    println!("-----------------------------------------------------------");
    println!("访问地址: {}", open_url);
    println!("监听地址: {}", listen_addr);
    println!("配置路径: {}", conf_path);
    if auth_enabled {
        println!("认证模式: BasicAuth 已开启 (user: {})", auth_user);
    } else {
        println!("认证模式: BasicAuth 未开启 (webui.user 为空)");
    }
    // 在线保存为强制开启（不再支持 enable_update 之类的开关）。
    // Online save is forced on (no enable_update switch).
    println!("在线保存: 已开启 (固定)");
    println!("-----------------------------------------------------------");
    if !auth_enabled {
        println!("警告: 当前未启用 BasicAuth，WebUI 将对局域网完全开放");
        println!("警告: /api/config 会返回包含密码的配置内容；/api/save 可写入配置；/api/runtime/clean 可清理规则");
        println!("提示: 建议在配置文件中设置 webui.user/webui.pass 启用 BasicAuth");
    }
    println!("提示: 停止定时任务后，计划任务将不会再按 Cron 自动执行");
    println!("退出方式: Ctrl+C");
    println!("===========================================================");
    println!();
}

struct AppState {
    config_path: PathBuf,
    config: Arc<tokio::sync::Mutex<Arc<ikb_core::config::Config>>>,
    runtime: Arc<RuntimeService>,
    cli_login: String,
}

#[derive(Debug, Serialize)]
struct ConfigResponse {
    #[serde(flatten)]
    meta: ikb_core::app::ConfigMeta,
    exe_path: String,
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

pub async fn start_web_server(
    config_path: PathBuf,
    config: Arc<tokio::sync::Mutex<Arc<ikb_core::config::Config>>>,
    runtime: Arc<RuntimeService>,
    cli_login: String,
    port: String,
) -> Result<(), String> {
    let state = Arc::new(AppState {
        config_path,
        config,
        runtime,
        cli_login,
    });

    let api = Router::new()
        .route("/api/config", get(api_config))
        .route("/api/diagnostics/report", get(api_diagnostics_report))
        .route("/api/save", post(api_save))
        .route("/api/save-raw", post(api_save_raw_yaml))
        .route("/api/remote/fetch", post(api_remote_fetch))
        .route("/api/test/ikuai-login", post(api_test_ikuai_login))
        .route("/api/test/github-proxy", post(api_test_github_proxy))
        .route("/api/github/releases", get(api_github_releases).post(api_github_releases_with_proxy))
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
    let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| e.to_string())?;
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    let auth_user = {
        let cfg = state.config.lock().await;
        cfg.as_ref().webui.user.to_string()
    };
    let conf_path = display_conf_path(&state.config_path);
    print_webui_banner(&port, &conf_path, &auth_user);
    Ok(())
}

fn find_frontend_dist_dir() -> Option<std::path::PathBuf> {
    fn valid(p: &std::path::Path) -> bool {
        p.join("index.html").is_file()
    }

    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("frontends/app/dist"));
        candidates.push(cwd.join("./frontends/app/dist"));
        // Legacy paths (pre-restructure)
        candidates.push(cwd.join("app/frontend/dist"));
        candidates.push(cwd.join("./app/frontend/dist"));
    }

    if let Ok(exe) = std::env::current_exe()
        && let Some(exe_dir) = exe.parent()
    {
        candidates.push(exe_dir.join("../frontends/app/dist"));
        candidates.push(exe_dir.join("../../frontends/app/dist"));
        // Legacy paths (pre-restructure)
        candidates.push(exe_dir.join("../app/frontend/dist"));
        candidates.push(exe_dir.join("../../app/frontend/dist"));

        for ancestor in exe_dir.ancestors().take(8) {
            candidates.push(ancestor.join("frontends/app/dist"));
            candidates.push(ancestor.join("app/frontend/dist"));
        }
    }

    candidates.push(std::path::PathBuf::from("frontends/app/dist"));
    candidates.push(std::path::PathBuf::from("./frontends/app/dist"));
    // Legacy paths (pre-restructure)
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

    // Avoid holding config lock while reading config file.
    let cfg_snapshot = { Arc::clone(&*state.config.lock().await) };
    let meta = match ikb_core::app::build_config_meta(cfg_snapshot.as_ref(), &state.config_path) {
        Ok(v) => v,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };
    let resp = ConfigResponse { meta, exe_path };
    // 返回内容包含密码等敏感信息，避免被浏览器/代理缓存。
    // Response contains secrets; disable caching.
    (
        StatusCode::OK,
        [(header::CACHE_CONTROL, "no-store")],
        Json(resp),
    )
        .into_response()
}

async fn api_diagnostics_report(State(state): State<Arc<AppState>>) -> Response {
    let cfg_snapshot = { Arc::clone(&*state.config.lock().await) };
    let cli_login = state.cli_login.to_string();
    let st = state.runtime.status();

    let report =
        ikb_core::app::build_diagnostics_report(cfg_snapshot.as_ref(), &state.config_path, Some(st), &cli_login)
            .await;
    (
        StatusCode::OK,
        [(header::CACHE_CONTROL, "no-store")],
        Json(report),
    )
        .into_response()
}

async fn api_save(State(state): State<Arc<AppState>>, Json(req): Json<SaveRequest>) -> Response {
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
        *current = Arc::new(req.config);
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
        *current = Arc::new(cfg);
    }
    state.runtime.set_defaults(None, Some(new_cron)).await;

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        r#"{"status":"success","message":"Raw YAML saved successfully"}"#,
    )
        .into_response()
}

async fn api_test_ikuai_login(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<ikb_core::app::TestIkuaiLoginRequest>,
) -> impl IntoResponse {
    let r = ikb_core::app::test_ikuai_login(req).await;
    (StatusCode::OK, Json(r))
}

async fn api_test_github_proxy(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<ikb_core::app::TestGithubProxyRequest>,
) -> impl IntoResponse {
    let r = ikb_core::app::test_github_proxy(req).await;
    (StatusCode::OK, Json(r))
}

#[derive(Debug, Deserialize)]
struct RemoteFetchRequest {
    url: String,
    proxy: ikb_core::config::ProxyConfig,
    #[serde(alias = "githubProxy")]
    github_proxy: String,
}

async fn api_remote_fetch(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<RemoteFetchRequest>,
) -> Response {
    match ikb_core::app::fetch_remote_config(&req.url, &req.proxy, &req.github_proxy).await {
        Ok(text) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            text,
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            e,
        )
            .into_response(),
    }
}

async fn api_github_releases(State(state): State<Arc<AppState>>) -> Response {
    let cfg = { Arc::clone(&*state.config.lock().await) };
    match ikb_core::app::fetch_github_releases(&cfg.proxy).await {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(e) => (StatusCode::BAD_GATEWAY, e).into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct GithubReleasesWithProxyRequest {
    proxy: ikb_core::config::ProxyConfig,
}

async fn api_github_releases_with_proxy(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<GithubReleasesWithProxyRequest>,
) -> Response {
    match ikb_core::app::fetch_github_releases(&req.proxy).await {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(e) => (StatusCode::BAD_GATEWAY, e).into_response(),
    }
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

async fn api_runtime_clean(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CleanRequest>,
) -> Response {
    // 避免在网络请求期间持有配置锁。
    // Avoid holding config lock while doing network requests.
    let cfg_snapshot = { Arc::clone(&*state.config.lock().await) };
    let cli_login = state.cli_login.to_string();
    match ikb_core::app::run_clean(cfg_snapshot.as_ref(), &cli_login, &req.clean_tag).await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "success"}))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
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
        (
            cfg.as_ref().webui.user.to_string(),
            cfg.as_ref().webui.pass.to_string(),
        )
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
    // 注意：不能把长度差异截断到 u8，否则在极端情况下可能出现错误的相等判断。
    // Note: do not truncate length diff into u8.
    let mut diff: usize = a.len() ^ b.len();
    for i in 0..max {
        let aa = a.get(i).copied().unwrap_or(0);
        let bb = b.get(i).copied().unwrap_or(0);
        diff |= (aa ^ bb) as usize;
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
