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

    #[cfg(all(unix, not(target_os = "macos")))]
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

    let dir_cstr = unsafe { CStr::from_ptr(pwd.pw_dir) };
    let dir = dir_cstr.to_str().ok()?;
    if dir.is_empty() {
        return None;
    }
    Some(PathBuf::from(dir))
}

#[cfg(windows)]
fn windows_roaming_appdata() -> Option<PathBuf> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    use windows_sys::Win32::Foundation::PWSTR;
    use windows_sys::Win32::System::Com::{
        CoInitializeEx, CoTaskMemFree, COINIT_APARTMENTTHREADED,
    };
    use windows_sys::Win32::UI::Shell::{
        FOLDERID_RoamingAppData, SHGetKnownFolderPath, KF_FLAG_DEFAULT,
    };

    unsafe {
        let _ = CoInitializeEx(std::ptr::null_mut(), COINIT_APARTMENTTHREADED);

        let mut path_ptr: PWSTR = std::ptr::null_mut();
        let hr = SHGetKnownFolderPath(&FOLDERID_RoamingAppData, KF_FLAG_DEFAULT, 0, &mut path_ptr);
        if hr < 0 || path_ptr.is_null() {
            return None;
        }

        let mut len = 0usize;
        while *path_ptr.add(len) != 0 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(path_ptr, len);
        let os = OsString::from_wide(slice);
        CoTaskMemFree(path_ptr as *mut _);
        Some(PathBuf::from(os))
    }
}
