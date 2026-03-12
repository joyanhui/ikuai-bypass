use std::sync::Arc;
use std::path::PathBuf;

use ikb_core::config::Config;
use ikb_core::runtime::RuntimeService;
use tauri::Emitter;

#[tauri::command]
async fn get_config(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let cfg = state.config.lock().await;
    serde_json::to_value(&*cfg).map_err(|e| format!("Failed to encode config: {}", e))
}

#[derive(serde::Serialize)]
struct ConfigMeta {
    #[serde(flatten)]
    config: serde_json::Value,
    conf_path: String,
    raw_yaml: String,
    top_level_comments: std::collections::BTreeMap<String, String>,
    item_comments: std::collections::BTreeMap<String, String>,
    webui_comments: std::collections::BTreeMap<String, String>,
    max_number_of_one_records_comments: std::collections::BTreeMap<String, String>,
}

#[tauri::command]
async fn get_config_meta(state: tauri::State<'_, AppState>) -> Result<ConfigMeta, String> {
    let cfg_guard = state.config.lock().await;
    let config = serde_json::to_value(&*cfg_guard)
        .map_err(|e| format!("Failed to encode config: {}", e))?;
    let raw_yaml = std::fs::read_to_string(&state.config_path).unwrap_or_default();
    Ok(ConfigMeta {
        config,
        conf_path: state.config_path.to_string_lossy().to_string(),
        raw_yaml,
        top_level_comments: ikb_core::config::top_level_comments(),
        item_comments: ikb_core::config::item_comments(),
        webui_comments: ikb_core::config::webui_comments(),
        max_number_of_one_records_comments: ikb_core::config::max_number_of_one_records_comments(),
    })
}

#[tauri::command]
async fn save_config(state: tauri::State<'_, AppState>, config: Config) -> Result<(), String> {
    if let Err(e) = config.save_to_path(&state.config_path) {
        return Err(format!("Failed to save config: {}", e));
    }

    let new_cron = config.cron.to_string();
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
    if let Err(e) = config.save_to_path_with_comments(&state.config_path, with_comments) {
        return Err(format!("Failed to save config: {}", e));
    }
    let new_cron = config.cron.to_string();
    *state.config.lock().await = config;
    state.runtime.set_defaults(None, Some(new_cron)).await;
    Ok(())
}

#[tauri::command]
async fn save_raw_yaml(
    state: tauri::State<'_, AppState>,
    yaml_text: String,
    with_comments: bool,
) -> Result<(), String> {
    let cfg = Config::validate_and_save_raw_yaml(&yaml_text, &state.config_path, with_comments)
        .map_err(|e| format!("Failed to save config: {}", e))?;
    let new_cron = cfg.cron.to_string();
    *state.config.lock().await = cfg;
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
async fn runtime_stop(state: tauri::State<'_, AppState>) -> Result<(), String> {
    Arc::clone(&state.runtime)
        .stop_all()
        .await
        .map_err(|e| e.to_string())
}

async fn run_clean(config: &Config, clean_tag: String) -> Result<(), String> {
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

#[tauri::command]
async fn runtime_clean(
    state: tauri::State<'_, AppState>,
    clean_tag: String,
) -> Result<(), String> {
    let cfg_guard = state.config.lock().await;
    run_clean(&cfg_guard, clean_tag).await
}

#[tauri::command]
async fn runtime_tail_logs(
    state: tauri::State<'_, AppState>,
    tail: Option<usize>,
) -> Result<Vec<ikb_core::logger::LogRecord>, String> {
    Ok(state.runtime.tail_logs(tail.unwrap_or(200)).await)
}

#[tauri::command]
async fn fetch_remote_config(url: String, github_proxy: String) -> Result<String, String> {
    let url = url.trim();
    if url.is_empty() {
        return Err("Remote URL is empty".to_string());
    }
    let mut final_url = url.to_string();
    let proxy = github_proxy.trim();
    if !proxy.is_empty() && url.starts_with("https://raw.githubusercontent.com/") {
        final_url = if proxy.ends_with('/') {
            format!("{}{}", proxy, url)
        } else {
            format!("{}/{}", proxy, url)
        };
    }

    let client = reqwest::Client::builder()
        .user_agent("ikb-app")
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(final_url).send().await.map_err(|e| e.to_string())?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("HTTP {}", status));
    }
    resp.text().await.map_err(|e| e.to_string())
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
            let handle = app.handle();
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
            runtime_status,
            runtime_run_once,
            runtime_cron_start,
            runtime_cron_stop,
            runtime_stop,
            runtime_clean,
            runtime_tail_logs,
            fetch_remote_config,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("[ERR:启动失败] Failed to run tauri application: {}", e);
        });
}
