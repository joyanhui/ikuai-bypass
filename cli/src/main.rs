use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::io::IsTerminal;

use clap::Parser;

use ikb_cli::normalize_go_style_args;
use ikb_core::runtime::RuntimeService;

mod web;

#[derive(Debug, Parser)]
#[command(name = "ikuai-bypass")]
struct Args {
    #[arg(short = 'c', long = "c")]
    config_path: Option<PathBuf>,

    #[arg(short = 'r', long = "r", default_value = "cron")]
    run_mode: String,

    #[arg(short = 'm', long = "m", default_value = "ispdomain")]
    module: String,

    #[arg(long = "tag", default_value = "")]
    clean_tag: String,

    #[arg(long = "exportPath", default_value = "/tmp")]
    export_path: String,

    #[arg(long = "login", default_value = "")]
    ikuai_login_info: String,

    #[arg(long = "isIpGroupNameAddRandomSuff", default_value = "1")]
    is_ip_group_name_add_random_suff: String,
}

fn main() {
    let raw_args: Vec<String> = std::env::args().collect();
    let normalized = normalize_go_style_args(&raw_args);
    let args = Args::parse_from(normalized);

    let config_path = args
        .config_path
        .clone()
        .unwrap_or_else(ikb_core::paths::default_config_path);

    println!(
        "[START:启动程序] Run mode: {}, Config path: '{}'",
        args.run_mode,
        config_path.display()
    );

    let cfg = match ikb_core::config::Config::load_from_path(&config_path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[CONF:配置读取] Failed to read configuration file: {}", e);
            std::process::exit(1);
        }
    };
    let config = Arc::new(tokio::sync::Mutex::new(cfg));

    if let Err(e) = ikb_core::runner::validate_module(&args.module) {
        eprintln!("[ERR:参数错误] {}", e);
        std::process::exit(2);
    }

    let stop = Arc::new(AtomicBool::new(false));
    {
        let stop = Arc::clone(&stop);
        if let Err(e) = ctrlc::set_handler(move || {
            stop.store(true, Ordering::SeqCst);
        }) {
            eprintln!("[ERR:启动失败] Failed to set ctrlc handler: {}", e);
            std::process::exit(1);
        }
    }

    let rt = match tokio::runtime::Runtime::new() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[ERR:启动失败] Failed to create tokio runtime: {}", e);
            std::process::exit(1);
        }
    };

    let code = rt.block_on(async move { run(args, config_path, config, stop).await });
    std::process::exit(code);
}

