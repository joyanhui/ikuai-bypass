// 覆盖点：
// 1) CLI 内置 WebUI 的 BasicAuth 认证；
// 2) /api/config 与 /api/save-raw 的真实配置读写链路；
// 3) /api/runtime/run-once 与 /api/runtime/stop 的任务启动/停止状态流转。
// Coverage:
// 1) BasicAuth for the built-in CLI WebUI.
// 2) Real config read/write flow via /api/config and /api/save-raw.
// 3) Task start/stop state transitions through /api/runtime/run-once and /api/runtime/stop.
mod common;

use std::fs;
use std::time::Duration;

use common::{
    TestHarness, http_get_json, http_get_status, http_post_json, http_post_json_expect_status,
    render_test_config, reserve_local_port, wait_for_condition, wait_for_webui_ready,
};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
struct RuntimeStatus {
    running: bool,
    cron_running: bool,
    cron_expr: String,
    module: String,
    last_run_at: String,
    next_run_at: String,
}

#[derive(Debug, Deserialize)]
struct ConfigMetaResp {
    conf_path: String,
    raw_yaml: String,
}

#[derive(Debug, Deserialize)]
struct StartedResp {
    started: bool,
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webui_auth_runtime_config_smoke() -> Result<(), String> {
    let harness = TestHarness::start("webui_auth_runtime_config_smoke").await?;
    common::print_failure_hint("webui_auth_runtime_config_smoke", harness.artifact_dir());

    harness.fixture().set_text(
        "/webui-flow/ipv4.txt",
        concat!(
            "198.18.0.1\n",
            "198.18.0.2\n",
            "198.18.0.3\n",
            "198.18.0.4\n",
            "198.18.0.5\n"
        ),
    );

    let port = reserve_local_port()?;
    let webui_user = "webuser";
    let webui_pass = "webpass";
    let config = render_test_config(
        harness.base_url(),
        harness.username(),
        harness.password(),
        &format!(
            concat!(
                "webui:\n",
                "  enable: true\n",
                "  port: \"{}\"\n",
                "  user: \"{}\"\n",
                "  pass: \"{}\"\n",
                "  cdn-prefix: \"https://cdn.jsdelivr.net/npm\"\n",
                "ip-group:\n",
                "  - tag: WebUiFlow\n",
                "    url: \"{}\"\n",
                "stream-ipport:\n",
                "  - type: \"1\"\n",
                "    opt-tagname: WebUiRoute\n",
                "    interface: \"\"\n",
                "    nexthop: 192.168.1.2\n",
                "    src-addr: 192.168.77.10-192.168.77.20\n",
                "    src-addr-opt-ipgroup: \"\"\n",
                "    ip-group: WebUiFlow\n",
                "    mode: 0\n",
                "    ifaceband: 0\n",
                "MaxNumberOfOneRecords:\n",
                "  Isp: 5000\n",
                "  Ipv4: 1\n",
                "  Ipv6: 1000\n",
                "  Domain: 5000\n"
            ),
            port,
            webui_user,
            webui_pass,
            harness.fixture().url("/webui-flow/ipv4.txt"),
        ),
    )
    .replacen("AddWait: 10ms\n", "AddWait: 700ms\n", 1);
    let config_path = harness.write_config("webui-flow.yml", &config)?;
    let config_path_str = config_path.to_string_lossy().to_string();

    let _server = harness.spawn_cli_background(
        "webui-auth-runtime-config",
        &["-c", &config_path_str, "-r", "cronAft", "-m", "ipgroup"],
    )?;

    let base_url = format!("http://127.0.0.1:{port}");
    let auth = Some((webui_user, webui_pass));
    wait_for_webui_ready(&base_url, auth, Duration::from_secs(30)).await?;

    let unauth_status = http_get_status(&format!("{base_url}/api/config"), None).await?;
    assert_eq!(unauth_status, reqwest::StatusCode::UNAUTHORIZED);

    let meta: ConfigMetaResp = http_get_json(&format!("{base_url}/api/config"), auth).await?;
    assert!(meta.conf_path.ends_with("webui-flow.yml"));
    assert!(meta.raw_yaml.contains("webui:"));
    assert!(meta.raw_yaml.contains("user: \"webuser\""));

    let updated_yaml = meta
        .raw_yaml
        .replace("cron: \"\"", "cron: \"*/2 * * * * *\"")
        .replace("nexthop: 192.168.1.2", "nexthop: 192.168.1.99");
    let save_status = http_post_json_expect_status(
        &format!("{base_url}/api/save-raw"),
        auth,
        &json!({
            "yaml_text": updated_yaml,
            "with_comments": true,
        }),
    )
    .await?;
    assert_eq!(save_status, reqwest::StatusCode::OK);

    let saved_text = fs::read_to_string(&config_path).map_err(|e| {
        format!(
            "failed to read saved config '{}': {e}",
            config_path.display()
        )
    })?;
    assert!(saved_text.contains("cron: \"*/2 * * * * *\""));
    assert!(saved_text.contains("nexthop: 192.168.1.99"));

    let status: RuntimeStatus =
        http_get_json(&format!("{base_url}/api/runtime/status"), auth).await?;
    assert_eq!(status.cron_expr, "*/2 * * * * *");
    assert_eq!(status.module, "ipgroup");
    assert!(!status.running);
    assert!(!status.cron_running);
    assert!(status.last_run_at.is_empty());
    assert!(status.next_run_at.is_empty());

    let started: StartedResp = http_post_json(
        &format!("{base_url}/api/runtime/run-once"),
        auth,
        &json!({ "module": "ipgroup" }),
    )
    .await?;
    assert!(started.started, "run-once should start a new task");

    wait_for_condition(
        "webui run-once enters running state",
        Duration::from_secs(5),
        Duration::from_millis(100),
        || {
            let url = format!("{base_url}/api/runtime/status");
            async move {
                let status: RuntimeStatus = http_get_json(&url, auth).await?;
                Ok(status.running)
            }
        },
    )
    .await?;

    let stop_status =
        http_post_json_expect_status(&format!("{base_url}/api/runtime/stop"), auth, &json!({}))
            .await?;
    assert_eq!(stop_status, reqwest::StatusCode::OK);

    wait_for_condition(
        "webui run-once stops",
        Duration::from_secs(5),
        Duration::from_millis(100),
        || {
            let url = format!("{base_url}/api/runtime/status");
            async move {
                let status: RuntimeStatus = http_get_json(&url, auth).await?;
                Ok(!status.running)
            }
        },
    )
    .await?;

    let logs: Vec<Value> =
        http_get_json(&format!("{base_url}/api/runtime/logs?tail=50"), auth).await?;
    let stop_seen = logs
        .iter()
        .any(|item| item.get("tag").and_then(Value::as_str) == Some("TASK:任务停止"));
    assert!(stop_seen, "runtime stop should emit TASK:任务停止 log");

    Ok(())
}
