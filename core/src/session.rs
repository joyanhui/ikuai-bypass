use thiserror::Error;

use crate::config::Config;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginParams {
    pub base_url: String,
    pub username: String,
    pub password: String,
    pub source: LoginSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginSource {
    Cli,
    Config,
    Gateway,
}

#[derive(Debug, Error)]
pub enum LoginParamsError {
    #[error("command line parameter format error")]
    CliFormat,
    #[error("ikuai-url is empty in config file")]
    MissingIkuaiUrl,
    #[error("default gateway not found")]
    GatewayNotFound,
}

pub fn resolve_login_params(
    cfg: &Config,
    cli_login: &str,
) -> Result<LoginParams, LoginParamsError> {
    let cli_login = cli_login.trim();
    if !cli_login.is_empty() {
        let mut parts = cli_login.split(',').map(str::trim);
        let base_url = parts.next().unwrap_or("");
        let username = parts.next().unwrap_or("");
        let password = parts.next().unwrap_or("");
        if base_url.is_empty()
            || username.is_empty()
            || password.is_empty()
            || parts.next().is_some()
        {
            return Err(LoginParamsError::CliFormat);
        }
        return Ok(LoginParams {
            base_url: base_url.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            source: LoginSource::Cli,
        });
    }

    if !cfg.ikuai_url.trim().is_empty() {
        return Ok(LoginParams {
            base_url: cfg.ikuai_url.trim().to_string(),
            username: cfg.username.to_string(),
            password: cfg.password.to_string(),
            source: LoginSource::Config,
        });
    }

    // 移动端不尝试通过默认网关猜测爱快地址，避免误判/不可用。
    // Mobile: do not guess iKuai URL via default gateway.
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        return Err(LoginParamsError::MissingIkuaiUrl);
    }

    let gw = crate::router::get_gateway_v4().map_err(|_| LoginParamsError::GatewayNotFound)?;
    Ok(LoginParams {
        base_url: format!("http://{}", gw),
        username: cfg.username.to_string(),
        password: cfg.password.to_string(),
        source: LoginSource::Gateway,
    })
}
