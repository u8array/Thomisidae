use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub features: HashMap<String, bool>,
}

impl Config {
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        match fs::read_to_string(path) {
            Ok(contents) => match toml::from_str::<Config>(&contents) {
                Ok(cfg) => cfg,
                Err(err) => {
                    eprintln!(
                        "[lm_mcp_server] Failed to parse config at '{}': {}. Using defaults.",
                        path.display(),
                        err
                    );
                    Self::default()
                }
            },
            Err(_) => Self::default(),
        }
    }

    pub fn load_default() -> Self {
        if let Ok(p) = std::env::var("LM_MCP_CONFIG") {
            return Self::load_from_path(p);
        }
        let default = PathBuf::from("config.toml");
        Self::load_from_path(default)
    }

    pub fn is_enabled(&self, name: &str) -> bool {
        self.features.get(name).copied().unwrap_or(true)
    }
}
