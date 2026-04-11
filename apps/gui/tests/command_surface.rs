use std::fs;
use std::path::PathBuf;

fn read_repo_file(rel: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = root.join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("failed to read '{}': {e}", path.display());
    })
}

#[test]
fn tauri_invoke_handler_includes_runtime_commands() {
    let src = read_repo_file("src/lib.rs");
    for cmd in [
        "runtime_status",
        "runtime_run_once",
        "runtime_cron_start",
        "runtime_cron_stop",
        "runtime_stop",
        "runtime_clean",
        "runtime_tail_logs",
        "get_config_meta",
        "get_embedded_default_config",
        "save_raw_yaml",
        "fetch_remote_config",
        "fetch_github_releases",
        "diagnostics_report",
    ] {
        assert!(
            src.contains(cmd),
            "tauri command '{}' should be wired in invoke handler",
            cmd
        );
    }
}

#[test]
fn tauri_config_file_exists() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = root.join("tauri.conf.json");
    assert!(path.is_file(), "missing tauri config: {}", path.display());
}
