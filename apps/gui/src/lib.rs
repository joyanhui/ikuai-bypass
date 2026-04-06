use std::path::PathBuf;
use std::sync::Arc;

use ikb_core::config::Config;
use ikb_core::runtime::RuntimeService;
use tauri::{Emitter, Manager};

#[tauri::command]
async fn get_config(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let cfg = state.config.lock().await;
    serde_json::to_value(cfg.as_ref()).map_err(|e| format!("Failed to encode config: {}", e))
}

type ConfigMeta = ikb_core::app::ConfigMeta;
type TestResult = ikb_core::app::TestResult;
type TestIkuaiLoginReq = ikb_core::app::TestIkuaiLoginRequest;
type TestGithubProxyReq = ikb_core::app::TestGithubProxyRequest;
type GithubRelease = ikb_core::app::GithubRelease;
type DiagnosticsReport = ikb_core::app::DiagnosticsReport;

#[tauri::command]
async fn test_ikuai_login(
    _state: tauri::State<'_, AppState>,
    req: TestIkuaiLoginReq,
) -> Result<TestResult, String> {
    Ok(ikb_core::app::test_ikuai_login(req).await)
}

#[tauri::command]
async fn test_github_proxy(
    _state: tauri::State<'_, AppState>,
    req: TestGithubProxyReq,
) -> Result<TestResult, String> {
    Ok(ikb_core::app::test_github_proxy(req).await)
}

#[tauri::command]
async fn get_config_meta(state: tauri::State<'_, AppState>) -> Result<ConfigMeta, String> {
    // Avoid holding locks across await to prevent deadlocks on mobile.
    let cfg_snapshot = { Arc::clone(&*state.config.lock().await) };
    let path_guard = state.config_path.lock().await;
    ikb_core::app::build_config_meta(cfg_snapshot.as_ref(), &path_guard)
}

#[tauri::command]
async fn save_config(state: tauri::State<'_, AppState>, config: Config) -> Result<(), String> {
    {
        let path_guard = state.config_path.lock().await;
        if let Err(e) = config.save_to_path(&*path_guard) {
            return Err(format!("Failed to save config: {}", e));
        }
    }

    let new_cron = config.cron.to_string();
    *state.config.lock().await = Arc::new(config);
    state.runtime.set_defaults(None, Some(new_cron)).await;
    Ok(())
}

#[tauri::command]
async fn save_config_with_comments(
    state: tauri::State<'_, AppState>,
    config: Config,
    with_comments: bool,
) -> Result<(), String> {
    {
        let path_guard = state.config_path.lock().await;
        if let Err(e) = config.save_to_path_with_comments(&*path_guard, with_comments) {
            return Err(format!("Failed to save config: {}", e));
        }
    }
    let new_cron = config.cron.to_string();
    *state.config.lock().await = Arc::new(config);
    state.runtime.set_defaults(None, Some(new_cron)).await;
    Ok(())
}

#[tauri::command]
async fn save_raw_yaml(
    state: tauri::State<'_, AppState>,
    yaml_text: String,
    with_comments: bool,
) -> Result<(), String> {
    let cfg = {
        let path_guard = state.config_path.lock().await;
        Config::validate_and_save_raw_yaml(&yaml_text, &*path_guard, with_comments)
            .map_err(|e| format!("Failed to save config: {}", e))?
    };
    let new_cron = cfg.cron.to_string();
    *state.config.lock().await = Arc::new(cfg);
    state.runtime.set_defaults(None, Some(new_cron)).await;
    Ok(())
}

#[tauri::command]
async fn runtime_status(
    state: tauri::State<'_, AppState>,
) -> Result<ikb_core::runtime::RuntimeStatus, String> {
    Ok(state.runtime.status())
}

#[tauri::command]
async fn runtime_run_once(
    state: tauri::State<'_, AppState>,
    module: String,
) -> Result<bool, String> {
    let started = Arc::clone(&state.runtime)
        .start_run_once(module)
        .await
        .map_err(|e| e.to_string())?;
    Ok(started)
}

