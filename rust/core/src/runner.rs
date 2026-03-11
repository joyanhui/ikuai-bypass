use thiserror::Error;

#[derive(Debug, Error)]
pub enum RunError {
    #[error("Invalid -m parameter: {0}")]
    InvalidModule(String),
}

pub fn validate_module(module: &str) -> Result<(), RunError> {
    match module {
        "ispdomain" | "ipgroup" | "ipv6group" | "ii" | "ip" | "iip" => Ok(()),
        other => Err(RunError::InvalidModule(other.to_string())),
    }
}
