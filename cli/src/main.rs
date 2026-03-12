use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::io::IsTerminal;

use clap::Parser;
use cron::Schedule;

use ikb_cli::{normalize_cron_expr, normalize_go_style_args};

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

    match args.run_mode.as_str() {
        "web" => {
            println!("[MODE:运行模式] WebUI mode - starting web service");
            let port = config.blocking_lock().webui.port.trim().to_string();
            let port = if port.is_empty() { "8080".to_string() } else { port };
            let server = web::start_web_server(config_path.to_path_buf(), Arc::clone(&config), port);
            if let Err(e) = server {
                eprintln!("[ERR:启动失败] WebUI Server failed to start, port might be occupied: {}", e);
                std::process::exit(1);
            }
            wait_until_stopped(&stop);
        }
        "cron" => {
            println!("[MODE:运行模式] Cron mode - executing once then entering scheduled mode");
            let cfg_guard = config.blocking_lock();
            match ikb_core::session::resolve_login_params(&cfg_guard, &args.ikuai_login_info) {
                Ok(p) => {
                    if p.source == ikb_core::session::LoginSource::Cli {
                        println!("[AUTH:登录认证] Logging in using command line parameters");
                    } else if p.source == ikb_core::session::LoginSource::Gateway {
                        println!(
                            "[SYS:网关检测] Using default gateway address: {}",
                            p.base_url
                        );
                    }
                }
                Err(_) => {
                    eprintln!("[AUTH:登录认证] Command line parameter format error, please use -login http://ip,username,password");
                    std::process::exit(2);
                }
            }
            if cfg_guard.webui.enable {
                let port = cfg_guard.webui.port.trim().to_string();
                let port = if port.is_empty() { "8080".to_string() } else { port };
                if let Err(e) = web::start_web_server(config_path.to_path_buf(), Arc::clone(&config), port) {
                    eprintln!("[ERR:启动失败] WebUI Server failed to start, port might be occupied: {}", e);
                    std::process::exit(1);
                }
            }
            if cfg_guard.cron.trim().is_empty() {
                println!("[CRON:定时任务] Cron configuration is empty, exiting...");
                return;
            }

            let cron_expr = cfg_guard.cron.to_string();
            drop(cfg_guard);
            run_update_once(&config, &args.ikuai_login_info, &args.module);
            run_cron_loop(&config, &args.ikuai_login_info, &cron_expr, &args.module, &stop);
        }
        "cronAft" => {
            println!("[MODE:运行模式] CronAft mode - scheduled execution only");
            let cfg_guard = config.blocking_lock();
            match ikb_core::session::resolve_login_params(&cfg_guard, &args.ikuai_login_info) {
                Ok(p) => {
                    if p.source == ikb_core::session::LoginSource::Cli {
                        println!("[AUTH:登录认证] Logging in using command line parameters");
                    } else if p.source == ikb_core::session::LoginSource::Gateway {
                        println!(
                            "[SYS:网关检测] Using default gateway address: {}",
                            p.base_url
                        );
                    }
                }
                Err(_) => {
                    eprintln!("[AUTH:登录认证] Command line parameter format error, please use -login http://ip,username,password");
                    std::process::exit(2);
                }
            }
            if cfg_guard.webui.enable {
                let port = cfg_guard.webui.port.trim().to_string();
                let port = if port.is_empty() { "8080".to_string() } else { port };
                if let Err(e) = web::start_web_server(config_path.to_path_buf(), Arc::clone(&config), port) {
                    eprintln!("[ERR:启动失败] WebUI Server failed to start, port might be occupied: {}", e);
                    std::process::exit(1);
                }
            }
            if cfg_guard.cron.trim().is_empty() {
                println!("[CRON:定时任务] Cron configuration is empty, exiting...");
                return;
            }

            let cron_expr = cfg_guard.cron.to_string();
            drop(cfg_guard);
            run_cron_loop(&config, &args.ikuai_login_info, &cron_expr, &args.module, &stop);
        }
        "nocron" | "once" | "1" => {
            run_update_once(&config, &args.ikuai_login_info, &args.module);
            println!("[END:运行完毕] Once mode execution completed, exiting...");
        }
        "clean" => {
            if args.clean_tag.trim().is_empty() {
                eprintln!("[ERR:参数错误] Clean mode requires -tag (or cleanAll)");
                std::process::exit(2);
            }
            println!("[MODE:运行模式] Clean mode");
            if args.clean_tag.trim() == ikb_core::ikuai::CLEAN_MODE_ALL {
                println!(
                    "[CLEAN:清理范围] Clearing all rules with prefix IKB (includes legacy notes)"
                );
            } else {
                println!(
                    "[CLEAN:清理范围] Clearing rules with TagName or Name: {}",
                    args.clean_tag
                );
            }

            let cfg_guard = config.blocking_lock();
            let params = match ikb_core::session::resolve_login_params(&cfg_guard, &args.ikuai_login_info)
            {
                Ok(p) => p,
                Err(_) => {
                    eprintln!("[AUTH:登录认证] Command line parameter format error, please use -login http://ip,username,password");
                    std::process::exit(2);
                }
            };
            let rt = match tokio::runtime::Runtime::new() {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("[ERR:启动失败] Failed to create tokio runtime: {}", e);
                    std::process::exit(1);
                }
            };
            let clean_tag = args.clean_tag.to_string();
            rt.block_on(async move {
                let api = match ikb_core::ikuai::IKuaiClient::new(params.base_url.to_string()) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("[LOGIN:登录失败] Failed to build iKuai client: {}", e);
                        std::process::exit(1);
                    }
                };
                if let Err(e) = api.login(&params.username, &params.password).await {
                    eprintln!("[LOGIN:登录失败] Failed to login to iKuai: {}", e);
                    std::process::exit(1);
                }

                if let Err(e) = ikb_core::ikuai::custom_isp::del_custom_isp_all(&api, &clean_tag).await {
                    eprintln!("[CLEAN:清理失败] Failed to remove old custom ISP for tag {}: {}", clean_tag, e);
                    std::process::exit(1);
                }
                if let Err(e) = ikb_core::ikuai::stream_domain::del_stream_domain_all(&api, &clean_tag).await {
                    eprintln!("[CLEAN:清理失败] Failed to remove old domain streaming for tag {}: {}", clean_tag, e);
                    std::process::exit(1);
                }
                if let Err(e) = ikb_core::ikuai::ip_group::del_ikuai_bypass_ip_group(&api, &clean_tag).await {
                    eprintln!("[CLEAN:清理失败] Failed to remove old IP group for tag {}: {}", clean_tag, e);
                    std::process::exit(1);
                }
                if let Err(e) = ikb_core::ikuai::ipv6_group::del_ikuai_bypass_ipv6_group(&api, &clean_tag).await {
                    eprintln!("[CLEAN:清理失败] Failed to remove old IPv6 group for tag {}: {}", clean_tag, e);
                    std::process::exit(1);
                }
                if let Err(e) = ikb_core::ikuai::stream_ipport::del_ikuai_bypass_stream_ipport(&api, &clean_tag).await {
                    eprintln!("[CLEAN:清理失败] Failed to remove old port streaming for tag {}: {}", clean_tag, e);
                    std::process::exit(1);
                }

                println!("[CLEAN:操作成功] Cleared rules with tag: {}", clean_tag);
            });
        }
        other => {
            eprintln!("[ERR:参数错误] Invalid -r parameter: {}", other);
            std::process::exit(2);
        }
    }
}

