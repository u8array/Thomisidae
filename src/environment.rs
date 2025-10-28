use std::path::{Path, PathBuf};
use crate::config::Config;

pub fn load_env() {
    let _ = dotenvy::dotenv();

    if let Ok(mut exe_path) = std::env::current_exe() {
        exe_path.pop();
        let env_path: PathBuf = exe_path.join(".env");
        if env_path.exists() {
            let _ = dotenvy::from_path(&env_path);
        }
    }
}

pub fn load_from_path<P: AsRef<Path>>(path: P) -> bool {
    dotenvy::from_path(path.as_ref()).is_ok()
}

pub fn var(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

pub fn get_google_api_key(cfg: &Config) -> Option<String> {
    var("GOOGLE_API_KEY").or_else(|| cfg.google_search.as_ref().and_then(|g| g.api_key.clone()))
}

pub fn get_google_cse_id(cfg: &Config) -> Option<String> {
    var("GOOGLE_CSE_ID").or_else(|| cfg.google_search.as_ref().and_then(|g| g.cse_id.clone()))
}
