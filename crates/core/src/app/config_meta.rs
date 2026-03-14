use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::config::Config;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConfigMeta {
    #[serde(flatten)]
    pub config: serde_json::Value,
    pub conf_path: String,
    pub raw_yaml: String,
    pub top_level_comments: BTreeMap<String, String>,
    pub item_comments: BTreeMap<String, String>,
    pub webui_comments: BTreeMap<String, String>,
    pub max_number_of_one_records_comments: BTreeMap<String, String>,
}

pub fn build_config_meta(cfg: &Config, config_path: &Path) -> Result<ConfigMeta, String> {
    let config =
        serde_json::to_value(cfg).map_err(|e| format!("Failed to encode config: {}", e))?;

    let abs_path = to_abs_path(config_path);
    let conf_path = abs_path.to_string_lossy().to_string();
    let raw_yaml = std::fs::read_to_string(&abs_path).unwrap_or_default();

    Ok(ConfigMeta {
        config,
        conf_path,
        raw_yaml,
        top_level_comments: crate::config::top_level_comments(),
        item_comments: crate::config::item_comments(),
        webui_comments: crate::config::webui_comments(),
        max_number_of_one_records_comments: crate::config::max_number_of_one_records_comments(),
    })
}

fn to_abs_path(p: &Path) -> PathBuf {
    if p.is_absolute() {
        return p.to_path_buf();
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    cwd.join(p)
}
