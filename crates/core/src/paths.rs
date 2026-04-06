use std::path::{Path, PathBuf};

pub fn default_config_path() -> PathBuf {
    default_config_path_with_app_name("ikuai-bypass")
}

pub fn default_config_path_with_app_name(app_name: &str) -> PathBuf {
    if let Some(base) = platform_config_dir_no_env() {
        return base.join(app_name).join("config.yml");
    }
    PathBuf::from("./config.yml")
}

pub fn config_path_from_base_dir(base: impl AsRef<Path>, app_name: &str) -> PathBuf {
    base.as_ref().join(app_name).join("config.yml")
}

fn platform_config_dir_no_env() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        return windows_roaming_appdata();
    }

    #[cfg(target_os = "macos")]
    {
        return unix_home_dir_no_env().map(|h| h.join("Library").join("Application Support"));
    }

    // Android 沙箱环境中 getpwuid_r 不可靠，且应用应使用 Tauri 提供的路径 API。
    // 返回 None 让调用方 fallback 到 ./config.yml 或由 Tauri 层覆写。
    // Android sandbox: getpwuid_r is unreliable; caller should use Tauri's path API instead.
    #[cfg(target_os = "android")]
    {
        return None;
    }

    #[cfg(all(unix, not(target_os = "macos"), not(target_os = "android")))]
    {
        return unix_home_dir_no_env().map(|h| h.join(".config"));
    }

    #[allow(unreachable_code)]
    None
}

#[cfg(unix)]
fn unix_home_dir_no_env() -> Option<PathBuf> {
    use std::ffi::CStr;
    use std::os::raw::c_char;

    let uid = unsafe { libc::getuid() };
    let mut buf_len = unsafe { libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) };
    if buf_len <= 0 {
        buf_len = 16 * 1024;
    }
    let mut buf = vec![0u8; buf_len as usize];
    let mut pwd: libc::passwd = unsafe { std::mem::zeroed() };
    let mut result: *mut libc::passwd = std::ptr::null_mut();
    let rc = unsafe {
        libc::getpwuid_r(
            uid,
            &mut pwd,
            buf.as_mut_ptr() as *mut c_char,
            buf.len(),
            &mut result,
        )
    };
    if rc != 0 || result.is_null() || pwd.pw_dir.is_null() {
        return None;
    }
    let dir = unsafe { CStr::from_ptr(pwd.pw_dir) }.to_str().ok()?;
    if dir.is_empty() {
        return None;
    }
    Some(PathBuf::from(dir))
}

#[cfg(windows)]
fn windows_roaming_appdata() -> Option<PathBuf> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    use windows_sys::Win32::System::Com::{
        COINIT_APARTMENTTHREADED, CoInitializeEx, CoTaskMemFree,
    };
    use windows_sys::Win32::UI::Shell::{
        FOLDERID_RoamingAppData, KF_FLAG_DEFAULT, SHGetKnownFolderPath,
    };
    use windows_sys::core::PWSTR;

    unsafe {
        let _ = CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED as u32);
        let mut path_ptr: PWSTR = std::ptr::null_mut();
        let hr = SHGetKnownFolderPath(
            &FOLDERID_RoamingAppData,
            KF_FLAG_DEFAULT as u32,
            std::ptr::null_mut(),
            &mut path_ptr,
        );
        if hr < 0 || path_ptr.is_null() {
            return None;
        }
        let mut len = 0usize;
        while *path_ptr.add(len) != 0 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(path_ptr, len);
        let os = OsString::from_wide(slice);
        CoTaskMemFree(path_ptr.cast());
        Some(PathBuf::from(os))
    }
}