fn run_update_once(cfg: &Arc<tokio::sync::Mutex<ikb_core::config::Config>>, cli_login: &str, module: &str) {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[ERR:启动失败] Failed to create tokio runtime: {}", e);
            std::process::exit(1);
        }
    };
    let use_color = std::io::stdout().is_terminal();
    let renderer = ikb_core::logger::Renderer::new(use_color);
    let sink: ikb_core::logger::LogSink = std::sync::Arc::new(move |rec| {
        println!("{}", renderer.render(&rec));
    });
    let res = rt.block_on(async {
        let cfg_guard = cfg.lock().await;
        ikb_core::update::run_update_by_module(&cfg_guard, cli_login, module, sink).await
    });
    if let Err(e) = res {
        eprintln!("[UPDATE:更新失败] {}", e);
    }
}

fn wait_until_stopped(stop: &Arc<AtomicBool>) {
    while !stop.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(200));
    }
}

fn run_cron_loop(
    cfg: &Arc<tokio::sync::Mutex<ikb_core::config::Config>>,
    cli_login: &str,
    expr: &str,
    module: &str,
    stop: &Arc<AtomicBool>,
) {
    let expr = match normalize_cron_expr(expr) {
        Ok(v) => v,
        Err(_) => {
            eprintln!("[CRON:定时任务] Failed to start scheduled task: Invalid cron expression.");
            std::process::exit(1);
        }
    };

    let schedule = match Schedule::from_str(&expr) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[CRON:定时任务] Failed to start scheduled task: {}", e);
            std::process::exit(1);
        }
    };

    println!("[CRON:定时任务] Scheduled task started: {}", expr);

    let mut upcoming = schedule.upcoming(chrono::Local);
    while !stop.load(Ordering::SeqCst) {
        let next = match upcoming.next() {
            Some(t) => t,
            None => return,
        };

        while !stop.load(Ordering::SeqCst) {
            let now = chrono::Local::now();
            if next <= now {
                break;
            }
            let wait = (next - now).to_std().unwrap_or(Duration::from_millis(100));
            let wait = wait.min(Duration::from_secs(1));
            std::thread::sleep(wait);
        }

        if stop.load(Ordering::SeqCst) {
            return;
        }

        run_update_once(cfg, cli_login, module);
    }
}
