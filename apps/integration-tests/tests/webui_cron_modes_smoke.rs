// 覆盖点：
// 1) CLI `-r cron` 启动时先执行一次再进入定时状态；
// 2) CLI `-r cronAft` 启动时先进入等待，再由 cron 触发执行；
// 3) WebUI `/api/runtime/status` 与 `/api/runtime/cron/stop` 的状态流转。
// Coverage:
// 1) CLI `-r cron` runs once before scheduling.
// 2) CLI `-r cronAft` waits first, then runs by cron trigger.
// 3) WebUI runtime status and cron-stop state transitions.
mod common;

use std::time::Duration;

use common::{
    TestHarness, http_get_json, http_post_json_expect_status, render_test_config,
    reserve_local_port, wait_for_condition, wait_for_webui_ready,
};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
struct RuntimeStatus {
    cron_running: bool,
    cron_expr: String,
    module: String,
    last_run_at: String,
    next_run_at: String,
}

const CRON_EXPR: &str = "*/1 * * * * *";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webui_cron_modes_smoke() -> Result<(), String> {
    let harness = TestHarness::start("webui_cron_modes_smoke").await?;
    common::print_failure_hint("webui_cron_modes_smoke", harness.artifact_dir());

    harness
        .fixture()
        .set_text("/webui-cron/ipv4.txt", "203.0.113.1\n203.0.113.2\n");

    verify_cron_mode(
        &harness,
        "cron",
        "webui-cron-mode.yml",
        reserve_local_port()?,
    )
    .await?;
    verify_cron_mode(
        &harness,
        "cronAft",
        "webui-cronaft-mode.yml",
        reserve_local_port()?,
    )
    .await?;
    Ok(())
}

async fn verify_cron_mode(
    harness: &TestHarness,
    run_mode: &str,
    file_name: &str,
    port: u16,
) -> Result<(), String> {
    let webui_user = "cronuser";
    let webui_pass = "cronpass";
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
                "  - tag: CronFlow\n",
                "    url: \"{}\"\n",
                "stream-ipport:\n",
                "  - type: \"1\"\n",
                "    opt-tagname: CronRoute\n",
                "    interface: \"\"\n",
                "    nexthop: 192.168.1.2\n",
                "    src-addr: 192.168.66.10-192.168.66.20\n",
                "    src-addr-opt-ipgroup: \"\"\n",
                "    ip-group: CronFlow\n",
                "    mode: 0\n",
                "    ifaceband: 0\n"
            ),
            port,
            webui_user,
            webui_pass,
            harness.fixture().url("/webui-cron/ipv4.txt"),
        ),
    )
    .replacen("cron: \"\"\n", &format!("cron: \"{}\"\n", CRON_EXPR), 1)
    .replacen("AddWait: 10ms\n", "AddWait: 50ms\n", 1);
    let config_path = harness.write_config(file_name, &config)?;
    let config_path_str = config_path.to_string_lossy().to_string();

    let _server = harness.spawn_cli_background(
        &format!("webui-{run_mode}-mode"),
        &["-c", &config_path_str, "-r", run_mode, "-m", "ipgroup"],
    )?;

    let base_url = format!("http://127.0.0.1:{port}");
    let auth = Some((webui_user, webui_pass));
    wait_for_webui_ready(&base_url, auth, Duration::from_secs(30)).await?;

    wait_for_condition(
        &format!("{run_mode} enters cron-running state"),
        Duration::from_secs(5),
        Duration::from_millis(100),
        || {
            let url = format!("{base_url}/api/runtime/status");
            async move {
                let status: RuntimeStatus = http_get_json(&url, auth).await?;
                Ok(status.cron_running
                    && status.cron_expr == CRON_EXPR
                    && !status.module.is_empty())
            }
        },
    )
    .await?;

    let initial: RuntimeStatus =
        http_get_json(&format!("{base_url}/api/runtime/status"), auth).await?;
    assert!(initial.cron_running);
    assert_eq!(initial.cron_expr, CRON_EXPR);
    assert_eq!(initial.module, "ipgroup");
    assert!(
        !initial.next_run_at.is_empty(),
        "cron mode should expose next_run_at"
    );

    if run_mode == "cron" {
        assert!(
            !initial.last_run_at.is_empty(),
            "cron should perform one immediate run before entering schedule"
        );
    } else {
        assert!(
            initial.last_run_at.is_empty(),
            "cronAft should not have run before the first schedule fires"
        );
    }

    wait_for_condition(
        &format!("{run_mode} observes a scheduled run"),
        Duration::from_secs(4),
        Duration::from_millis(150),
        || {
            let url = format!("{base_url}/api/runtime/status");
            async move {
                let status: RuntimeStatus = http_get_json(&url, auth).await?;
                Ok(!status.last_run_at.is_empty())
            }
        },
    )
    .await?;

    let stop_status = http_post_json_expect_status(
        &format!("{base_url}/api/runtime/cron/stop"),
        auth,
        &json!({}),
    )
    .await?;
    assert_eq!(stop_status, reqwest::StatusCode::OK);

    wait_for_condition(
        &format!("{run_mode} stops cron"),
        Duration::from_secs(5),
        Duration::from_millis(100),
        || {
            let url = format!("{base_url}/api/runtime/status");
            async move {
                let status: RuntimeStatus = http_get_json(&url, auth).await?;
                Ok(!status.cron_running && status.next_run_at.is_empty())
            }
        },
    )
    .await?;

    Ok(())
}
