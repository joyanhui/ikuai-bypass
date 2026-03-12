use std::path::Path;
use std::time::Duration;

use ikb_core::config::Config;

#[test]
fn load_repo_config_yml() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("config.yml");
    let cfg = match Config::load_from_path(&path) {
        Ok(v) => v,
        Err(e) => panic!("load config.yml failed: {}", e),
    };
    assert!(!cfg.ikuai_url.is_empty());
    assert!(cfg.max_number_of_one_records.isp > 0);
}

#[test]
fn defaults_match_go_expectations() {
    let yaml = r#"
ikuai-url: http://192.168.9.1
username: admin
password: pass
cron: ""
webui: {}
MaxNumberOfOneRecords: {}
stream-domain:
  - interface: wan1
    url: https://example.com
"#;

    let mut cfg: Config = match serde_yaml::from_str(yaml) {
        Ok(v) => v,
        Err(e) => panic!("parse failed: {}", e),
    };
    cfg.apply_defaults();

    assert_eq!(cfg.webui.cdn_prefix, "https://cdn.jsdelivr.net/npm");
    assert_eq!(cfg.max_number_of_one_records.isp, 5000);
    assert_eq!(cfg.max_number_of_one_records.ipv4, 1000);
    assert_eq!(cfg.max_number_of_one_records.ipv6, 1000);
    assert_eq!(cfg.max_number_of_one_records.domain, 5000);
    assert_eq!(
        cfg.stream_domain.get(0).map(|v| v.tag.as_str()),
        Some("wan1")
    );
    assert_eq!(cfg.add_wait, Duration::from_secs(0));
}
