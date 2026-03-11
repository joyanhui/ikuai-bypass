use std::sync::Arc;
use std::path::PathBuf;

use ikb_core::config::Config;
use ikb_core::runtime::RuntimeService;
use tauri::Emitter;

#[tauri::command]
async fn get_config(state: tauri::State<'_, AppState>) -> Result<Config, String> {
    Ok(state.config.lock().await.clone())
}

#[derive(serde::Serialize)]
struct ConfigMeta {
    #[serde(flatten)]
    config: Config,
    conf_path: String,
    top_level_comments: std::collections::BTreeMap<String, String>,
    item_comments: std::collections::BTreeMap<String, String>,
    webui_comments: std::collections::BTreeMap<String, String>,
    max_number_of_one_records_comments: std::collections::BTreeMap<String, String>,
}

#[tauri::command]
async fn get_config_meta(state: tauri::State<'_, AppState>) -> Result<ConfigMeta, String> {
    let cfg = state.config.lock().await.clone();
    Ok(ConfigMeta {
        config: cfg,
        conf_path: state.config_path.to_string_lossy().to_string(),
        top_level_comments: ikb_core::config::top_level_comments(),
        item_comments: ikb_core::config::item_comments(),
        webui_comments: ikb_core::config::webui_comments(),
        max_number_of_one_records_comments: ikb_core::config::max_number_of_one_records_comments(),
    })
}

#[tauri::command]
async fn save_config(state: tauri::State<'_, AppState>, config: Config) -> Result<(), String> {
    let allow = state.config.lock().await.webui.enable_update;
    if !allow {
        return Err("Forbidden: Online update is disabled in configuration".to_string());
    }
    if let Err(e) = config.save_to_path(&state.config_path) {
        return Err(format!("Failed to save config: {}", e));
    }

    let new_cron = config.cron.clone();
    *state.config.lock().await = config;
    state.runtime.set_defaults(None, Some(new_cron)).await;
    Ok(())
}

#[tauri::command]
async fn save_config_with_comments(
    state: tauri::State<'_, AppState>,
    config: Config,
    with_comments: bool,
) -> Result<(), String> {
    let allow = state.config.lock().await.webui.enable_update;
    if !allow {
        return Err("Forbidden: Online update is disabled in configuration".to_string());
    }
    if let Err(e) = config.save_to_path_with_comments(&state.config_path, with_comments) {
        return Err(format!("Failed to save config: {}", e));
    }
    let new_cron = config.cron.clone();
    *state.config.lock().await = config;
    state.runtime.set_defaults(None, Some(new_cron)).await;
    Ok(())
}

#[tauri::command]
async fn runtime_status(state: tauri::State<'_, AppState>) -> Result<ikb_core::runtime::RuntimeStatus, String> {
    Ok(state.runtime.status())
}

#[tauri::command]
async fn runtime_run_once(state: tauri::State<'_, AppState>, module: String) -> Result<bool, String> {
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
async fn runtime_tail_logs(
    state: tauri::State<'_, AppState>,
    tail: Option<usize>,
) -> Result<Vec<ikb_core::logger::LogRecord>, String> {
    Ok(state.runtime.tail_logs(tail.unwrap_or(200)).await)
}

pub struct AppState {
    config: Arc<tokio::sync::Mutex<Config>>,
    runtime: Arc<RuntimeService>,
    config_path: PathBuf,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config_path = ikb_core::paths::default_config_path();
    let cfg = ikb_core::config::Config::load_from_path(&config_path).unwrap_or_else(|_| {
        Config {
            ikuai_url: String::new(),
            username: String::new(),
            password: String::new(),
            cron: String::new(),
            add_err_retry_wait: std::time::Duration::from_secs(0),
            add_wait: std::time::Duration::from_secs(0),
            github_proxy: String::new(),
            custom_isp: Vec::new(),
            stream_domain: Vec::new(),
            ip_group: Vec::new(),
            ipv6_group: Vec::new(),
            stream_ipport: Vec::new(),
            webui: Default::default(),
            max_number_of_one_records: Default::default(),
        }
    });

    let config = Arc::new(tokio::sync::Mutex::new(cfg));
    let runtime = Arc::new(RuntimeService::new(
        Arc::clone(&config),
        String::new(),
        String::new(),
        "ispdomain".to_string(),
    ));

    let runtime_for_logs = Arc::clone(&runtime);

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .manage(AppState {
            config,
            runtime,
            config_path,
        })
        .setup(move |app| {
            let handle = app.handle().clone();
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
            runtime_status,
            runtime_run_once,
            runtime_cron_start,
            runtime_cron_stop,
            runtime_tail_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
