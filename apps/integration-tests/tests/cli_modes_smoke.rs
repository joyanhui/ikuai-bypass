// 覆盖点：
// 1) CLI 参数错误分支（invalid -r / invalid -m / clean 缺 tag / -r web 已移除）；
// 2) once 别名模式（-r 1）；
// 3) cronAft 启动路径。
// Coverage:
// 1) CLI invalid-arg branches.
// 2) once alias mode (-r 1).
// 3) cronAft startup path.
mod common;

use common::{TestHarness, assert_stderr_contains, assert_stdout_contains, render_test_config};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cli_modes_smoke() -> Result<(), String> {
    let harness = TestHarness::start("cli_modes_smoke").await?;
    common::print_failure_hint("cli_modes_smoke", harness.artifact_dir());

    harness
        .fixture()
        .set_text("/modes/isp.txt", "1.1.1.0/24\n2.2.2.0/24\n");
    harness
        .fixture()
        .set_text("/modes/domain.txt", "foo.example\nbar.example\n");
    harness
        .fixture()
        .set_text("/modes/ipv4.txt", "8.8.8.8\n9.9.9.0/24\n");

    let config = render_test_config(
        harness.base_url(),
        harness.username(),
        harness.password(),
        &format!(
            concat!(
                "custom-isp:\n",
                "  - tag: ModesIsp\n",
                "    url: \"{}\"\n",
                "stream-domain:\n",
                "  - interface: wan2\n",
                "    src-addr: 192.168.1.10-192.168.1.20\n",
                "    src-addr-opt-ipgroup: \"\"\n",
                "    url: \"{}\"\n",
                "    tag: ModesDom\n",
                "ip-group:\n",
                "  - tag: Modes4\n",
                "    url: \"{}\"\n"
            ),
            harness.fixture().url("/modes/isp.txt"),
            harness.fixture().url("/modes/domain.txt"),
            harness.fixture().url("/modes/ipv4.txt"),
        ),
    );
    let config_path = harness.write_config("cli-modes.yml", &config)?;
    let config_path_str = config_path.to_string_lossy().to_string();

    let invalid_r = harness.run_cli_expect_failure(&["-c", &config_path_str, "-r", "invalid-mode"])?;
    assert_stderr_contains(&invalid_r, "Invalid -r parameter")?;

    let removed_web = harness.run_cli_expect_failure(&["-c", &config_path_str, "-r", "web"])?;
    assert_stderr_contains(&removed_web, "-r web 已移除")?;

    let invalid_m = harness.run_cli_expect_failure(&[
        "-c",
        &config_path_str,
        "-r",
        "once",
        "-m",
        "invalid-module",
    ])?;
    assert_stderr_contains(&invalid_m, "invalid -m parameter")?;

    let clean_missing_tag = harness.run_cli_expect_failure(&["-c", &config_path_str, "-r", "clean"])?;
    assert_stderr_contains(&clean_missing_tag, "Clean mode requires -tag")?;

    let once_alias = harness.run_cli_expect_success(&[
        "-c",
        &config_path_str,
        "-r",
        "1",
        "-m",
        "ii",
    ])?;
    assert_stdout_contains(&once_alias, "[END:运行完毕]")?;

    let cron_aft = harness.run_cli_expect_success(&[
        "-c",
        &config_path_str,
        "-r",
        "cronAft",
        "-m",
        "ii",
    ])?;
    assert_stdout_contains(&cron_aft, "CronAft mode")?;

    Ok(())
}
