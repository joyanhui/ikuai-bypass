use crate::config::Config;
use crate::ikuai;
use crate::session::{resolve_login_params, LoginParamsError};

#[derive(Debug, thiserror::Error)]
pub enum CleanError {
    #[error("Clean mode requires clean_tag")]
    MissingTag,
    #[error("login params error: {0}")]
    LoginParams(#[from] LoginParamsError),
    #[error("clean step {step} failed: {source}")]
    Step {
        step: &'static str,
        #[source]
        source: ikuai::IKuaiError,
    },
}

pub async fn run_clean(cfg: &Config, cli_login: &str, clean_tag: &str) -> Result<(), CleanError> {
    let tag = clean_tag.trim();
    if tag.is_empty() {
        return Err(CleanError::MissingTag);
    }

    let params = resolve_login_params(cfg, cli_login)?;

    let api = ikuai::IKuaiClient::new(params.base_url.to_string(), &cfg.proxy)
        .map_err(|e| CleanError::Step { step: "init_client", source: e })?;
    api.login(&params.username, &params.password)
        .await
        .map_err(|e| CleanError::Step { step: "login", source: e })?;

    ikuai::custom_isp::del_custom_isp_all(&api, tag)
        .await
        .map_err(|e| CleanError::Step { step: "custom_isp", source: e })?;
    ikuai::stream_domain::del_stream_domain_all(&api, tag)
        .await
        .map_err(|e| CleanError::Step { step: "stream_domain", source: e })?;
    ikuai::ip_group::del_ikuai_bypass_ip_group(&api, tag)
        .await
        .map_err(|e| CleanError::Step { step: "ip_group", source: e })?;
    ikuai::ipv6_group::del_ikuai_bypass_ipv6_group(&api, tag)
        .await
        .map_err(|e| CleanError::Step { step: "ipv6_group", source: e })?;
    ikuai::stream_ipport::del_ikuai_bypass_stream_ipport(&api, tag)
        .await
        .map_err(|e| CleanError::Step { step: "stream_ipport", source: e })?;

    Ok(())
}
