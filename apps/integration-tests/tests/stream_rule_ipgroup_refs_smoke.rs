// 覆盖点：
// 1) ii 模式首轮同步时，先创建 ip-group 再解析 stream-domain 的 src-addr-opt-ipgroup；
// 2) stream-ipport 的 src/dst 都走 ip-group 对象引用；
// 3) 目标对象存在多分片时，规则内应引用全部分片名称。
// Coverage:
// 1) In ii mode, first sync materializes ip-groups before stream-domain object resolution.
// 2) stream-ipport resolves both src/dst via ip-group objects.
// 3) Rules reference all matching chunk names when targets are sharded.
mod common;

use common::{TestHarness, csv_items, render_test_config};
use ikb_core::ikuai;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stream_rule_ipgroup_refs_smoke() -> Result<(), String> {
    let harness = TestHarness::start("stream_rule_ipgroup_refs_smoke").await?;
    common::print_failure_hint("stream_rule_ipgroup_refs_smoke", harness.artifact_dir());

    harness.fixture().set_text(
        "/refs/src-ipv4.txt",
        concat!("172.16.10.1\n", "172.16.10.2\n", "172.16.10.3\n"),
    );
    harness.fixture().set_text(
        "/refs/dst-ipv4.txt",
        concat!(
            "100.64.0.1\n",
            "100.64.0.2\n",
            "100.64.0.3\n",
            "100.64.0.4\n"
        ),
    );
    harness.fixture().set_text(
        "/refs/domain.txt",
        concat!("object.example\n", "ref.example\n", "_skip.me\n"),
    );

    let config = render_test_config(
        harness.base_url(),
        harness.username(),
        harness.password(),
        &format!(
            concat!(
                "ip-group:\n",
                "  - tag: ObjSrc\n",
                "    url: \"{}\"\n",
                "  - tag: ObjDst\n",
                "    url: \"{}\"\n",
                "stream-domain:\n",
                "  - interface: wan1\n",
                "    src-addr: \"\"\n",
                "    src-addr-opt-ipgroup: ObjSrc\n",
                "    url: \"{}\"\n",
                "    tag: ObjDom\n",
                "stream-ipport:\n",
                "  - type: \"1\"\n",
                "    opt-tagname: ObjRoute\n",
                "    interface: \"\"\n",
                "    nexthop: 192.168.1.2\n",
                "    src-addr: \"\"\n",
                "    src-addr-opt-ipgroup: ObjSrc\n",
                "    ip-group: ObjDst\n",
                "    mode: 0\n",
                "    ifaceband: 0\n",
                "MaxNumberOfOneRecords:\n",
                "  Isp: 5000\n",
                "  Ipv4: 2\n",
                "  Ipv6: 1000\n",
                "  Domain: 5000\n"
            ),
            harness.fixture().url("/refs/src-ipv4.txt"),
            harness.fixture().url("/refs/dst-ipv4.txt"),
            harness.fixture().url("/refs/domain.txt"),
        ),
    );
    let config_path = harness.write_config("stream-rule-refs.yml", &config)?;
    let config_path_str = config_path.to_string_lossy().to_string();

    harness.run_cli_expect_success(&["-c", &config_path_str, "-r", "once", "-m", "ii"])?;

    let api = harness.login_api().await?;

    let mut src_groups = ikuai::ip_group::show_ip_group_by_tag_name(&api, "ObjSrc")
        .await
        .map_err(|e| format!("failed to query ObjSrc groups: {e}"))?;
    src_groups.sort_by(|a, b| a.group_name.cmp(&b.group_name));
    let expected_src_names: Vec<String> = src_groups
        .iter()
        .map(|item| item.group_name.to_string())
        .collect();
    assert_eq!(
        expected_src_names.len(),
        2,
        "ObjSrc should split into two groups"
    );

    let mut dst_groups = ikuai::ip_group::show_ip_group_by_tag_name(&api, "ObjDst")
        .await
        .map_err(|e| format!("failed to query ObjDst groups: {e}"))?;
    dst_groups.sort_by(|a, b| a.group_name.cmp(&b.group_name));
    let expected_dst_names: Vec<String> = dst_groups
        .iter()
        .map(|item| item.group_name.to_string())
        .collect();
    assert_eq!(
        expected_dst_names.len(),
        2,
        "ObjDst should split into two groups"
    );

    let domains = ikuai::stream_domain::show_stream_domain_by_tag_name(&api, "ObjDom")
        .await
        .map_err(|e| format!("failed to query object-backed stream-domain: {e}"))?;
    assert_eq!(domains.len(), 1, "expected one stream-domain rule");
    assert_eq!(
        csv_items(&domains[0].domain),
        vec!["object.example", "ref.example"]
    );
    assert_eq!(csv_items(&domains[0].src_addr), expected_src_names.clone());

    let routes = ikuai::stream_ipport::show_stream_ipport_by_tag_name(&api, "ObjRoute")
        .await
        .map_err(|e| format!("failed to query object-backed stream-ipport: {e}"))?;
    assert_eq!(routes.len(), 1, "expected one stream-ipport rule");
    assert_eq!(csv_items(&routes[0].src_addr), expected_src_names);
    assert_eq!(csv_items(&routes[0].dst_addr), expected_dst_names);
    assert_eq!(routes[0].src_addr_inv, 0);
    assert_eq!(routes[0].dst_addr_inv, 0);

    Ok(())
}
