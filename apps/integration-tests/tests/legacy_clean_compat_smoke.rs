// 覆盖点：
// 1) clean 对旧版 IKUAI_BYPASS 备注规则的兼容清理；
// 2) custom-isp / stream-domain / ip-group / ipv6-group / stream-ipport 全链路验证；
// 3) 非目标 legacy tag 不应被误删。
// Coverage:
// 1) clean-mode compatibility with legacy IKUAI_BYPASS comments.
// 2) Verifies all managed rule categories end-to-end.
// 3) Non-target legacy tags must remain untouched.
mod common;

use common::{TestHarness, render_test_config};
use ikb_core::ikuai::{self, IKuaiClient};
use serde_json::json;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn legacy_clean_compat_smoke() -> Result<(), String> {
    let harness = TestHarness::start("legacy_clean_compat_smoke").await?;
    common::print_failure_hint("legacy_clean_compat_smoke", harness.artifact_dir());

    let api = harness.login_api().await?;
    seed_legacy_rules(&api, "LegacyDrop", "10.10.10.0/24", "2001:db8:10::/64").await?;
    seed_legacy_rules(&api, "LegacyStay", "10.20.20.0/24", "2001:db8:20::/64").await?;

    let config = render_test_config(
        harness.base_url(),
        harness.username(),
        harness.password(),
        "",
    );
    let config_path = harness.write_config("legacy-clean.yml", &config)?;
    let config_path_str = config_path.to_string_lossy().to_string();

    harness.run_cli_expect_success(&[
        "-c",
        &config_path_str,
        "-r",
        "clean",
        "-tag",
        "LegacyDrop",
    ])?;

    let api = harness.login_api().await?;

    let custom_isp = ikuai::custom_isp::show_custom_isp_by_tag_name(&api, "")
        .await
        .map_err(|e| format!("failed to query custom ISP after legacy clean: {e}"))?;
    assert_eq!(
        count_legacy_tag(
            custom_isp.iter().map(|item| item.comment.as_str()),
            "LegacyDrop"
        ),
        0
    );
    assert_eq!(
        count_legacy_tag(
            custom_isp.iter().map(|item| item.comment.as_str()),
            "LegacyStay"
        ),
        1
    );

    let stream_domains = ikuai::stream_domain::show_stream_domain_by_tag_name(&api, "")
        .await
        .map_err(|e| format!("failed to query stream-domain after legacy clean: {e}"))?;
    assert_eq!(
        count_legacy_tag(
            stream_domains.iter().map(|item| item.comment.as_str()),
            "LegacyDrop"
        ),
        0
    );
    assert_eq!(
        count_legacy_tag(
            stream_domains.iter().map(|item| item.comment.as_str()),
            "LegacyStay"
        ),
        1
    );

    let ipv4_groups = ikuai::ip_group::show_ip_group_by_tag_name(&api, "")
        .await
        .map_err(|e| format!("failed to query IPv4 groups after legacy clean: {e}"))?;
    assert_eq!(
        count_legacy_tag(
            ipv4_groups.iter().map(|item| item.comment.as_str()),
            "LegacyDrop"
        ),
        0
    );
    assert_eq!(
        count_legacy_tag(
            ipv4_groups.iter().map(|item| item.comment.as_str()),
            "LegacyStay"
        ),
        1
    );

    let ipv6_groups = ikuai::ipv6_group::show_ipv6_group_by_tag_name(&api, "")
        .await
        .map_err(|e| format!("failed to query IPv6 groups after legacy clean: {e}"))?;
    assert_eq!(
        count_legacy_tag(
            ipv6_groups.iter().map(|item| item.comment.as_str()),
            "LegacyDrop"
        ),
        0
    );
    assert_eq!(
        count_legacy_tag(
            ipv6_groups.iter().map(|item| item.comment.as_str()),
            "LegacyStay"
        ),
        1
    );

    let stream_ipports = ikuai::stream_ipport::show_stream_ipport_by_tag_name(&api, "")
        .await
        .map_err(|e| format!("failed to query stream-ipport after legacy clean: {e}"))?;
    assert_eq!(
        count_legacy_tag(
            stream_ipports.iter().map(|item| item.comment.as_str()),
            "LegacyDrop"
        ),
        0
    );
    assert_eq!(
        count_legacy_tag(
            stream_ipports.iter().map(|item| item.comment.as_str()),
            "LegacyStay"
        ),
        1
    );

    Ok(())
}

