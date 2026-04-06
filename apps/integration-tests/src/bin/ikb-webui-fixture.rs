use std::path::PathBuf;
use std::time::Duration;

use ikb_integration_tests::webui_fixture::WebUiFixture;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut env_file: Option<PathBuf> = None;
    let mut json_file: Option<PathBuf> = None;
    let mut artifact_root: Option<PathBuf> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--env-file" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--env-file requires a path".to_string())?;
                env_file = Some(PathBuf::from(value));
            }
            "--json-file" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--json-file requires a path".to_string())?;
                json_file = Some(PathBuf::from(value));
            }
            "--artifact-root" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--artifact-root requires a path".to_string())?;
                artifact_root = Some(PathBuf::from(value));
            }
            "--help" | "-h" => {
                print_usage();
                return Ok(());
            }
            other => {
                return Err(format!("unsupported arg '{other}'"));
            }
        }
    }

    let mut fixture = WebUiFixture::start(artifact_root).await?;
    if let Some(path) = env_file.as_deref() {
        fixture.write_env_file(path)?;
    }
    if let Some(path) = json_file.as_deref() {
        fixture.write_manifest_json(path)?;
    }

    let manifest = fixture.manifest();
    println!("fixture ready: {}", manifest.webui_base_url);
    println!("artifact dir: {}", manifest.artifact_dir);
    println!("config path: {}", manifest.config_path);

    loop {
        tokio::select! {
            _ = shutdown_signal() => {
                break;
            }
            _ = tokio::time::sleep(Duration::from_millis(300)) => {
                if let Some(status) = fixture.try_wait_cli()? {
                    return Err(format!(
                        "browser fixture cli exited unexpectedly: {status}\nstdout log: {}\nstderr log: {}",
                        manifest.cli_stdout_log_path,
                        manifest.cli_stderr_log_path,
                    ));
                }
            }
        }
    }

    Ok(())
}

#[cfg(unix)]
async fn shutdown_signal() {
    use tokio::signal::unix::{SignalKind, signal};

    let mut sigterm = signal(SignalKind::terminate()).expect("failed to register SIGTERM handler");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = sigterm.recv() => {}
    }
}

#[cfg(not(unix))]
async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

fn print_usage() {
    println!(
        "Usage: ikb-webui-fixture [--env-file PATH] [--json-file PATH] [--artifact-root PATH]"
    );
}
