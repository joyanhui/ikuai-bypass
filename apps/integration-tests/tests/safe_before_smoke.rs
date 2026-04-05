mod common;

use common::{TestHarness, csv_items, render_test_config};
use ikb_core::ikuai;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn safe_before_smoke() -> Result<(), String> {
    let harness = TestHarness::start("safe_before_smoke").await?;

    harness
        .fixture()
        .set_text("/safe/isp.txt", "10.0.0.0/24\n10.0.1.0/24\n");
    harness
        .fixture()
        .set_text("/safe/domain.txt", "safe.example\nkeep.example\n");
    harness.fixture().set_text("/safe/ipv4.txt", "100.64.0.1\n100.64.0.2\n");
    harness
        .fixture()
        .set_text("/safe/ipv6.txt", "2001:db8:2::1\n2001:db8:2::/64\n");

    let config = render_test_config(
        harness.base_url(),
        harness.username(),
        harness.password(),
        &format!(
            concat!(
                "custom-isp:\n",
                "  - tag: SafeIsp\n",
                "    url: \"{}\"\n",
                "stream-domain:\n",
                "  - interface: wan2\n",
                "    src-addr: 192.168.9.10-192.168.9.20\n",
                "    src-addr-opt-ipgroup: \"\"\n",
                "    url: \"{}\"\n",
                "    tag: SafeDom\n",
                "ip-group:\n",
                "  - tag: Safe4\n",
                "    url: \"{}\"\n",
                "ipv6-group:\n",
                "  - tag: Safe6\n",
                "    url: \"{}\"\n",
                "stream-ipport:\n",
                "  - type: \"1\"\n",
                "    opt-tagname: SafeRoute\n",
                "    interface: \"\"\n",
                "    nexthop: 192.168.1.2\n",
                "    src-addr: 192.168.9.10-192.168.9.20\n",
                "    src-addr-opt-ipgroup: \"\"\n",
                "    ip-group: Safe4\n",
                "    mode: 0\n",
                "    ifaceband: 0\n"
            ),
            harness.fixture().url("/safe/isp.txt"),
            harness.fixture().url("/safe/domain.txt"),
            harness.fixture().url("/safe/ipv4.txt"),
            harness.fixture().url("/safe/ipv6.txt"),
        ),
    );
    let config_path = harness.write_config("safe-before.yml", &config)?;
    let config_path_str = config_path.to_string_lossy().to_string();

    harness.run_cli_expect_success(&["-c", &config_path_str, "-r", "once", "-m", "iip"])?;

    let api = harness.login_api().await?;
    let before_custom = ikuai::custom_isp::show_custom_isp_by_tag_name(&api, "SafeIsp")
        .await
        .map_err(|e| format!("failed to query pre-failure custom ISP: {e}"))?;
    let before_domain = ikuai::stream_domain::show_stream_domain_by_tag_name(&api, "SafeDom")
        .await
        .map_err(|e| format!("failed to query pre-failure stream-domain: {e}"))?;
    let before_ipv4 = ikuai::ip_group::show_ip_group_by_tag_name(&api, "Safe4")
        .await
        .map_err(|e| format!("failed to query pre-failure IPv4 groups: {e}"))?;
    let before_ipv6 = ikuai::ipv6_group::show_ipv6_group_by_tag_name(&api, "Safe6")
        .await
        .map_err(|e| format!("failed to query pre-failure IPv6 groups: {e}"))?;
    let before_stream = ikuai::stream_ipport::show_stream_ipport_by_tag_name(&api, "SafeRoute")
        .await
        .map_err(|e| format!("failed to query pre-failure stream-ipport: {e}"))?;

    assert_eq!(before_custom.len(), 1);
    assert_eq!(before_domain.len(), 1);
    assert_eq!(before_ipv4.len(), 1);
    assert_eq!(before_ipv6.len(), 1);
    assert_eq!(before_stream.len(), 1);

    harness
        .fixture()
        .set_status("/safe/isp.txt", 503, "custom isp download failed\n");
    harness
        .fixture()
        .set_status("/safe/domain.txt", 503, "domain download failed\n");
    harness
        .fixture()
        .set_status("/safe/ipv4.txt", 503, "ipv4 download failed\n");
    harness
        .fixture()
        .set_status("/safe/ipv6.txt", 503, "ipv6 download failed\n");

    harness.run_cli_expect_success(&["-c", &config_path_str, "-r", "once", "-m", "iip"])?;

    let api = harness.login_api().await?;
    let after_custom = ikuai::custom_isp::show_custom_isp_by_tag_name(&api, "SafeIsp")
        .await
        .map_err(|e| format!("failed to query post-failure custom ISP: {e}"))?;
    let after_domain = ikuai::stream_domain::show_stream_domain_by_tag_name(&api, "SafeDom")
        .await
        .map_err(|e| format!("failed to query post-failure stream-domain: {e}"))?;
    let after_ipv4 = ikuai::ip_group::show_ip_group_by_tag_name(&api, "Safe4")
        .await
        .map_err(|e| format!("failed to query post-failure IPv4 groups: {e}"))?;
    let after_ipv6 = ikuai::ipv6_group::show_ipv6_group_by_tag_name(&api, "Safe6")
        .await
        .map_err(|e| format!("failed to query post-failure IPv6 groups: {e}"))?;
    let after_stream = ikuai::stream_ipport::show_stream_ipport_by_tag_name(&api, "SafeRoute")
        .await
        .map_err(|e| format!("failed to query post-failure stream-ipport: {e}"))?;

    assert_eq!(after_custom.len(), 1);
    assert_eq!(after_domain.len(), 1);
    assert_eq!(after_ipv4.len(), 1);
    assert_eq!(after_ipv6.len(), 1);
    assert_eq!(after_stream.len(), 1);

    assert_eq!(after_custom[0].id, before_custom[0].id);
    assert_eq!(csv_items(&after_custom[0].ipgroup), csv_items(&before_custom[0].ipgroup));

    assert_eq!(after_domain[0].id, before_domain[0].id);
    assert_eq!(csv_items(&after_domain[0].domain), csv_items(&before_domain[0].domain));

    assert_eq!(after_ipv4[0].id, before_ipv4[0].id);
    assert_eq!(csv_items(&after_ipv4[0].addr_pool), csv_items(&before_ipv4[0].addr_pool));

    assert_eq!(after_ipv6[0].id, before_ipv6[0].id);
    assert_eq!(csv_items(&after_ipv6[0].addr_pool), csv_items(&before_ipv6[0].addr_pool));

    assert_eq!(after_stream[0].id, before_stream[0].id);
    assert_eq!(after_stream[0].nexthop, before_stream[0].nexthop);

    Ok(())
}