async fn seed_legacy_rules(
    api: &IKuaiClient,
    tag: &str,
    ipv4_addr: &str,
    ipv6_addr: &str,
) -> Result<(), String> {
    add_legacy_custom_isp(api, tag, ipv4_addr).await?;
    add_legacy_stream_domain(api, tag).await?;
    add_legacy_route_object(api, tag, 0, ipv4_addr).await?;
    add_legacy_route_object(api, tag, 1, ipv6_addr).await?;
    add_legacy_stream_ipport(api, tag, ipv4_addr).await?;
    Ok(())
}

async fn add_legacy_custom_isp(
    api: &IKuaiClient,
    tag: &str,
    ipv4_addr: &str,
) -> Result<(), String> {
    api.call::<_, serde_json::Value>(
        ikuai::FUNC_NAME_CUSTOM_ISP,
        "add",
        &json!({
            "name": format!("legacy-custom-{tag}"),
            "ipgroup": ipv4_addr,
            "comment": legacy_comment(tag),
        }),
    )
    .await
    .map(|_| ())
    .map_err(|e| format!("failed to seed legacy custom ISP '{tag}': {e}"))
}

async fn add_legacy_stream_domain(api: &IKuaiClient, tag: &str) -> Result<(), String> {
    api.call::<_, serde_json::Value>(
        ikuai::FUNC_NAME_STREAM_DOMAIN,
        "add",
        &json!({
            "enabled": "yes",
            "tagname": format!("legacy-domain-{tag}"),
            "interface": "wan2",
            "comment": legacy_comment(tag),
            "src_addr": {"custom": ["192.168.88.10-192.168.88.20"], "object": []},
            "domain": {"custom": [format!("{tag}.legacy.example")], "object": []},
            "time": weekly_time(),
        }),
    )
    .await
    .map(|_| ())
    .map_err(|e| format!("failed to seed legacy stream-domain '{tag}': {e}"))
}

async fn add_legacy_route_object(
    api: &IKuaiClient,
    tag: &str,
    kind: i64,
    addr: &str,
) -> Result<(), String> {
    let value_key = if kind == 0 { "ip" } else { "ipv6" };
    api.call::<_, serde_json::Value>(
        ikuai::FUNC_NAME_ROUTE_OBJECT,
        "add",
        &json!({
            "group_name": format!("legacy-group-{kind}-{tag}"),
            "type": kind,
            "group_value": [{value_key: addr, "comment": ""}],
            "comment": legacy_comment(tag),
        }),
    )
    .await
    .map(|_| ())
    .map_err(|e| format!("failed to seed legacy route-object kind={kind} tag='{tag}': {e}"))
}

async fn add_legacy_stream_ipport(
    api: &IKuaiClient,
    tag: &str,
    ipv4_addr: &str,
) -> Result<(), String> {
    api.call::<_, serde_json::Value>(
        ikuai::FUNC_NAME_STREAM_IPPORT,
        "add",
        &json!({
            "enabled": "yes",
            "tagname": format!("legacy-route-{tag}"),
            "interface": "",
            "nexthop": "192.168.1.2",
            "iface_band": 0,
            "comment": legacy_comment(tag),
            "type": 1,
            "mode": 0,
            "protocol": "tcp+udp",
            "src_addr": {"custom": ["192.168.66.10-192.168.66.20"], "object": []},
            "dst_addr": {"custom": [ipv4_addr], "object": []},
            "src_port": {"custom": [], "object": []},
            "dst_port": {"custom": [], "object": []},
            "time": weekly_time(),
        }),
    )
    .await
    .map(|_| ())
    .map_err(|e| format!("failed to seed legacy stream-ipport '{tag}': {e}"))
}

fn legacy_comment(tag: &str) -> String {
    format!("{}_{}", ikuai::COMMENT_IKUAI_BYPASS, tag)
}

fn weekly_time() -> serde_json::Value {
    json!({
        "custom": [{
            "type": "weekly",
            "weekdays": "1234567",
            "start_time": "00:00",
            "end_time": "23:59",
            "comment": "",
        }],
        "object": [],
    })
}

fn count_legacy_tag<'a>(comments: impl Iterator<Item = &'a str>, tag: &str) -> usize {
    comments.filter(|comment| comment.contains(tag)).count()
}
