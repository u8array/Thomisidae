use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn default_true() -> bool { true }
fn default_ttl_secs() -> u64 { 3600 }

#[derive(Debug, Clone, Deserialize)]
pub struct RobotsConfig {
    #[serde(default = "default_true")]
    pub obey: bool,
    #[serde(default)]
    pub user_agent: Option<String>,
    #[serde(default = "default_ttl_secs")]
    pub cache_ttl_secs: u64,
}

impl Default for RobotsConfig {
    fn default() -> Self {
        Self { obey: true, user_agent: None, cache_ttl_secs: 3600 }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct GoogleSearchConfig {
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub cse_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub features: HashMap<String, bool>,
    #[serde(default)]
    pub google_search: Option<GoogleSearchConfig>,
    #[serde(default)]
    pub robots: RobotsConfig,
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
        if let Ok(mut exe_path) = std::env::current_exe() {
            exe_path.pop();
            let exe_cfg = exe_path.join("config.toml");
            if exe_cfg.exists() {
                eprintln!(
                    "[lm_mcp_server] Using config next to executable: {}",
                    exe_cfg.display()
                );
                return Self::load_from_path(exe_cfg);
            }
        }

        eprintln!("[lm_mcp_server] No config.toml found. Using defaults (all features enabled).");
        Self::default()
    }

    pub fn is_enabled(&self, name: &str) -> bool {
        self.features.get(name).copied().unwrap_or(true)
    }
}
