use std::path::Path;
use serde::Deserialize;
use anyhow::{Result, Error};

#[derive(Deserialize, Clone, Debug)]
pub struct EtcLqos {
    pub lqos_directory: String,
}

impl EtcLqos {
    pub fn load() -> Result<Self> {
        if !Path::new("/etc/lqos").exists() {
            return Err(Error::msg("You must setup /etc/lqos"));
        }
        let raw = std::fs::read_to_string("/etc/lqos")?;
        let config: Self = toml::from_str(&raw)?;
        Ok(config)
    }
}