#[tauri::command]
async fn runtime_cron_start(
    state: tauri::State<'_, AppState>,
    expr: String,
    module: String,
) -> Result<(), String> {
    Arc::clone(&state.runtime)
        .start_cron(expr, module)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn runtime_cron_stop(state: tauri::State<'_, AppState>) -> Result<(), String> {
    Arc::clone(&state.runtime)
        .stop_cron()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn runtime_stop(state: tauri::State<'_, AppState>) -> Result<(), String> {
    Arc::clone(&state.runtime)
        .stop_all()
        .await
        .map_err(|e| e.to_string())
}

async fn run_clean(config: &Config, clean_tag: String) -> Result<(), String> {
    ikb_core::app::run_clean(config, "", &clean_tag)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn runtime_clean(state: tauri::State<'_, AppState>, clean_tag: String) -> Result<(), String> {
    // 避免在网络请求期间持有配置锁。
    // Avoid holding config lock while doing network requests.
    let cfg = { Arc::clone(&*state.config.lock().await) };
    run_clean(cfg.as_ref(), clean_tag).await
}

#[tauri::command]
async fn runtime_tail_logs(
    state: tauri::State<'_, AppState>,
    tail: Option<usize>,
) -> Result<Vec<ikb_core::logger::LogRecord>, String> {
    Ok(state.runtime.tail_logs(tail.unwrap_or(200)).await)
}

#[tauri::command]
async fn fetch_remote_config(
    url: String,
    proxy: ikb_core::config::ProxyConfig,
    github_proxy: String,
) -> Result<String, String> {
    ikb_core::app::fetch_remote_config(&url, &proxy, &github_proxy).await
}

#[tauri::command]
async fn fetch_github_releases(
    proxy: ikb_core::config::ProxyConfig,
) -> Result<Vec<GithubRelease>, String> {
    ikb_core::app::fetch_github_releases(&proxy).await
}

#[tauri::command]
async fn diagnostics_report(
    state: tauri::State<'_, AppState>,
) -> Result<DiagnosticsReport, String> {
    // Avoid holding locks across await.
    let cfg_snapshot = { Arc::clone(&*state.config.lock().await) };
    let path_guard = state.config_path.lock().await;
    let st = state.runtime.status();
    Ok(
        ikb_core::app::build_diagnostics_report(cfg_snapshot.as_ref(), &path_guard, Some(st), "")
            .await,
    )
}

pub struct AppState {
    config: Arc<tokio::sync::Mutex<Arc<Config>>>,
    runtime: Arc<RuntimeService>,
    config_path: Arc<tokio::sync::Mutex<PathBuf>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let fallback_config_path = ikb_core::paths::default_config_path();

    let mut fallback_cfg = Config {
        ikuai_url: String::new(),
        username: String::new(),
        password: String::new(),
        cron: "0 7 * * *".to_string(),
        add_err_retry_wait: std::time::Duration::from_secs(10),
        add_wait: std::time::Duration::from_secs(1),
        github_proxy: String::new(),
        proxy: Default::default(),
        custom_isp: Vec::new(),
        stream_domain: Vec::new(),
        ip_group: Vec::new(),
        ipv6_group: Vec::new(),
        stream_ipport: Vec::new(),
        webui: Default::default(),
        max_number_of_one_records: Default::default(),
    };
    fallback_cfg.apply_defaults();
    let config = Arc::new(tokio::sync::Mutex::new(Arc::new(fallback_cfg)));

    let runtime = Arc::new(RuntimeService::new(
        Arc::clone(&config),
        String::new(),
        String::new(),
        "ispdomain".to_string(),
        Arc::new(Default::default()),
    ));

    let runtime_for_logs = Arc::clone(&runtime);
    let config_for_setup = Arc::clone(&config);

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .manage(AppState {
            config: Arc::clone(&config),
            runtime: Arc::clone(&runtime),
            config_path: Arc::new(tokio::sync::Mutex::new(fallback_config_path)),
        })
        .setup(move |app| {
            // 移动端使用 Tauri 提供的 app_config_dir；桌面端沿用 ikb_core 路径逻辑
            // Mobile: use Tauri's app_config_dir; Desktop: use ikb_core's platform path
            let is_mobile = cfg!(target_os = "android") || cfg!(target_os = "ios");
            let config_path = if is_mobile {
                let dir = app
                    .path()
                    .app_config_dir()
                    .unwrap_or_else(|_| PathBuf::from("."));
                if !dir.exists() {
                    let _ = std::fs::create_dir_all(&dir);
                }
                dir.join("config.yml")
            } else {
                ikb_core::paths::default_config_path()
            };

            let state = app.state::<AppState>();
            {
                let path_lock = Arc::clone(&state.config_path);
                let cfg_clone = Arc::clone(&config_for_setup);
                tauri::async_runtime::block_on(async move {
                    let loaded = Config::load_from_path(&config_path).ok();
                    *path_lock.lock().await = config_path;
                    if let Some(cfg) = loaded {
                        *cfg_clone.lock().await = Arc::new(cfg);
                    }
                });
            }

            let handle = app.handle().to_owned();
            let runtime = Arc::clone(&runtime_for_logs);
            tauri::async_runtime::spawn(async move {
                let mut rx = runtime.subscribe_logs().await;
                loop {
                    let rec = rx.recv().await;
                    match rec {
                        Ok(v) => {
                            let _ = handle.emit("ikb://log", v);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            get_config_meta,
            save_config,
            save_config_with_comments,
            save_raw_yaml,
            test_ikuai_login,
            test_github_proxy,
            runtime_status,
            runtime_run_once,
            runtime_cron_start,
            runtime_cron_stop,
            runtime_stop,
            runtime_clean,
            runtime_tail_logs,
            fetch_remote_config,
            fetch_github_releases,
            diagnostics_report,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("[ERR:启动失败] Failed to run tauri application: {}", e);
        });
}
