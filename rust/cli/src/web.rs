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

use crate::runtime::RuntimeService;

#[derive(Clone)]
struct AppState {
    config_path: PathBuf,
    config: Arc<tokio::sync::Mutex<ikb_core::config::Config>>,
    runtime: Arc<RuntimeService>,
}

#[derive(Debug, Serialize)]
struct ConfigResponse {
    #[serde(flatten)]
    config: ikb_core::config::Config,
    exe_path: String,
    conf_path: String,
}

#[derive(Debug, Deserialize)]
struct SaveRequest {
    #[serde(flatten)]
    config: ikb_core::config::Config,
    with_comments: bool,
}

pub fn start_web_server(
    config_path: PathBuf,
    cfg: ikb_core::config::Config,
    port: String,
) -> Result<(), String> {
    let config = Arc::new(tokio::sync::Mutex::new(cfg));
    let default_cron = config.blocking_lock().cron.clone();
    let runtime = Arc::new(RuntimeService::new(
        Arc::clone(&config),
        String::new(),
        default_cron,
        "ispdomain".to_string(),
    ));
    let state = AppState {
        config_path,
        config,
        runtime,
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/api/config", get(api_config))
        .route("/api/save", post(api_save))
        .route("/api/runtime/status", get(api_runtime_status))
        .route("/api/runtime/run-once", post(api_runtime_run_once))
        .route("/api/runtime/cron/start", post(api_runtime_cron_start))
        .route("/api/runtime/cron/stop", post(api_runtime_cron_stop))
        .route("/api/runtime/logs", get(api_runtime_logs))
        .route("/api/runtime/logs/stream", get(api_runtime_logs_stream))
        .layer(axum::middleware::from_fn_with_state(state.clone(), basic_auth))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).map_err(|e| e.to_string())?;
    listener
        .set_nonblocking(true)
        .map_err(|e| e.to_string())?;

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        rt.block_on(async move {
            let listener = tokio::net::TcpListener::from_std(listener).expect("tokio listener");
            let _ = axum::serve(listener, app).await;
        });
    });

    println!("[WEB:服务启动] WebUI is available at http://0.0.0.0:{}", port);
    Ok(())
}

async fn index() -> Html<&'static str> {
    Html("<html><body><h1>iKuai Bypass WebUI (Rust)</h1></body></html>")
}

async fn api_config(State(state): State<AppState>) -> impl IntoResponse {
    let exe_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_default();

    let conf_path = state
        .config_path
        .to_str()
        .map(|s| s.to_string())
        .unwrap_or_default();

    let cfg = state.config.lock().await.clone();
    let resp = ConfigResponse {
        config: cfg,
        exe_path,
        conf_path,
    };
    (StatusCode::OK, Json(resp))
}

async fn api_save(State(state): State<AppState>, Json(req): Json<SaveRequest>) -> Response {
    let allow = state.config.lock().await.webui.enable_update;
    if !allow {
        return (
            StatusCode::FORBIDDEN,
            "Forbidden: Online update is disabled in configuration",
        )
            .into_response();
    }

    let _ = req.with_comments;
    if let Err(e) = req.config.save_to_path(&state.config_path) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save config: {}", e),
        )
            .into_response();
    }

    let new_cron = req.config.cron.clone();
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

async fn api_runtime_status(State(state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, Json(state.runtime.status()))
}

#[derive(Debug, Deserialize)]
struct RunOnceRequest {
    module: String,
}

async fn api_runtime_run_once(
    State(state): State<AppState>,
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
    State(state): State<AppState>,
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

async fn api_runtime_cron_stop(State(state): State<AppState>) -> Response {
    if let Err(e) = Arc::clone(&state.runtime).stop_cron().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to stop cron: {}", e),
        )
            .into_response();
    }
    (StatusCode::OK, Json(serde_json::json!({"status": "success"}))).into_response()
}

async fn api_runtime_logs(State(state): State<AppState>, req: axum::http::Request<axum::body::Body>) -> Response {
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
        if k == "tail" {
            if let Ok(n) = v.parse::<usize>() {
                if n > 0 {
                    return Some(n);
                }
            }
        }
    }
    None
}

async fn api_runtime_logs_stream(State(state): State<AppState>) -> impl IntoResponse {
    Arc::clone(&state.runtime).sse_stream().await
}

async fn basic_auth(
    State(state): State<AppState>,
    req: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Response {
    let (user, pass) = {
        let cfg = state.config.lock().await;
        (cfg.webui.user.clone(), cfg.webui.pass.clone())
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
    if u != user || p != pass {
        return unauthorized();
    }
    next.run(req).await
}

fn unauthorized() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, "Basic realm=\"Restricted\"")],
        "Unauthorized",
    )
        .into_response()
}