async fn run(
    args: Args,
    config_path: PathBuf,
    config: Arc<tokio::sync::Mutex<ikb_core::config::Config>>,
    stop: Arc<AtomicBool>,
) -> i32 {
    match args.run_mode.as_str() {
        "web" => {
            println!("[MODE:运行模式] WebUI mode - starting web service");
            // web 模式是显式入口：即使配置中禁用 webui，也允许通过 WebUI 写入配置。
            // Web mode is explicit: allow config writes even if webui.enable=false.
            let port = {
                let port = config.lock().await.webui.port.trim().to_string();
                if port.is_empty() { "8080".to_string() } else { port }
            };

            let default_cron = { config.lock().await.cron.to_string() };
            let runtime = Arc::new(RuntimeService::new(
                Arc::clone(&config),
                args.ikuai_login_info.to_string(),
                default_cron,
                args.module.to_string(),
            ));
            spawn_runtime_stdout_forwarder(Arc::clone(&runtime));

            if let Err(e) = web::start_web_server(
                config_path,
                Arc::clone(&config),
                Arc::clone(&runtime),
                args.ikuai_login_info.to_string(),
                port,
                true,
            )
            .await
            {
                eprintln!("[ERR:启动失败] WebUI Server failed to start, port might be occupied: {}", e);
                return 1;
            }

            wait_until_stopped(&stop).await;
            let _ = runtime.stop_all().await;
            0
        }

        "cron" => {
            println!("[MODE:运行模式] Cron mode - executing once then entering scheduled mode");

            let (cron_expr, webui_enable, webui_port) = {
                let cfg_guard = config.lock().await;
                match ikb_core::session::resolve_login_params(&cfg_guard, &args.ikuai_login_info) {
                    Ok(p) => {
                        if p.source == ikb_core::session::LoginSource::Cli {
                            println!("[AUTH:登录认证] Logging in using command line parameters");
                        } else if p.source == ikb_core::session::LoginSource::Gateway {
                            println!("[SYS:网关检测] Using default gateway address: {}", p.base_url);
                        }
                    }
                    Err(_) => {
                        eprintln!("[AUTH:登录认证] Command line parameter format error, please use -login http://ip,username,password");
                        return 2;
                    }
                }
                let port = cfg_guard.webui.port.trim().to_string();
                let port = if port.is_empty() { "8080".to_string() } else { port };
                (cfg_guard.cron.to_string(), cfg_guard.webui.enable, port)
            };

            let runtime = Arc::new(RuntimeService::new(
                Arc::clone(&config),
                args.ikuai_login_info.to_string(),
                cron_expr.to_string(),
                args.module.to_string(),
            ));
            spawn_runtime_stdout_forwarder(Arc::clone(&runtime));

            if webui_enable {
                if let Err(e) = web::start_web_server(
                    config_path,
                    Arc::clone(&config),
                    Arc::clone(&runtime),
                    args.ikuai_login_info.to_string(),
                    webui_port,
                    false,
                )
                .await
                {
                    eprintln!("[ERR:启动失败] WebUI Server failed to start, port might be occupied: {}", e);
                    return 1;
                }
            }

            if cron_expr.trim().is_empty() {
                println!("[CRON:定时任务] Cron configuration is empty, exiting...");
                return 0;
            }

            match Arc::clone(&runtime).start_run_once(args.module.to_string()).await {
                Ok(started) => {
                    if !started {
                        println!("[TASK:任务状态] Task is already running, ignore start request");
                    } else {
                        // 与旧版 CLI 行为保持一致：cron 模式先执行一次（完成后）再进入定时等待。
                        // Keep legacy behavior: run once (finish) then start cron loop.
                        while runtime.status().running {
                            tokio::time::sleep(Duration::from_millis(200)).await;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[UPDATE:更新失败] {}", e);
                }
            }

            if let Err(e) = Arc::clone(&runtime)
                .start_cron(cron_expr.to_string(), args.module.to_string())
                .await
            {
                eprintln!("[CRON:定时任务] Failed to start scheduled task: {}", e);
                return 1;
            }

            wait_until_stopped(&stop).await;
            let _ = runtime.stop_all().await;
            0
        }

        "cronAft" => {
            println!("[MODE:运行模式] CronAft mode - scheduled execution only");

            let (cron_expr, webui_enable, webui_port) = {
                let cfg_guard = config.lock().await;
                match ikb_core::session::resolve_login_params(&cfg_guard, &args.ikuai_login_info) {
                    Ok(p) => {
                        if p.source == ikb_core::session::LoginSource::Cli {
                            println!("[AUTH:登录认证] Logging in using command line parameters");
                        } else if p.source == ikb_core::session::LoginSource::Gateway {
                            println!("[SYS:网关检测] Using default gateway address: {}", p.base_url);
                        }
                    }
                    Err(_) => {
                        eprintln!("[AUTH:登录认证] Command line parameter format error, please use -login http://ip,username,password");
                        return 2;
                    }
                }
                let port = cfg_guard.webui.port.trim().to_string();
                let port = if port.is_empty() { "8080".to_string() } else { port };
                (cfg_guard.cron.to_string(), cfg_guard.webui.enable, port)
            };

            let runtime = Arc::new(RuntimeService::new(
                Arc::clone(&config),
                args.ikuai_login_info.to_string(),
                cron_expr.to_string(),
                args.module.to_string(),
            ));
            spawn_runtime_stdout_forwarder(Arc::clone(&runtime));

            if webui_enable {
                if let Err(e) = web::start_web_server(
                    config_path,
                    Arc::clone(&config),
                    Arc::clone(&runtime),
                    args.ikuai_login_info.to_string(),
                    webui_port,
                    false,
                )
                .await
                {
                    eprintln!("[ERR:启动失败] WebUI Server failed to start, port might be occupied: {}", e);
                    return 1;
                }
            }

            if cron_expr.trim().is_empty() {
                println!("[CRON:定时任务] Cron configuration is empty, exiting...");
                return 0;
            }

            if let Err(e) = Arc::clone(&runtime)
                .start_cron(cron_expr.to_string(), args.module.to_string())
                .await
            {
                eprintln!("[CRON:定时任务] Failed to start scheduled task: {}", e);
                return 1;
            }

            wait_until_stopped(&stop).await;
            let _ = runtime.stop_all().await;
            0
        }

        "nocron" | "once" | "1" => {
            if let Err(e) = run_update_once(&config, &args.ikuai_login_info, &args.module).await {
                eprintln!("[UPDATE:更新失败] {}", e);
                return 1;
            }
            println!("[END:运行完毕] Once mode execution completed, exiting...");
            0
        }

        "clean" => {
            if args.clean_tag.trim().is_empty() {
                eprintln!("[ERR:参数错误] Clean mode requires -tag (or cleanAll)");
                return 2;
            }
            println!("[MODE:运行模式] Clean mode");
            if args.clean_tag.trim() == ikb_core::ikuai::CLEAN_MODE_ALL {
                println!("[CLEAN:清理范围] Clearing all rules with prefix IKB (includes legacy notes)");
            } else {
                println!(
                    "[CLEAN:清理范围] Clearing rules with TagName or Name: {}",
                    args.clean_tag
                );
            }

            // 避免在网络请求期间持有配置锁。
            // Avoid holding config lock while doing network requests.
            let cfg_snapshot = { config.lock().await.clone() };
            let params = match ikb_core::session::resolve_login_params(&cfg_snapshot, &args.ikuai_login_info) {
                Ok(p) => p,
                Err(_) => {
                    eprintln!("[AUTH:登录认证] Command line parameter format error, please use -login http://ip,username,password");
                    return 2;
                }
            };

            let api = match ikb_core::ikuai::IKuaiClient::new(params.base_url.to_string()) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("[LOGIN:登录失败] Failed to build iKuai client: {}", e);
                    return 1;
                }
            };
            if let Err(e) = api.login(&params.username, &params.password).await {
                eprintln!("[LOGIN:登录失败] Failed to login to iKuai: {}", e);
                return 1;
            }

            let clean_tag = args.clean_tag.to_string();
            if let Err(e) = ikb_core::ikuai::custom_isp::del_custom_isp_all(&api, &clean_tag).await {
                eprintln!("[CLEAN:清理失败] Failed to remove old custom ISP for tag {}: {}", clean_tag, e);
                return 1;
            }
            if let Err(e) = ikb_core::ikuai::stream_domain::del_stream_domain_all(&api, &clean_tag).await {
                eprintln!("[CLEAN:清理失败] Failed to remove old domain streaming for tag {}: {}", clean_tag, e);
                return 1;
            }
            if let Err(e) = ikb_core::ikuai::ip_group::del_ikuai_bypass_ip_group(&api, &clean_tag).await {
                eprintln!("[CLEAN:清理失败] Failed to remove old IP group for tag {}: {}", clean_tag, e);
                return 1;
            }
            if let Err(e) = ikb_core::ikuai::ipv6_group::del_ikuai_bypass_ipv6_group(&api, &clean_tag).await {
                eprintln!("[CLEAN:清理失败] Failed to remove old IPv6 group for tag {}: {}", clean_tag, e);
                return 1;
            }
            if let Err(e) = ikb_core::ikuai::stream_ipport::del_ikuai_bypass_stream_ipport(&api, &clean_tag).await {
                eprintln!("[CLEAN:清理失败] Failed to remove old port streaming for tag {}: {}", clean_tag, e);
                return 1;
            }

            println!("[CLEAN:操作成功] Cleared rules with tag: {}", clean_tag);
            0
        }

        other => {
            eprintln!("[ERR:参数错误] Invalid -r parameter: {}", other);
            2
        }
    }
}

