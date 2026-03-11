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

#[cfg(test)]
mod tests {
    use super::{resolve_login_params, LoginSource};
    use crate::config::{Config, MaxNumberOfOneRecordsConfig, WebUiConfig};

    fn base_config() -> Config {
        Config {
            ikuai_url: "http://192.168.9.1".to_string(),
            username: "admin".to_string(),
            password: "pass".to_string(),
            cron: String::new(),
            add_err_retry_wait: std::time::Duration::from_secs(0),
            add_wait: std::time::Duration::from_secs(0),
            github_proxy: String::new(),
            custom_isp: Vec::new(),
            stream_domain: Vec::new(),
            ip_group: Vec::new(),
            ipv6_group: Vec::new(),
            stream_ipport: Vec::new(),
            webui: WebUiConfig::default(),
            max_number_of_one_records: MaxNumberOfOneRecordsConfig::default(),
        }
    }

    #[test]
    fn cli_login_overrides_config() {
        let cfg = base_config();
        let p = resolve_login_params(&cfg, "http://1.2.3.4,u,p").expect("ok");
        assert_eq!(p.base_url, "http://1.2.3.4");
        assert_eq!(p.source, LoginSource::Cli);
    }

    #[test]
    fn config_login_used_when_present() {
        let cfg = base_config();
        let p = resolve_login_params(&cfg, "").expect("ok");
        assert_eq!(p.base_url, "http://192.168.9.1");
        assert_eq!(p.source, LoginSource::Config);
    }
}
