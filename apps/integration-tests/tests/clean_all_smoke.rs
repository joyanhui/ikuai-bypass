mod common;

use common::{TestHarness, render_test_config};
use ikb_core::ikuai;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn clean_all_smoke() -> Result<(), String> {
    let harness = TestHarness::start("clean_all_smoke").await?;

    harness.fixture().set_text("/cleanall/a-isp.txt", "10.1.1.0/24\n");
    harness.fixture().set_text("/cleanall/b-isp.txt", "10.2.2.0/24\n");
    harness
        .fixture()
        .set_text("/cleanall/a-domain.txt", "a.example\n");
    harness
        .fixture()
        .set_text("/cleanall/b-domain.txt", "b.example\n");
    harness.fixture().set_text("/cleanall/a-ipv4.txt", "172.16.10.1\n");
    harness.fixture().set_text("/cleanall/b-ipv4.txt", "172.16.20.1\n");

    let config = render_test_config(
        harness.base_url(),
        harness.username(),
        harness.password(),
        &format!(
            concat!(
                "custom-isp:\n",
                "  - tag: AllA\n",
                "    url: \"{}\"\n",
                "  - tag: AllB\n",
                "    url: \"{}\"\n",
                "stream-domain:\n",
                "  - interface: wan2\n",
                "    src-addr: 192.168.10.10-192.168.10.20\n",
                "    src-addr-opt-ipgroup: \"\"\n",
                "    url: \"{}\"\n",
                "    tag: AllA\n",
                "  - interface: wan2\n",
                "    src-addr: 192.168.20.10-192.168.20.20\n",
                "    src-addr-opt-ipgroup: \"\"\n",
                "    url: \"{}\"\n",
                "    tag: AllB\n",
                "ip-group:\n",
                "  - tag: AllA\n",
                "    url: \"{}\"\n",
                "  - tag: AllB\n",
                "    url: \"{}\"\n",
                "stream-ipport:\n",
                "  - type: \"1\"\n",
                "    opt-tagname: AllAFlow\n",
                "    interface: \"\"\n",
                "    nexthop: 192.168.1.2\n",
                "    src-addr: 192.168.10.10-192.168.10.20\n",
                "    src-addr-opt-ipgroup: \"\"\n",
                "    ip-group: AllA\n",
                "    mode: 0\n",
                "    ifaceband: 0\n",
                "  - type: \"1\"\n",
                "    opt-tagname: AllBFlow\n",
                "    interface: \"\"\n",
                "    nexthop: 192.168.1.3\n",
                "    src-addr: 192.168.20.10-192.168.20.20\n",
                "    src-addr-opt-ipgroup: \"\"\n",
                "    ip-group: AllB\n",
                "    mode: 0\n",
                "    ifaceband: 0\n"
            ),
            harness.fixture().url("/cleanall/a-isp.txt"),
            harness.fixture().url("/cleanall/b-isp.txt"),
            harness.fixture().url("/cleanall/a-domain.txt"),
            harness.fixture().url("/cleanall/b-domain.txt"),
            harness.fixture().url("/cleanall/a-ipv4.txt"),
            harness.fixture().url("/cleanall/b-ipv4.txt"),
        ),
    );
    let config_path = harness.write_config("clean-all.yml", &config)?;
    let config_path_str = config_path.to_string_lossy().to_string();

    harness.run_cli_expect_success(&["-c", &config_path_str, "-r", "once", "-m", "ii"])?;
    harness.run_cli_expect_success(&[
        "-c",
        &config_path_str,
        "-r",
        "clean",
        "-tag",
        ikuai::CLEAN_MODE_ALL,
    ])?;

    let api = harness.login_api().await?;
    assert!(
        ikuai::custom_isp::show_custom_isp_by_tag_name(&api, "")
            .await
            .map_err(|e| format!("failed to query custom isp after cleanAll: {e}"))?
            .is_empty()
    );
    assert!(
        ikuai::stream_domain::show_stream_domain_by_tag_name(&api, "")
            .await
            .map_err(|e| format!("failed to query stream-domain after cleanAll: {e}"))?
            .is_empty()
    );
    assert!(
        ikuai::ip_group::show_ip_group_by_tag_name(&api, "")
            .await
            .map_err(|e| format!("failed to query ip-group after cleanAll: {e}"))?
            .is_empty()
    );
    assert!(
        ikuai::stream_ipport::show_stream_ipport_by_tag_name(&api, "")
            .await
            .map_err(|e| format!("failed to query stream-ipport after cleanAll: {e}"))?
            .is_empty()
    );

    Ok(())
}
