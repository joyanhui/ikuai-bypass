// 覆盖点：
// 1) 手工维护、非 IKB 前缀的 IP 分组可被 stream-domain 引用；
// 2) 手工维护、非 IKB 前缀的 IP 分组可被 stream-ipport 的 src/dst 引用；
// 3) 规则引用解析不应把“查询适配”误当成“仅限本项目托管分组”。
// Coverage:
// 1) User-managed non-IKB IP groups can be referenced by stream-domain.
// 2) User-managed non-IKB IP groups can be referenced by stream-ipport src/dst.
// 3) Rule reference lookup must not be restricted to project-managed groups only.
mod common;

use common::{TestHarness, csv_items, render_test_config};
use ikb_core::ikuai;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn manual_ip_group_refs_smoke() -> Result<(), String> {
    let harness = TestHarness::start("manual_ip_group_refs_smoke").await?;
    common::print_failure_hint("manual_ip_group_refs_smoke", harness.artifact_dir());

    let api = harness.login_api().await?;
    ikuai::ip_group::add_ip_group_named(&api, "asd", "172.16.1.1,172.16.1.2")
        .await
        .map_err(|e| format!("failed to seed manual src ip-group: {e}"))?;
    ikuai::ip_group::add_ip_group_named(&api, "manual-dst", "100.64.0.1,100.64.0.2")
        .await
        .map_err(|e| format!("failed to seed manual dst ip-group: {e}"))?;

    harness.fixture().set_text(
        "/manual/domain.txt",
        concat!("manual.example\n", "object.example\n"),
    );

    let config = render_test_config(
        harness.base_url(),
        harness.username(),
        harness.password(),
        &format!(
            concat!(
                "stream-domain:\n",
                "  - interface: wan1\n",
                "    src-addr: \"\"\n",
                "    src-addr-opt-ipgroup: asd\n",
                "    url: \"{}\"\n",
                "    tag: ManualDom\n",
                "stream-ipport:\n",
                "  - type: \"1\"\n",
                "    opt-tagname: ManualRoute\n",
                "    interface: \"\"\n",
                "    nexthop: 192.168.1.2\n",
                "    src-addr: \"\"\n",
                "    src-addr-opt-ipgroup: asd\n",
                "    src-addr-inv: 0\n",
                "    ip-group: manual-dst\n",
                "    dst-addr-inv: 0\n",
                "    mode: 0\n",
                "    ifaceband: 0\n",
                "MaxNumberOfOneRecords:\n",
                "  Isp: 5000\n",
                "  Ipv4: 5000\n",
                "  Ipv6: 1000\n",
                "  Domain: 5000\n"
            ),
            harness.fixture().url("/manual/domain.txt"),
        ),
    );
    let config_path = harness.write_config("manual-ip-group-refs.yml", &config)?;
    let config_path_str = config_path.to_string_lossy().to_string();

    let output = harness.run_cli_expect_success(&["-c", &config_path_str, "-r", "once", "-m", "ispdomain"])?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("No matching source IP groups found")
        || stdout.contains("No matching destination IP groups found")
    {
        return Err(format!(
            "manual IP group references should not be skipped, got stdout:\n{}",
            stdout
        ));
    }

    let api = harness.login_api().await?;

    let domains = ikuai::stream_domain::show_stream_domain_by_tag_name(&api, "ManualDom")
        .await
        .map_err(|e| format!("failed to query manual stream-domain: {e}"))?;
    assert_eq!(domains.len(), 1, "expected one stream-domain rule");
    assert_eq!(
        csv_items(&domains[0].domain),
        vec!["manual.example", "object.example"]
    );
    assert_eq!(csv_items(&domains[0].src_addr), vec!["asd"]);

    let routes = ikuai::stream_ipport::show_stream_ipport_by_tag_name(&api, "ManualRoute")
        .await
        .map_err(|e| format!("failed to query manual stream-ipport: {e}"))?;
    assert_eq!(routes.len(), 1, "expected one stream-ipport rule");
    assert_eq!(csv_items(&routes[0].src_addr), vec!["asd"]);
    assert_eq!(csv_items(&routes[0].dst_addr), vec!["manual-dst"]);

    Ok(())
}
