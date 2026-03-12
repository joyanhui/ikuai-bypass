use std::path::Path;

use ikb_core::config::{validate_save_path, ConfigError};
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
    let err = validate_save_path(p).expect_err("should reject extension");
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
    fs::create_dir_all(&base).expect("mkdir");

    let target = format!("{}/target.yml", base);
    let link = format!("{}/config.yml", base);

    {
        let mut f = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&target)
            .expect("write target");
        let _ = f.write_all(b"a: 1\n");
    }

    std::os::unix::fs::symlink(&target, &link).expect("symlink");

    let err = validate_save_path(Path::new(&link)).expect_err("should reject symlink");
    assert!(matches!(err, ConfigError::SymlinkDenied));
}
