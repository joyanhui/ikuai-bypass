// 覆盖点：
// 1) 多分片规则同步成功后形成稳定的旧状态；
// 2) 第二次本应缩容到更少分片，但上游下载失败；
// 3) 验证 Safe-Before：旧分片数量、ID 与内容都不被清理或误改。
// Coverage:
// 1) Multi-chunk rules establish a stable baseline.
// 2) A later shrink would reduce chunks, but upstream fetch fails.
// 3) Safe-Before preserves old chunk counts, IDs, and contents.
mod common;

use common::{TestHarness, csv_items, render_test_config};
use ikb_core::ikuai;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn safe_before_chunk_shrink_smoke() -> Result<(), String> {
    let harness = TestHarness::start("safe_before_chunk_shrink_smoke").await?;
    common::print_failure_hint("safe_before_chunk_shrink_smoke", harness.artifact_dir());

    harness.fixture().set_text(
        "/safe-chunk/isp.txt",
        concat!(
            "10.20.0.1\n",
            "10.20.0.2\n",
            "10.20.0.3\n",
            "10.20.0.4\n",
            "10.20.0.5\n"
        ),
    );
    harness.fixture().set_text(
        "/safe-chunk/domain.txt",
        concat!(
            "safe-a.example\n",
            "safe-b.example\n",
            "safe-c.example\n",
            "safe-d.example\n",
            "safe-e.example\n"
        ),
    );
    harness.fixture().set_text(
        "/safe-chunk/ipv4.txt",
        concat!(
            "100.80.0.1\n",
            "100.80.0.2\n",
            "100.80.0.3\n",
            "100.80.0.4\n",
            "100.80.0.5\n"
        ),
    );
    harness.fixture().set_text(
        "/safe-chunk/ipv6.txt",
        concat!(
            "2001:db8:20::1\n",
            "2001:db8:20::2\n",
            "2001:db8:20::3\n",
            "2001:db8:20::4\n",
            "2001:db8:20::5\n"
        ),
    );

    let config = render_test_config(
        harness.base_url(),
        harness.username(),
        harness.password(),
        &format!(
            concat!(
                "custom-isp:\n",
                "  - tag: SafeChunkIsp\n",
                "    url: \"{}\"\n",
                "stream-domain:\n",
                "  - interface: wan2\n",
                "    src-addr: 192.168.88.10-192.168.88.20\n",
                "    src-addr-opt-ipgroup: \"\"\n",
                "    url: \"{}\"\n",
                "    tag: SafeChunkDom\n",
                "ip-group:\n",
                "  - tag: SafeChunk4\n",
                "    url: \"{}\"\n",
                "ipv6-group:\n",
                "  - tag: SafeChunk6\n",
                "    url: \"{}\"\n",
                "stream-ipport:\n",
                "  - type: \"1\"\n",
                "    opt-tagname: SafeChunkRoute\n",
                "    interface: \"\"\n",
                "    nexthop: 192.168.1.2\n",
                "    src-addr: 192.168.88.10-192.168.88.20\n",
                "    src-addr-opt-ipgroup: \"\"\n",
                "    ip-group: SafeChunk4\n",
                "    mode: 0\n",
                "    ifaceband: 0\n",
                "MaxNumberOfOneRecords:\n",
                "  Isp: 2\n",
                "  Ipv4: 2\n",
                "  Ipv6: 2\n",
                "  Domain: 2\n"
            ),
            harness.fixture().url("/safe-chunk/isp.txt"),
            harness.fixture().url("/safe-chunk/domain.txt"),
            harness.fixture().url("/safe-chunk/ipv4.txt"),
            harness.fixture().url("/safe-chunk/ipv6.txt"),
        ),
    );
    let config_path = harness.write_config("safe-before-chunk.yml", &config)?;
    let config_path_str = config_path.to_string_lossy().to_string();

    harness.run_cli_expect_success(&["-c", &config_path_str, "-r", "once", "-m", "iip"])?;

    let api = harness.login_api().await?;

    let before_custom = sorted_custom_isp(&api, "SafeChunkIsp").await?;
    let before_domain = sorted_stream_domain(&api, "SafeChunkDom").await?;
    let before_ipv4 = sorted_ip_group(&api, "SafeChunk4").await?;
    let before_ipv6 = sorted_ipv6_group(&api, "SafeChunk6").await?;
    let before_route = ikuai::stream_ipport::show_stream_ipport_by_tag_name(&api, "SafeChunkRoute")
        .await
        .map_err(|e| format!("failed to query SafeChunkRoute before failure: {e}"))?;

    assert_eq!(before_custom.len(), 3, "expected three custom ISP chunks");
    assert_eq!(
        before_domain.len(),
        3,
        "expected three stream-domain chunks"
    );
    assert_eq!(before_ipv4.len(), 3, "expected three IPv4 chunks");
    assert_eq!(before_ipv6.len(), 3, "expected three IPv6 chunks");
    assert_eq!(before_route.len(), 1, "expected one stream-ipport rule");

    harness.fixture().set_status(
        "/safe-chunk/isp.txt",
        503,
        "shrink would reduce chunks to one\n",
    );
    harness.fixture().set_status(
        "/safe-chunk/domain.txt",
        503,
        "shrink would reduce chunks to one\n",
    );
    harness.fixture().set_status(
        "/safe-chunk/ipv4.txt",
        503,
        "shrink would reduce chunks to one\n",
    );
    harness.fixture().set_status(
        "/safe-chunk/ipv6.txt",
        503,
        "shrink would reduce chunks to one\n",
    );

    harness.run_cli_expect_success(&["-c", &config_path_str, "-r", "once", "-m", "iip"])?;

    let api = harness.login_api().await?;
    let after_custom = sorted_custom_isp(&api, "SafeChunkIsp").await?;
    let after_domain = sorted_stream_domain(&api, "SafeChunkDom").await?;
    let after_ipv4 = sorted_ip_group(&api, "SafeChunk4").await?;
    let after_ipv6 = sorted_ipv6_group(&api, "SafeChunk6").await?;
    let after_route = ikuai::stream_ipport::show_stream_ipport_by_tag_name(&api, "SafeChunkRoute")
        .await
        .map_err(|e| format!("failed to query SafeChunkRoute after failure: {e}"))?;

    assert_eq!(
        after_custom.iter().map(|item| item.id).collect::<Vec<_>>(),
        before_custom.iter().map(|item| item.id).collect::<Vec<_>>(),
        "Safe-Before should keep all custom ISP chunk IDs"
    );
    assert_eq!(
        after_domain.iter().map(|item| item.id).collect::<Vec<_>>(),
        before_domain.iter().map(|item| item.id).collect::<Vec<_>>(),
        "Safe-Before should keep all stream-domain chunk IDs"
    );
    assert_eq!(
        after_ipv4.iter().map(|item| item.id).collect::<Vec<_>>(),
        before_ipv4.iter().map(|item| item.id).collect::<Vec<_>>(),
        "Safe-Before should keep all IPv4 chunk IDs"
    );
    assert_eq!(
        after_ipv6.iter().map(|item| item.id).collect::<Vec<_>>(),
        before_ipv6.iter().map(|item| item.id).collect::<Vec<_>>(),
        "Safe-Before should keep all IPv6 chunk IDs"
    );

    assert_eq!(
        after_custom
            .iter()
            .map(|item| csv_items(&item.ipgroup))
            .collect::<Vec<_>>(),
        before_custom
            .iter()
            .map(|item| csv_items(&item.ipgroup))
            .collect::<Vec<_>>(),
        "Safe-Before should keep custom ISP chunk contents"
    );
    assert_eq!(
        after_domain
            .iter()
            .map(|item| csv_items(&item.domain))
            .collect::<Vec<_>>(),
        before_domain
            .iter()
            .map(|item| csv_items(&item.domain))
            .collect::<Vec<_>>(),
        "Safe-Before should keep stream-domain chunk contents"
    );
    assert_eq!(
        after_ipv4
            .iter()
            .map(|item| csv_items(&item.addr_pool))
            .collect::<Vec<_>>(),
        before_ipv4
            .iter()
            .map(|item| csv_items(&item.addr_pool))
            .collect::<Vec<_>>(),
        "Safe-Before should keep IPv4 chunk contents"
    );
    assert_eq!(
        after_ipv6
            .iter()
            .map(|item| csv_items(&item.addr_pool))
            .collect::<Vec<_>>(),
        before_ipv6
            .iter()
            .map(|item| csv_items(&item.addr_pool))
            .collect::<Vec<_>>(),
        "Safe-Before should keep IPv6 chunk contents"
    );

    assert_eq!(after_route.len(), 1);
    assert_eq!(after_route[0].id, before_route[0].id);
    assert_eq!(
        csv_items(&after_route[0].dst_addr),
        csv_items(&before_route[0].dst_addr)
    );

    Ok(())
}

