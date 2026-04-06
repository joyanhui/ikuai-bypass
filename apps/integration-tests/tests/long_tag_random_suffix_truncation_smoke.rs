// 覆盖点：
// 1) 超长 tag 在随机后缀开启时的截断命名；
// 2) 截断后的 ip-group / ipv6-group 仍可被正确识别并原地更新；
// 3) stream-ipport 仍能通过长 tag 解析到截断后的目标分组名。
// Coverage:
// 1) Long-tag truncation with deterministic random suffix enabled.
// 2) Truncated IPv4/IPv6 groups are still matched and edited in place.
// 3) stream-ipport can still resolve destination groups via the original long tag.
mod common;

use common::{TestHarness, csv_items, render_test_config};
use ikb_core::ikuai;

const LONG_IPV4_TAG: &str = "VeryLongIpv4TagForRandomSuffixTruncationCase";
const LONG_IPV6_TAG: &str = "VeryLongIpv6TagForRandomSuffixTruncationCase";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn long_tag_random_suffix_truncation_smoke() -> Result<(), String> {
    let harness = TestHarness::start("long_tag_random_suffix_truncation_smoke").await?;
    common::print_failure_hint(
        "long_tag_random_suffix_truncation_smoke",
        harness.artifact_dir(),
    );

    harness.fixture().set_text(
        "/long/ipv4.txt",
        concat!("10.10.0.1\n", "10.10.0.2\n", "10.10.0.3\n", "10.10.0.4\n"),
    );
    harness.fixture().set_text(
        "/long/ipv6.txt",
        concat!(
            "2001:db8:10::1\n",
            "2001:db8:10::2\n",
            "2001:db8:10::3\n",
            "2001:db8:10::4\n"
        ),
    );

    let config = render_test_config(
        harness.base_url(),
        harness.username(),
        harness.password(),
        &format!(
            concat!(
                "ip-group:\n",
                "  - tag: {}\n",
                "    url: \"{}\"\n",
                "ipv6-group:\n",
                "  - tag: {}\n",
                "    url: \"{}\"\n",
                "stream-ipport:\n",
                "  - type: \"1\"\n",
                "    opt-tagname: LongTagRoute\n",
                "    interface: \"\"\n",
                "    nexthop: 192.168.1.2\n",
                "    src-addr: 192.168.10.10-192.168.10.20\n",
                "    src-addr-opt-ipgroup: \"\"\n",
                "    ip-group: {}\n",
                "    mode: 0\n",
                "    ifaceband: 0\n",
                "MaxNumberOfOneRecords:\n",
                "  Isp: 5000\n",
                "  Ipv4: 2\n",
                "  Ipv6: 2\n",
                "  Domain: 5000\n"
            ),
            LONG_IPV4_TAG,
            harness.fixture().url("/long/ipv4.txt"),
            LONG_IPV6_TAG,
            harness.fixture().url("/long/ipv6.txt"),
            LONG_IPV4_TAG,
        ),
    );
    let config_path = harness.write_config("long-tag.yml", &config)?;
    let config_path_str = config_path.to_string_lossy().to_string();

    harness.run_cli_expect_success(&["-c", &config_path_str, "-r", "once", "-m", "ip"])?;

    let api = harness.login_api().await?;

    let mut ipv4_groups = ikuai::ip_group::show_ip_group_by_tag_name(&api, LONG_IPV4_TAG)
        .await
        .map_err(|e| format!("failed to query long-tag IPv4 groups: {e}"))?;
    ipv4_groups.sort_by(|a, b| a.group_name.cmp(&b.group_name));
    assert_eq!(ipv4_groups.len(), 2, "expected two IPv4 chunks");

    let expected_ipv4_names = vec![
        ikuai::tag_name::build_indexed_ip_group_tag_name(LONG_IPV4_TAG, 0),
        ikuai::tag_name::build_indexed_ip_group_tag_name(LONG_IPV4_TAG, 1),
    ];
    let actual_ipv4_names: Vec<String> = ipv4_groups
        .iter()
        .map(|item| item.group_name.to_string())
        .collect();
    assert_eq!(actual_ipv4_names, expected_ipv4_names);
    for name in &actual_ipv4_names {
        assert_eq!(
            name.len(),
            15,
            "truncated IPv4 group name should fill 15 bytes"
        );
    }
    let ipv4_ids: Vec<i64> = ipv4_groups.iter().map(|item| item.id).collect();
    assert_eq!(
        csv_items(&ipv4_groups[0].addr_pool),
        vec!["10.10.0.1", "10.10.0.2"]
    );
    assert_eq!(
        csv_items(&ipv4_groups[1].addr_pool),
        vec!["10.10.0.3", "10.10.0.4"]
    );

    let mut ipv6_groups = ikuai::ipv6_group::show_ipv6_group_by_tag_name(&api, LONG_IPV6_TAG)
        .await
        .map_err(|e| format!("failed to query long-tag IPv6 groups: {e}"))?;
    ipv6_groups.sort_by(|a, b| a.group_name.cmp(&b.group_name));
    assert_eq!(ipv6_groups.len(), 2, "expected two IPv6 chunks");

    let expected_ipv6_names = vec![
        ikuai::tag_name::build_indexed_ip_group_tag_name(LONG_IPV6_TAG, 0),
        ikuai::tag_name::build_indexed_ip_group_tag_name(LONG_IPV6_TAG, 1),
    ];
    let actual_ipv6_names: Vec<String> = ipv6_groups
        .iter()
        .map(|item| item.group_name.to_string())
        .collect();
    assert_eq!(actual_ipv6_names, expected_ipv6_names);
    for name in &actual_ipv6_names {
        assert_eq!(
            name.len(),
            15,
            "truncated IPv6 group name should fill 15 bytes"
        );
    }
    let ipv6_ids: Vec<i64> = ipv6_groups.iter().map(|item| item.id).collect();

    let routes = ikuai::stream_ipport::show_stream_ipport_by_tag_name(&api, "LongTagRoute")
        .await
        .map_err(|e| format!("failed to query long-tag stream-ipport: {e}"))?;
    assert_eq!(routes.len(), 1, "expected one stream-ipport rule");
    let route_id = routes[0].id;
    assert_eq!(csv_items(&routes[0].dst_addr), expected_ipv4_names.clone());

    harness.fixture().set_text(
        "/long/ipv4.txt",
        concat!("10.10.1.1\n", "10.10.1.2\n", "10.10.1.3\n", "10.10.1.4\n"),
    );
    harness.fixture().set_text(
        "/long/ipv6.txt",
        concat!(
            "2001:db8:11::1\n",
            "2001:db8:11::2\n",
            "2001:db8:11::3\n",
            "2001:db8:11::4\n"
        ),
    );

    harness.run_cli_expect_success(&["-c", &config_path_str, "-r", "once", "-m", "ip"])?;

    let api = harness.login_api().await?;
    let mut ipv4_groups = ikuai::ip_group::show_ip_group_by_tag_name(&api, LONG_IPV4_TAG)
        .await
        .map_err(|e| format!("failed to re-query long-tag IPv4 groups: {e}"))?;
    ipv4_groups.sort_by(|a, b| a.group_name.cmp(&b.group_name));
    assert_eq!(
        ipv4_groups.iter().map(|item| item.id).collect::<Vec<_>>(),
        ipv4_ids,
        "truncated IPv4 groups should edit in place"
    );
    assert_eq!(
        csv_items(&ipv4_groups[0].addr_pool),
        vec!["10.10.1.1", "10.10.1.2"]
    );
    assert_eq!(
        csv_items(&ipv4_groups[1].addr_pool),
        vec!["10.10.1.3", "10.10.1.4"]
    );

    let mut ipv6_groups = ikuai::ipv6_group::show_ipv6_group_by_tag_name(&api, LONG_IPV6_TAG)
        .await
        .map_err(|e| format!("failed to re-query long-tag IPv6 groups: {e}"))?;
    ipv6_groups.sort_by(|a, b| a.group_name.cmp(&b.group_name));
    assert_eq!(
        ipv6_groups.iter().map(|item| item.id).collect::<Vec<_>>(),
        ipv6_ids,
        "truncated IPv6 groups should edit in place"
    );
    assert_eq!(
        csv_items(&ipv6_groups[0].addr_pool),
        vec!["2001:db8:11::1", "2001:db8:11::2"]
    );
    assert_eq!(
        csv_items(&ipv6_groups[1].addr_pool),
        vec!["2001:db8:11::3", "2001:db8:11::4"]
    );

    let routes = ikuai::stream_ipport::show_stream_ipport_by_tag_name(&api, "LongTagRoute")
        .await
        .map_err(|e| format!("failed to re-query long-tag stream-ipport: {e}"))?;
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].id, route_id, "stream-ipport should edit in place");
    assert_eq!(csv_items(&routes[0].dst_addr), expected_ipv4_names);

    Ok(())
}
