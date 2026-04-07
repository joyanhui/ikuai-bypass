// 覆盖点：
// 1) once -m iip 全链路创建规则；
// 2) 第二次同步走原地 edit（ID 稳定）；
// 3) custom-isp / stream-domain / ip-group / ipv6-group / stream-ipport 全量断言。
// Coverage:
// 1) Full iip sync creates rules.
// 2) Second sync updates in place with stable IDs.
// 3) Asserts all major rule types end-to-end.
mod common;

use common::{TestHarness, csv_items, render_test_config};
use ikb_core::ikuai;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rule_sync_update_in_place_smoke() -> Result<(), String> {
    let harness = TestHarness::start("rule_sync_update_in_place_smoke").await?;
    common::print_failure_hint("rule_sync_update_in_place_smoke", harness.artifact_dir());

    harness
        .fixture()
        .set_text("/sync/isp.txt", "1.1.1.0/24\n2.2.2.0/24\n");
    harness
        .fixture()
        .set_text("/sync/domain.txt", "example.com\nfoo.example\n");
    harness
        .fixture()
        .set_text("/sync/ipv4.txt", "8.8.8.8\n9.9.9.0/24\n");
    harness
        .fixture()
        .set_text("/sync/ipv6.txt", "2001:db8::1\n2001:db8::/64\n");

    let config = render_test_config(
        harness.base_url(),
        harness.username(),
        harness.password(),
        &format!(
            concat!(
                "custom-isp:\n",
                "  - tag: SyncIsp\n",
                "    url: \"{}\"\n",
                "stream-domain:\n",
                "  - interface: wan2\n",
                "    src-addr: 192.168.1.10-192.168.1.20\n",
                "    src-addr-opt-ipgroup: \"\"\n",
                "    url: \"{}\"\n",
                "    tag: SyncDom\n",
                "ip-group:\n",
                "  - tag: Sync4\n",
                "    url: \"{}\"\n",
                "ipv6-group:\n",
                "  - tag: Sync6\n",
                "    url: \"{}\"\n",
                "stream-ipport:\n",
                "  - type: \"1\"\n",
                "    opt-tagname: SyncRoute\n",
                "    interface: \"\"\n",
                "    nexthop: 192.168.1.2\n",
                "    src-addr: 192.168.1.10-192.168.1.20\n",
                "    src-addr-opt-ipgroup: \"\"\n",
                "    src-addr-inv: 1\n",
                "    ip-group: Sync4\n",
                "    dst-addr-inv: 1\n",
                "    mode: 0\n",
                "    ifaceband: 0\n"
            ),
            harness.fixture().url("/sync/isp.txt"),
            harness.fixture().url("/sync/domain.txt"),
            harness.fixture().url("/sync/ipv4.txt"),
            harness.fixture().url("/sync/ipv6.txt"),
        ),
    );
    let config_path = harness.write_config("sync.yml", &config)?;
    let config_path_str = config_path.to_string_lossy().to_string();

    harness.run_cli_expect_success(&["-c", &config_path_str, "-r", "once", "-m", "iip"])?;

    let api = harness.login_api().await?;

    let mut custom_isp = ikuai::custom_isp::show_custom_isp_by_tag_name(&api, "SyncIsp")
        .await
        .map_err(|e| format!("failed to query custom ISP: {e}"))?;
    custom_isp.sort_by_key(|item| item.id);
    assert_eq!(custom_isp.len(), 1, "expected one custom ISP chunk");
    assert_eq!(
        csv_items(&custom_isp[0].ipgroup),
        vec!["1.1.1.0/24", "2.2.2.0/24"]
    );
    assert_eq!(custom_isp[0].comment, ikuai::NEW_COMMENT);
    let custom_isp_id = custom_isp[0].id;

    let mut stream_domains = ikuai::stream_domain::show_stream_domain_by_tag_name(&api, "SyncDom")
        .await
        .map_err(|e| format!("failed to query stream-domain: {e}"))?;
    stream_domains.sort_by_key(|item| item.id);
    assert_eq!(stream_domains.len(), 1, "expected one stream-domain chunk");
    assert_eq!(stream_domains[0].interface, "wan2");
    assert_eq!(
        csv_items(&stream_domains[0].domain),
        vec!["example.com", "foo.example"]
    );
    assert_eq!(stream_domains[0].comment, ikuai::NEW_COMMENT);
    let stream_domain_id = stream_domains[0].id;

    let mut ipv4_groups = ikuai::ip_group::show_ip_group_by_tag_name(&api, "Sync4")
        .await
        .map_err(|e| format!("failed to query IPv4 groups: {e}"))?;
    ipv4_groups.sort_by_key(|item| item.id);
    assert_eq!(ipv4_groups.len(), 1, "expected one IPv4 group chunk");
    assert_eq!(
        csv_items(&ipv4_groups[0].addr_pool),
        vec!["8.8.8.8", "9.9.9.0/24"]
    );
    let ipv4_group_id = ipv4_groups[0].id;

    let mut ipv6_groups = ikuai::ipv6_group::show_ipv6_group_by_tag_name(&api, "Sync6")
        .await
        .map_err(|e| format!("failed to query IPv6 groups: {e}"))?;
    ipv6_groups.sort_by_key(|item| item.id);
    assert_eq!(ipv6_groups.len(), 1, "expected one IPv6 group chunk");
    assert_eq!(
        csv_items(&ipv6_groups[0].addr_pool),
        vec!["2001:db8::/64", "2001:db8::1"]
    );
    let ipv6_group_id = ipv6_groups[0].id;

    let mut stream_ipports =
        ikuai::stream_ipport::show_stream_ipport_by_tag_name(&api, "SyncRoute")
            .await
            .map_err(|e| format!("failed to query stream-ipport: {e}"))?;
    stream_ipports.sort_by_key(|item| item.id);
    assert_eq!(stream_ipports.len(), 1, "expected one stream-ipport rule");
    assert_eq!(stream_ipports[0].nexthop, "192.168.1.2");
    assert_eq!(stream_ipports[0].src_addr_inv, 1);
    assert_eq!(stream_ipports[0].dst_addr_inv, 1);
    let stream_ipport_id = stream_ipports[0].id;

    harness
        .fixture()
        .set_text("/sync/isp.txt", "1.1.1.0/24\n3.3.3.0/24\n");
    harness
        .fixture()
        .set_text("/sync/domain.txt", "bar.example\nupdated.example\n");
    harness
        .fixture()
        .set_text("/sync/ipv4.txt", "4.4.4.4\n5.5.5.0/24\n");
    harness
        .fixture()
        .set_text("/sync/ipv6.txt", "2001:db8:1::1\n2001:db8:1::/64\n");

    harness.run_cli_expect_success(&["-c", &config_path_str, "-r", "once", "-m", "iip"])?;

    let api = harness.login_api().await?;

    let mut custom_isp = ikuai::custom_isp::show_custom_isp_by_tag_name(&api, "SyncIsp")
        .await
        .map_err(|e| format!("failed to re-query custom ISP: {e}"))?;
    custom_isp.sort_by_key(|item| item.id);
    assert_eq!(custom_isp.len(), 1);
    assert_eq!(
        custom_isp[0].id, custom_isp_id,
        "custom ISP should edit in-place"
    );
    assert_eq!(
        csv_items(&custom_isp[0].ipgroup),
        vec!["1.1.1.0/24", "3.3.3.0/24"]
    );

    let mut stream_domains = ikuai::stream_domain::show_stream_domain_by_tag_name(&api, "SyncDom")
        .await
        .map_err(|e| format!("failed to re-query stream-domain: {e}"))?;
    stream_domains.sort_by_key(|item| item.id);
    assert_eq!(stream_domains.len(), 1);
    assert_eq!(
        stream_domains[0].id, stream_domain_id,
        "stream-domain should edit in-place"
    );
    assert_eq!(
        csv_items(&stream_domains[0].domain),
        vec!["bar.example", "updated.example"]
    );

    let mut ipv4_groups = ikuai::ip_group::show_ip_group_by_tag_name(&api, "Sync4")
        .await
        .map_err(|e| format!("failed to re-query IPv4 groups: {e}"))?;
    ipv4_groups.sort_by_key(|item| item.id);
    assert_eq!(ipv4_groups.len(), 1);
    assert_eq!(
        ipv4_groups[0].id, ipv4_group_id,
        "IPv4 group should edit in-place"
    );
    assert_eq!(
        csv_items(&ipv4_groups[0].addr_pool),
        vec!["4.4.4.4", "5.5.5.0/24"]
    );

    let mut ipv6_groups = ikuai::ipv6_group::show_ipv6_group_by_tag_name(&api, "Sync6")
        .await
        .map_err(|e| format!("failed to re-query IPv6 groups: {e}"))?;
    ipv6_groups.sort_by_key(|item| item.id);
    assert_eq!(ipv6_groups.len(), 1);
    assert_eq!(
        ipv6_groups[0].id, ipv6_group_id,
        "IPv6 group should edit in-place"
    );
    assert_eq!(
        csv_items(&ipv6_groups[0].addr_pool),
        vec!["2001:db8:1::/64", "2001:db8:1::1"]
    );

    let mut stream_ipports =
        ikuai::stream_ipport::show_stream_ipport_by_tag_name(&api, "SyncRoute")
            .await
            .map_err(|e| format!("failed to re-query stream-ipport: {e}"))?;
    stream_ipports.sort_by_key(|item| item.id);
    assert_eq!(stream_ipports.len(), 1);
    assert_eq!(
        stream_ipports[0].id, stream_ipport_id,
        "stream-ipport should edit in-place"
    );
    assert_eq!(stream_ipports[0].nexthop, "192.168.1.2");
    assert_eq!(stream_ipports[0].src_addr_inv, 1);
    assert_eq!(stream_ipports[0].dst_addr_inv, 1);

    Ok(())
}
