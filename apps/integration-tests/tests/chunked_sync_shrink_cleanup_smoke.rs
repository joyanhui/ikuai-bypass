// 覆盖点：
// 1) 多分片创建（isp/domain/ipv4/ipv6）；
// 2) 第二次同步缩容后，首分片原地更新且冗余分片被删除；
// 3) 同时覆盖注释/空行/非法项过滤后的分片逻辑。
// Coverage:
// 1) Multi-chunk creation across isp/domain/ipv4/ipv6.
// 2) Shrink sync keeps first chunk in place and deletes redundant chunks.
// 3) Exercises chunking after filtering comments/blank/invalid lines.
mod common;

use common::{TestHarness, csv_items, render_test_config};
use ikb_core::ikuai;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn chunked_sync_shrink_cleanup_smoke() -> Result<(), String> {
    let harness = TestHarness::start("chunked_sync_shrink_cleanup_smoke").await?;
    common::print_failure_hint("chunked_sync_shrink_cleanup_smoke", harness.artifact_dir());

    harness.fixture().set_text(
        "/chunk/isp.txt",
        concat!(
            "10.0.0.0/24\n",
            "10.0.1.0/24 # keep\n",
            "\n",
            "2001:db8::1\n",
            "10.0.2.0/24\n",
            "10.0.3.0/24\n",
            "# comment\n",
            "10.0.4.0/24\n"
        ),
    );
    harness.fixture().set_text(
        "/chunk/domain.txt",
        concat!(
            "alpha.example\n",
            "beta.example\n",
            "_skip.example\n",
            "\n",
            "gamma.example # trailing\n",
            "delta.example\n",
            "epsilon.example\n"
        ),
    );
    harness.fixture().set_text(
        "/chunk/ipv4.txt",
        concat!(
            "1.1.1.1\n",
            "1.1.1.2\n",
            "\n",
            "2001:db8::2\n",
            "1.1.1.3\n",
            "1.1.1.4\n",
            "1.1.1.5\n"
        ),
    );
    harness.fixture().set_text(
        "/chunk/ipv6.txt",
        concat!(
            "2001:db8::1\n",
            "2001:db8::2\n",
            "1.1.1.1\n",
            "2001:db8::3\n",
            "\n",
            "2001:db8::4\n",
            "2001:db8::5\n"
        ),
    );

    let config = render_test_config(
        harness.base_url(),
        harness.username(),
        harness.password(),
        &format!(
            concat!(
                "custom-isp:\n",
                "  - tag: ChunkIsp\n",
                "    url: \"{}\"\n",
                "stream-domain:\n",
                "  - interface: wan2\n",
                "    src-addr: 192.168.50.10-192.168.50.20\n",
                "    src-addr-opt-ipgroup: \"\"\n",
                "    url: \"{}\"\n",
                "    tag: ChunkDom\n",
                "ip-group:\n",
                "  - tag: Chunk4\n",
                "    url: \"{}\"\n",
                "ipv6-group:\n",
                "  - tag: Chunk6\n",
                "    url: \"{}\"\n",
                "MaxNumberOfOneRecords:\n",
                "  Isp: 2\n",
                "  Ipv4: 2\n",
                "  Ipv6: 2\n",
                "  Domain: 2\n"
            ),
            harness.fixture().url("/chunk/isp.txt"),
            harness.fixture().url("/chunk/domain.txt"),
            harness.fixture().url("/chunk/ipv4.txt"),
            harness.fixture().url("/chunk/ipv6.txt"),
        ),
    );
    let config_path = harness.write_config("chunked-sync.yml", &config)?;
    let config_path_str = config_path.to_string_lossy().to_string();

    harness.run_cli_expect_success(&["-c", &config_path_str, "-r", "once", "-m", "iip"])?;

    let api = harness.login_api().await?;

    let mut custom_isp = ikuai::custom_isp::show_custom_isp_by_tag_name(&api, "ChunkIsp")
        .await
        .map_err(|e| format!("failed to query initial custom ISP chunks: {e}"))?;
    custom_isp.sort_by_key(|item| item.id);
    assert_eq!(custom_isp.len(), 3, "expected three custom ISP chunks");
    assert_eq!(
        csv_items(&custom_isp[0].ipgroup),
        vec!["10.0.0.0/24", "10.0.1.0/24"]
    );
    let custom_isp_first_id = custom_isp[0].id;

    let mut stream_domains = ikuai::stream_domain::show_stream_domain_by_tag_name(&api, "ChunkDom")
        .await
        .map_err(|e| format!("failed to query initial stream-domain chunks: {e}"))?;
    stream_domains.sort_by_key(|item| item.id);
    assert_eq!(
        stream_domains.len(),
        3,
        "expected three stream-domain chunks"
    );
    assert_eq!(
        csv_items(&stream_domains[0].domain),
        vec!["alpha.example", "beta.example"]
    );
    let stream_domain_first_id = stream_domains[0].id;

    let mut ipv4_groups = ikuai::ip_group::show_ip_group_by_tag_name(&api, "Chunk4")
        .await
        .map_err(|e| format!("failed to query initial IPv4 chunks: {e}"))?;
    ipv4_groups.sort_by_key(|item| item.id);
    assert_eq!(ipv4_groups.len(), 3, "expected three IPv4 group chunks");
    assert_eq!(
        csv_items(&ipv4_groups[0].addr_pool),
        vec!["1.1.1.1", "1.1.1.2"]
    );
    let ipv4_first_id = ipv4_groups[0].id;

    let mut ipv6_groups = ikuai::ipv6_group::show_ipv6_group_by_tag_name(&api, "Chunk6")
        .await
        .map_err(|e| format!("failed to query initial IPv6 chunks: {e}"))?;
    ipv6_groups.sort_by_key(|item| item.id);
    assert_eq!(ipv6_groups.len(), 3, "expected three IPv6 group chunks");
    assert_eq!(
        csv_items(&ipv6_groups[0].addr_pool),
        vec!["2001:db8::1", "2001:db8::2"]
    );
    let ipv6_first_id = ipv6_groups[0].id;

    harness.fixture().set_text(
        "/chunk/isp.txt",
        concat!("10.0.9.0/24\n", "10.0.9.1/24\n", "2001:db8::dead\n"),
    );
    harness.fixture().set_text(
        "/chunk/domain.txt",
        concat!(
            "renew.example\n",
            "steady.example\n",
            "_still_skip.example\n"
        ),
    );
    harness.fixture().set_text(
        "/chunk/ipv4.txt",
        concat!("9.9.9.1\n", "9.9.9.2\n", "2001:db8::beef\n"),
    );
    harness.fixture().set_text(
        "/chunk/ipv6.txt",
        concat!("2001:db8:9::1\n", "2001:db8:9::2\n", "9.9.9.9\n"),
    );

    harness.run_cli_expect_success(&["-c", &config_path_str, "-r", "once", "-m", "iip"])?;

    let api = harness.login_api().await?;

    let mut custom_isp = ikuai::custom_isp::show_custom_isp_by_tag_name(&api, "ChunkIsp")
        .await
        .map_err(|e| format!("failed to re-query custom ISP chunks: {e}"))?;
    custom_isp.sort_by_key(|item| item.id);
    assert_eq!(
        custom_isp.len(),
        1,
        "extra custom ISP chunks should be deleted"
    );
    assert_eq!(
        custom_isp[0].id, custom_isp_first_id,
        "first custom ISP chunk should update in place"
    );
    assert_eq!(
        csv_items(&custom_isp[0].ipgroup),
        vec!["10.0.9.0/24", "10.0.9.1/24"]
    );

    let mut stream_domains = ikuai::stream_domain::show_stream_domain_by_tag_name(&api, "ChunkDom")
        .await
        .map_err(|e| format!("failed to re-query stream-domain chunks: {e}"))?;
    stream_domains.sort_by_key(|item| item.id);
    assert_eq!(
        stream_domains.len(),
        1,
        "extra stream-domain chunks should be deleted"
    );
    assert_eq!(
        stream_domains[0].id, stream_domain_first_id,
        "first stream-domain chunk should update in place"
    );
    assert_eq!(
        csv_items(&stream_domains[0].domain),
        vec!["renew.example", "steady.example"]
    );

    let mut ipv4_groups = ikuai::ip_group::show_ip_group_by_tag_name(&api, "Chunk4")
        .await
        .map_err(|e| format!("failed to re-query IPv4 chunks: {e}"))?;
    ipv4_groups.sort_by_key(|item| item.id);
    assert_eq!(ipv4_groups.len(), 1, "extra IPv4 chunks should be deleted");
    assert_eq!(
        ipv4_groups[0].id, ipv4_first_id,
        "first IPv4 chunk should update in place"
    );
    assert_eq!(
        csv_items(&ipv4_groups[0].addr_pool),
        vec!["9.9.9.1", "9.9.9.2"]
    );

    let mut ipv6_groups = ikuai::ipv6_group::show_ipv6_group_by_tag_name(&api, "Chunk6")
        .await
        .map_err(|e| format!("failed to re-query IPv6 chunks: {e}"))?;
    ipv6_groups.sort_by_key(|item| item.id);
    assert_eq!(ipv6_groups.len(), 1, "extra IPv6 chunks should be deleted");
    assert_eq!(
        ipv6_groups[0].id, ipv6_first_id,
        "first IPv6 chunk should update in place"
    );
    assert_eq!(
        csv_items(&ipv6_groups[0].addr_pool),
        vec!["2001:db8:9::1", "2001:db8:9::2"]
    );

    Ok(())
}