async fn sorted_custom_isp(
    api: &ikuai::IKuaiClient,
    tag: &str,
) -> Result<Vec<ikuai::CustomIspData>, String> {
    let mut items = ikuai::custom_isp::show_custom_isp_by_tag_name(api, tag)
        .await
        .map_err(|e| format!("failed to query custom ISP '{tag}': {e}"))?;
    items.sort_by_key(|item| item.id);
    Ok(items)
}

async fn sorted_stream_domain(
    api: &ikuai::IKuaiClient,
    tag: &str,
) -> Result<Vec<ikuai::StreamDomainData>, String> {
    let mut items = ikuai::stream_domain::show_stream_domain_by_tag_name(api, tag)
        .await
        .map_err(|e| format!("failed to query stream-domain '{tag}': {e}"))?;
    items.sort_by_key(|item| item.id);
    Ok(items)
}

async fn sorted_ip_group(
    api: &ikuai::IKuaiClient,
    tag: &str,
) -> Result<Vec<ikuai::IpGroupData>, String> {
    let mut items = ikuai::ip_group::show_ip_group_by_tag_name(api, tag)
        .await
        .map_err(|e| format!("failed to query ip-group '{tag}': {e}"))?;
    items.sort_by_key(|item| item.id);
    Ok(items)
}

async fn sorted_ipv6_group(
    api: &ikuai::IKuaiClient,
    tag: &str,
) -> Result<Vec<ikuai::Ipv6GroupData>, String> {
    let mut items = ikuai::ipv6_group::show_ipv6_group_by_tag_name(api, tag)
        .await
        .map_err(|e| format!("failed to query ipv6-group '{tag}': {e}"))?;
    items.sort_by_key(|item| item.id);
    Ok(items)
}