fn spawn_runtime_stdout_forwarder(runtime: Arc<RuntimeService>) {
    let use_color = std::io::stdout().is_terminal();
    let renderer = ikb_core::logger::Renderer::new(use_color);
    tokio::spawn(async move {
        let mut rx = runtime.subscribe_logs().await;
        loop {
            match rx.recv().await {
                Ok(rec) => {
                    println!("{}", renderer.render(&rec));
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}

async fn run_update_once(
    cfg: &Arc<tokio::sync::Mutex<ikb_core::config::Config>>,
    cli_login: &str,
    module: &str,
) -> Result<(), String> {
    let use_color = std::io::stdout().is_terminal();
    let renderer = ikb_core::logger::Renderer::new(use_color);
    let sink: ikb_core::logger::LogSink = std::sync::Arc::new(move |rec| {
        println!("{}", renderer.render(&rec));
    });

    // 更新期间避免长时间持有配置锁：配置只读，拷贝一份用于本次任务。
    // Avoid holding config lock across awaits: clone config for this run.
    let cfg_snapshot = { cfg.lock().await.clone() };
    ikb_core::update::run_update_by_module(&cfg_snapshot, cli_login, module, sink)
        .await
        .map_err(|e| e.to_string())
}

async fn wait_until_stopped(stop: &Arc<AtomicBool>) {
    while !stop.load(Ordering::SeqCst) {
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}
