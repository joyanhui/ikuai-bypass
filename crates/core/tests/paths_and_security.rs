use std::path::Path;

use ikb_core::config::{Config, ConfigError, validate_save_path};
use ikb_core::paths;

#[test]
fn default_path_has_expected_suffix() {
    let p = paths::default_config_path();
    let s = p.to_string_lossy();
    assert!(
        s.ends_with("ikuai-bypass/config.yml")
            || s.ends_with("ikuai-bypass\\config.yml")
            || s.ends_with("./config.yml")
    );
}

#[test]
fn invalid_extension_is_rejected() {
    let p = Path::new("/tmp/ikuai-bypass.config");
    let err = match validate_save_path(p) {
        Ok(_) => panic!("should reject extension"),
        Err(e) => e,
    };
    assert!(matches!(err, ConfigError::InvalidExtension));
}

#[cfg(unix)]
#[test]
fn symlink_is_rejected() {
    use std::fs;
    use std::io::Write;

    let pid = unsafe { libc::getpid() };
    let base = format!("/tmp/ikb-rs-test-{}", pid);
    let _ = fs::remove_dir_all(&base);
    if let Err(e) = fs::create_dir_all(&base) {
        panic!("mkdir failed: {}", e);
    }

    let target = format!("{}/target.yml", base);
    let link = format!("{}/config.yml", base);

    {
        let mut f = match fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&target)
        {
            Ok(v) => v,
            Err(e) => panic!("write target failed: {}", e),
        };
        let _ = f.write_all(b"a: 1\n");
    }

    if let Err(e) = std::os::unix::fs::symlink(&target, &link) {
        panic!("symlink failed: {}", e);
    }

    let err = match validate_save_path(Path::new(&link)) {
        Ok(_) => panic!("should reject symlink"),
        Err(e) => e,
    };
    assert!(matches!(err, ConfigError::SymlinkDenied));
}

#[test]
fn save_raw_yaml_creates_missing_parent_dirs() {
    use std::fs;

    let pid = std::process::id();
    let base = std::env::temp_dir().join(format!("ikb-rs-save-dir-{}", pid));
    let _ = fs::remove_dir_all(&base);

    let config_path = base.join("nested").join("config.yml");
    let yaml = r#"
ikuai-url: http://192.168.9.1
username: admin
password: pass
cron: ""
"#;

    let cfg = Config::validate_and_save_raw_yaml(yaml, &config_path)
        .unwrap_or_else(|e| panic!("save should create parent dirs: {e}"));

    assert_eq!(cfg.ikuai_url, "http://192.168.9.1");
    assert!(config_path.is_file(), "config file should be created");

    let saved = fs::read_to_string(&config_path).unwrap_or_else(|e| {
        panic!(
            "failed to read saved config '{}': {e}",
            config_path.display()
        )
    });
    assert!(saved.contains("ikuai-url: http://192.168.9.1"));

    let _ = fs::remove_dir_all(&base);
}
