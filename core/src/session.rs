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
    #[error("default gateway not found")]
    GatewayNotFound,
}

pub fn resolve_login_params(
    cfg: &Config,
    cli_login: &str,
) -> Result<LoginParams, LoginParamsError> {
    let cli_login = cli_login.trim();
    if !cli_login.is_empty() {
        let parts: Vec<&str> = cli_login.split(',').collect();
        if parts.len() != 3 {
            return Err(LoginParamsError::CliFormat);
        }
        return Ok(LoginParams {
            base_url: parts[0].trim().to_string(),
            username: parts[1].trim().to_string(),
            password: parts[2].trim().to_string(),
            source: LoginSource::Cli,
        });
    }

    if !cfg.ikuai_url.trim().is_empty() {
        return Ok(LoginParams {
            base_url: cfg.ikuai_url.trim().to_string(),
            username: cfg.username.clone(),
            password: cfg.password.clone(),
            source: LoginSource::Config,
        });
    }

    let gw = crate::router::get_gateway_v4().map_err(|_| LoginParamsError::GatewayNotFound)?;
    Ok(LoginParams {
        base_url: format!("http://{}", gw),
        username: cfg.username.clone(),
        password: cfg.password.clone(),
        source: LoginSource::Gateway,
    })
}
