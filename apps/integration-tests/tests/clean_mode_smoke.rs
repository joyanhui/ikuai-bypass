mod common;

use common::{TestHarness, assert_stderr_contains, render_test_config};
use ikb_core::ikuai;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn clean_mode_smoke() -> Result<(), String> {
    let harness = TestHarness::start("clean_mode_smoke").await?;

    harness.fixture().set_text("/clean/a-isp.txt", "10.10.10.0/24\n");
    harness.fixture().set_text("/clean/b-isp.txt", "10.10.20.0/24\n");
    harness.fixture().set_text("/clean/a-domain.txt", "alpha.example\n");
    harness.fixture().set_text("/clean/b-domain.txt", "beta.example\n");
    harness.fixture().set_text("/clean/a-ipv4.txt", "172.16.10.1\n");
    harness.fixture().set_text("/clean/b-ipv4.txt", "172.16.20.1\n");

    let config = render_test_config(
        harness.base_url(),
        harness.username(),
        harness.password(),
        &format!(
            concat!(
                "custom-isp:\n",
                "  - tag: ClnA\n",
                "    url: \"{}\"\n",
                "  - tag: ClnB\n",
                "    url: \"{}\"\n",
                "stream-domain:\n",
                "  - interface: wan2\n",
                "    src-addr: 192.168.10.10-192.168.10.20\n",
                "    src-addr-opt-ipgroup: \"\"\n",
                "    url: \"{}\"\n",
                "    tag: ClnA\n",
                "  - interface: wan2\n",
                "    src-addr: 192.168.20.10-192.168.20.20\n",
                "    src-addr-opt-ipgroup: \"\"\n",
                "    url: \"{}\"\n",
                "    tag: ClnB\n",
                "ip-group:\n",
                "  - tag: ClnA\n",
                "    url: \"{}\"\n",
                "  - tag: ClnB\n",
                "    url: \"{}\"\n",
                "stream-ipport:\n",
                "  - type: \"1\"\n",
                "    opt-tagname: ClnAFlow\n",
                "    interface: \"\"\n",
                "    nexthop: 192.168.1.2\n",
                "    src-addr: 192.168.10.10-192.168.10.20\n",
                "    src-addr-opt-ipgroup: \"\"\n",
                "    ip-group: ClnA\n",
                "    mode: 0\n",
                "    ifaceband: 0\n",
                "  - type: \"1\"\n",
                "    opt-tagname: ClnBFlow\n",
                "    interface: \"\"\n",
                "    nexthop: 192.168.1.3\n",
                "    src-addr: 192.168.20.10-192.168.20.20\n",
                "    src-addr-opt-ipgroup: \"\"\n",
                "    ip-group: ClnB\n",
                "    mode: 0\n",
                "    ifaceband: 0\n"
            ),
            harness.fixture().url("/clean/a-isp.txt"),
            harness.fixture().url("/clean/b-isp.txt"),
            harness.fixture().url("/clean/a-domain.txt"),
            harness.fixture().url("/clean/b-domain.txt"),
            harness.fixture().url("/clean/a-ipv4.txt"),
            harness.fixture().url("/clean/b-ipv4.txt"),
        ),
    );
    let config_path = harness.write_config("clean.yml", &config)?;
    let config_path_str = config_path.to_string_lossy().to_string();

    let failure = harness.run_cli_expect_failure(&["-c", &config_path_str, "-r", "clean"])?;
    assert_stderr_contains(&failure, "Clean mode requires -tag")?;

    harness.run_cli_expect_success(&["-c", &config_path_str, "-r", "once", "-m", "ii"])?;

    let api = harness.login_api().await?;
    assert_eq!(
        ikuai::custom_isp::show_custom_isp_by_tag_name(&api, "ClnA")
            .await
            .map_err(|e| format!("failed to query ClnA custom ISP: {e}"))?
            .len(),
        1
    );
    assert_eq!(
        ikuai::custom_isp::show_custom_isp_by_tag_name(&api, "ClnB")
            .await
            .map_err(|e| format!("failed to query ClnB custom ISP: {e}"))?
            .len(),
        1
    );
    assert_eq!(
        ikuai::stream_domain::show_stream_domain_by_tag_name(&api, "ClnA")
            .await
            .map_err(|e| format!("failed to query ClnA domain: {e}"))?
            .len(),
        1
    );
    assert_eq!(
        ikuai::stream_domain::show_stream_domain_by_tag_name(&api, "ClnB")
            .await
            .map_err(|e| format!("failed to query ClnB domain: {e}"))?
            .len(),
        1
    );
    assert_eq!(
        ikuai::ip_group::show_ip_group_by_tag_name(&api, "ClnA")
            .await
            .map_err(|e| format!("failed to query ClnA ip-group: {e}"))?
            .len(),
        1
    );
    assert_eq!(
        ikuai::ip_group::show_ip_group_by_tag_name(&api, "ClnB")
            .await
            .map_err(|e| format!("failed to query ClnB ip-group: {e}"))?
            .len(),
        1
    );
    assert_eq!(
        ikuai::stream_ipport::show_stream_ipport_by_tag_name(&api, "ClnAFlow")
            .await
            .map_err(|e| format!("failed to query ClnA stream-ipport: {e}"))?
            .len(),
        1
    );
    assert_eq!(
        ikuai::stream_ipport::show_stream_ipport_by_tag_name(&api, "ClnBFlow")
            .await
            .map_err(|e| format!("failed to query ClnB stream-ipport: {e}"))?
            .len(),
        1
    );

    harness.run_cli_expect_success(&["-c", &config_path_str, "-r", "clean", "-tag", "ClnA"])?;

    let api = harness.login_api().await?;
    assert_eq!(
        ikuai::custom_isp::show_custom_isp_by_tag_name(&api, "ClnA")
            .await
            .map_err(|e| format!("failed to re-query ClnA custom ISP: {e}"))?
            .len(),
        0
    );
    assert_eq!(
        ikuai::stream_domain::show_stream_domain_by_tag_name(&api, "ClnA")
            .await
            .map_err(|e| format!("failed to re-query ClnA domain: {e}"))?
            .len(),
        0
    );
    assert_eq!(
        ikuai::ip_group::show_ip_group_by_tag_name(&api, "ClnA")
            .await
            .map_err(|e| format!("failed to re-query ClnA ip-group: {e}"))?
            .len(),
        0
    );
    assert_eq!(
        ikuai::stream_ipport::show_stream_ipport_by_tag_name(&api, "ClnAFlow")
            .await
            .map_err(|e| format!("failed to re-query ClnA stream-ipport: {e}"))?
            .len(),
        0
    );

    assert_eq!(
        ikuai::custom_isp::show_custom_isp_by_tag_name(&api, "ClnB")
            .await
            .map_err(|e| format!("failed to verify ClnB custom ISP after clean: {e}"))?
            .len(),
        1
    );
    assert_eq!(
        ikuai::stream_domain::show_stream_domain_by_tag_name(&api, "ClnB")
            .await
            .map_err(|e| format!("failed to verify ClnB domain after clean: {e}"))?
            .len(),
        1
    );
    assert_eq!(
        ikuai::ip_group::show_ip_group_by_tag_name(&api, "ClnB")
            .await
            .map_err(|e| format!("failed to verify ClnB ip-group after clean: {e}"))?
            .len(),
        1
    );
    assert_eq!(
        ikuai::stream_ipport::show_stream_ipport_by_tag_name(&api, "ClnBFlow")
            .await
            .map_err(|e| format!("failed to verify ClnB stream-ipport after clean: {e}"))?
            .len(),
        1
    );

    Ok(())
}
