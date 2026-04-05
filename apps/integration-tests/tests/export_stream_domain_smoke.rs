// 覆盖点：
// 1) CLI 导出模式 exportDomainSteamToTxt；
// 2) 导出文件命名和内容过滤（注释/空行/非法域名）；
// 3) 导出完成 banner。
// Coverage:
// 1) CLI exportDomainSteamToTxt mode.
// 2) Output file naming and domain filtering.
// 3) Export completion banner.
mod common;

use std::fs;

use common::{TestHarness, assert_stdout_contains, render_test_config};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn export_stream_domain_smoke() -> Result<(), String> {
    let harness = TestHarness::start("export_stream_domain_smoke").await?;
    common::print_failure_hint("export_stream_domain_smoke", harness.artifact_dir());

    harness.fixture().set_text(
        "/export/domains.txt",
        "foo.example\n# comment\n_bad.example\nbar.example\n\n",
    );

    let config = render_test_config(
        harness.base_url(),
        harness.username(),
        harness.password(),
        &format!(
            concat!(
                "stream-domain:\n",
                "  - interface: wan2\n",
                "    src-addr: 192.168.1.10-192.168.1.20\n",
                "    src-addr-opt-ipgroup: \"\"\n",
                "    url: \"{}\"\n",
                "    tag: ExportTag\n"
            ),
            harness.fixture().url("/export/domains.txt")
        ),
    );
    let config_path = harness.write_config("export.yml", &config)?;
    let config_path_str = config_path.to_string_lossy().to_string();

    let export_dir = harness.artifact_dir().join("export-out");
    let export_dir_str = export_dir.to_string_lossy().to_string();

    let output = harness.run_cli_expect_success(&[
        "-c",
        &config_path_str,
        "-r",
        "exportDomainSteamToTxt",
        "-exportPath",
        &export_dir_str,
    ])?;
    assert_stdout_contains(&output, "[END:导出完毕]")?;

    let file = export_dir.join("stream-domain_wan2_ExportTag.txt");
    let content = fs::read_to_string(&file)
        .map_err(|e| format!("failed to read export file '{}': {e}", file.display()))?;
    assert!(content.contains("foo.example\n"));
    assert!(content.contains("bar.example\n"));
    assert!(!content.contains("_bad.example"));

    Ok(())
}
