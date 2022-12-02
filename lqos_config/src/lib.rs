use anyhow::{Error, Result};
use std::{fs, path::Path};

const DEFAULT_DIR: &str = "/opt/libreqos/v1.3/ispConfig.py";

pub struct LibreQoSConfig {
    pub internet_interface: String,
    pub isp_interface: String,
}

impl LibreQoSConfig {
    pub fn load_from_default() -> Result<Self> {
        let path = Path::new(DEFAULT_DIR);
        if !path.exists() {
            return Err(Error::msg("Unable to find ispConfig.py"));
        }

        // Read the config
        let mut result = Self {
            internet_interface: String::new(),
            isp_interface: String::new(),
        };
        result.parse_isp_config(path)?;
        Ok(result)
    }

    pub fn load_from_path(path: &str) -> Result<Self> {
        let path = Path::new(path);
        if !path.exists() {
            return Err(Error::msg("Unable to find ispConfig.py"));
        }

        // Read the config
        let mut result = Self {
            internet_interface: String::new(),
            isp_interface: String::new(),
        };
        result.parse_isp_config(path)?;
        Ok(result)
    }

    fn parse_isp_config(&mut self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)?;
        for line in content.split("\n") {
            if line.starts_with("interfaceA") {
                self.isp_interface = split_at_equals(line);
            }
            if line.starts_with("interfaceB") {
                self.internet_interface = split_at_equals(line);
            }
        }
        Ok(())
    }
}

fn split_at_equals(line: &str) -> String {
    line.split('=')
        .nth(1)
        .unwrap_or("")
        .trim()
        .replace("\"", "")
}
