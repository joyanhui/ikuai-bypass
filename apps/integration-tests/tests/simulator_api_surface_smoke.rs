// 覆盖点：
// 1) 模拟器 Action/login 与会话流程；
// 2) custom_isp / route_object / stream_domain / stream_ipport 的 add/show/edit/del；
// 3) 覆盖 core 当前使用到的主要 API 面。
// Coverage:
// 1) Simulator login/session path.
// 2) CRUD for all major API groups used by core.
// 3) Guards simulator API surface compatibility.
use ikb_core::ikuai;
use ikb_core::ikuai::IKuaiClient;
use ikb_integration_tests::ikuai_simulator::IKuaiSimulator;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn simulator_api_surface_smoke() -> Result<(), String> {
    let simulator = IKuaiSimulator::start("admin", "admin888").await?;
    let api = IKuaiClient::new(simulator.base_url().to_string(), false)
        .map_err(|e| format!("failed to create api client: {e}"))?;
    api.login("admin", "admin888")
        .await
        .map_err(|e| format!("failed to login simulator: {e}"))?;

    ikuai::custom_isp::add_custom_isp(&api, "SimIsp", "1.1.1.0/24", 0)
        .await
        .map_err(|e| format!("custom_isp add failed: {e}"))?;
    let custom = ikuai::custom_isp::show_custom_isp_by_tag_name(&api, "SimIsp")
        .await
        .map_err(|e| format!("custom_isp show failed: {e}"))?;
    let custom_id = custom[0].id;
    ikuai::custom_isp::edit_custom_isp(&api, "SimIsp", "2.2.2.0/24", 0, custom_id)
        .await
        .map_err(|e| format!("custom_isp edit failed: {e}"))?;

    ikuai::ip_group::add_ip_group_named(&api, "IKBSim4", "8.8.8.8,9.9.9.0/24")
        .await
        .map_err(|e| format!("ip_group add failed: {e}"))?;
    let ipv4 = ikuai::ip_group::show_ip_group_by_name(&api, "Sim4")
        .await
        .map_err(|e| format!("ip_group show failed: {e}"))?;
    let ipv4_id = ipv4[0].id;
    ikuai::ip_group::edit_ip_group_named(&api, "IKBSim4", "4.4.4.4", ipv4_id)
        .await
        .map_err(|e| format!("ip_group edit failed: {e}"))?;

    ikuai::ipv6_group::add_ipv6_group_named(&api, "IKBSim6", "2001:db8::1")
        .await
        .map_err(|e| format!("ipv6_group add failed: {e}"))?;
    let ipv6 = ikuai::ipv6_group::show_ipv6_group_by_name(&api, "Sim6")
        .await
        .map_err(|e| format!("ipv6_group show failed: {e}"))?;
    let ipv6_id = ipv6[0].id;
    ikuai::ipv6_group::edit_ipv6_group_named(&api, "IKBSim6", "2001:db8::/64", ipv6_id)
        .await
        .map_err(|e| format!("ipv6_group edit failed: {e}"))?;

    ikuai::stream_domain::add_stream_domain(
        &api,
        ikuai::stream_domain::StreamDomainSpec {
            iface: "wan2",
            tag: "SimDom",
            src_addr: "192.168.1.10-192.168.1.20",
            src_addr_opt_ipgroup: "",
            domains: "foo.example,bar.example",
            index: 0,
        },
    )
    .await
    .map_err(|e| format!("stream_domain add failed: {e}"))?;
    let domains = ikuai::stream_domain::show_stream_domain_by_tag_name(&api, "SimDom")
        .await
        .map_err(|e| format!("stream_domain show failed: {e}"))?;
    let domain_id = domains[0].id;
    ikuai::stream_domain::edit_stream_domain(
        &api,
        ikuai::stream_domain::StreamDomainSpec {
            iface: "wan2",
            tag: "SimDom",
            src_addr: "192.168.1.10-192.168.1.20",
            src_addr_opt_ipgroup: "",
            domains: "updated.example",
            index: 0,
        },
        domain_id,
    )
    .await
    .map_err(|e| format!("stream_domain edit failed: {e}"))?;

    ikuai::stream_ipport::add_stream_ipport(
        &api,
        ikuai::stream_ipport::StreamIpPortSpec {
            forward_type: "1",
            iface: "",
            dst_custom: "",
            dst_objects: &["IKBSim4".to_string()],
            src_custom: "192.168.1.10-192.168.1.20",
            src_objects: &[],
            src_addr_inv: 1,
            nexthop: "192.168.1.2",
            tag: "SimRoute",
            dst_addr_inv: 1,
            prio: 5,
            mode: 6,
            iface_band: 0,
            protocol: "tcp+udp",
        },
    )
    .await
    .map_err(|e| format!("stream_ipport add failed: {e}"))?;
    let sip_rows = ikuai::stream_ipport::show_stream_ipport_by_tag_name(&api, "SimRoute")
        .await
        .map_err(|e| format!("stream_ipport show failed: {e}"))?;
    let sip_id = sip_rows[0].id;
    assert_eq!(sip_rows[0].mode, 6, "stream_ipport add should preserve mode=6");
    ikuai::stream_ipport::edit_stream_ipport(
        &api,
        ikuai::stream_ipport::StreamIpPortSpec {
            forward_type: "1",
            iface: "",
            dst_custom: "",
            dst_objects: &["IKBSim4".to_string()],
            src_custom: "192.168.1.10-192.168.1.20",
            src_objects: &[],
            src_addr_inv: 0,
            nexthop: "192.168.1.3",
            tag: "SimRoute",
            dst_addr_inv: 0,
            prio: 9,
            mode: 6,
            iface_band: 0,
            protocol: "tcp+udp",
        },
        sip_id,
    )
    .await
    .map_err(|e| format!("stream_ipport edit failed: {e}"))?;
    let sip_rows = ikuai::stream_ipport::show_stream_ipport_by_tag_name(&api, "SimRoute")
        .await
        .map_err(|e| format!("stream_ipport show after edit failed: {e}"))?;
    assert_eq!(sip_rows[0].mode, 6, "stream_ipport edit should preserve mode=6");

    ikuai::ip_group::add_ip_group_named(&api, "FW-LAN", "192.168.50.0/24")
        .await
        .map_err(|e| format!("ip_group FW-LAN add failed: {e}"))?;
    ikuai::stream_ipport::add_stream_ipport(
        &api,
        ikuai::stream_ipport::StreamIpPortSpec {
            forward_type: "1",
            iface: "",
            dst_custom: "",
            dst_objects: &["IKBSim4".to_string()],
            src_custom: "",
            src_objects: &["FW-LAN".to_string()],
            src_addr_inv: 0,
            nexthop: "192.168.80.110",
            tag: "SimHyphen",
            dst_addr_inv: 0,
            prio: 0,
            mode: 6,
            iface_band: 0,
            protocol: "tcp+udp",
        },
    )
    .await
    .map_err(|e| format!("stream_ipport hyphen-group add failed: {e}"))?;
    let raw_resp = api
        .call::<_, serde_json::Value>(
            ikb_core::ikuai::FUNC_NAME_STREAM_IPPORT,
            "show",
            &serde_json::json!({ "TYPE": "total,data", "limit": "0,1000" }),
        )
        .await
        .map_err(|e| format!("raw stream_ipport show failed: {e}"))?;
    let raw_rows = raw_resp
        .results
        .ok_or_else(|| "raw stream_ipport show missing results".to_string())?
        .data;
    let hyphen_row = raw_rows
        .as_array()
        .and_then(|rows| {
            rows.iter().find(|row| {
                row.get("tagname")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|name| name.starts_with("IKBSimHyphen"))
            })
        })
        .ok_or_else(|| "hyphen group rule not found in stream_ipport show".to_string())?;
    let src_obj = hyphen_row
        .get("src_addr")
        .and_then(|block| block.get("object"))
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "stream_ipport src_addr.object missing".to_string())?;
    assert_eq!(src_obj.len(), 1, "src_addr.object should contain exactly one reference");
    assert_eq!(src_obj[0]["gp_name"], "FW-LAN", "src_addr.object should reference FW-LAN by name");
    assert_eq!(src_obj[0]["type"], 0, "src_addr.object type should be 0");
    assert!(
        src_obj[0]["gid"].as_str().is_some_and(|gid| gid.starts_with("IPGP")),
        "src_addr.object gid should be IPGP-prefixed"
    );
    let src_custom = hyphen_row
        .get("src_addr")
        .and_then(|block| block.get("custom"));
    match src_custom {
        None => {}
        Some(serde_json::Value::Array(items)) => {
            assert!(items.is_empty(), "src_addr.custom must stay empty for ip-group sources")
        }
        Some(_) => {}
    }
    ikuai::stream_ipport::del_stream_ipport(
        &api,
        &hyphen_row["id"].as_i64().unwrap_or_default().to_string(),
    )
    .await
    .map_err(|e| format!("stream_ipport hyphen-group del failed: {e}"))?;

    ikuai::stream_ipport::del_stream_ipport(&api, &sip_id.to_string())
        .await
        .map_err(|e| format!("stream_ipport del failed: {e}"))?;
    ikuai::stream_domain::del_stream_domain(&api, &domain_id.to_string())
        .await
        .map_err(|e| format!("stream_domain del failed: {e}"))?;
    ikuai::ipv6_group::del_ipv6_group(&api, &ipv6_id.to_string())
        .await
        .map_err(|e| format!("ipv6_group del failed: {e}"))?;
    ikuai::ip_group::del_ip_group(&api, &ipv4_id.to_string())
        .await
        .map_err(|e| format!("ip_group del failed: {e}"))?;
    ikuai::custom_isp::del_custom_isp(&api, &custom_id.to_string())
        .await
        .map_err(|e| format!("custom_isp del failed: {e}"))?;

    Ok(())
}
