use std::sync::Arc;

use mcp_protocol_sdk::prelude::*;
use url::Url;

use crate::config::Config;

#[derive(Debug, Clone, Default)]
pub struct DomainPolicy {
    allowed: Arc<Vec<String>>,
    blocked: Arc<Vec<String>>,
}

impl DomainPolicy {
    pub fn from_config(cfg: &Config) -> Self {
        let mut allowed = cfg
            .allowed_domains
            .iter()
            .map(|s| s.trim().trim_matches('.').to_ascii_lowercase())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        let mut blocked = cfg
            .blocked_domains
            .iter()
            .map(|s| s.trim().trim_matches('.').to_ascii_lowercase())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        allowed.sort();
        allowed.dedup();
        blocked.sort();
        blocked.dedup();
        Self { allowed: Arc::new(allowed), blocked: Arc::new(blocked) }
    }

    pub fn allows_url(&self, url: &Url) -> bool {
        let host = match url.host_str() {
            Some(h) => h.to_ascii_lowercase(),
            None => return true,
        };
        self.allows_host(&host)
    }

    pub fn allows_host(&self, host: &str) -> bool {
        let host = host.trim_end_matches('.').to_ascii_lowercase();

        if self.matches_any(&host, &self.blocked) {
            return false;
        }

        if self.allowed.is_empty() {
            return true;
        }

        self.matches_any(&host, &self.allowed)
    }

    fn matches_any(&self, host: &str, patterns: &[String]) -> bool {
        patterns.iter().any(|pat| domain_matches(host, pat))
    }

    pub fn is_empty(&self) -> bool {
        self.allowed.is_empty() && self.blocked.is_empty()
    }

    pub fn describe(&self) -> String {
        let allowed = if self.allowed.is_empty() {
            "(all domains allowed unless blocked)".to_string()
        } else {
            self.allowed.join(", ")
        };
        let blocked = if self.blocked.is_empty() {
            "(none)".to_string()
        } else {
            self.blocked.join(", ")
        };
        format!("Allowed: {allowed}\nBlocked: {blocked}")
    }

    pub fn validation_error_message(&self, host: &str) -> String {
        let allowed_note = if self.allowed.is_empty() {
            "all domains allowed unless blocked".to_string()
        } else {
            format!("allowed: {}", self.allowed.join(", "))
        };
        let blocked_note = if self.blocked.is_empty() {
            "none".to_string()
        } else {
            self.blocked.join(", ")
        };
        format!(
            "Access to domain '{host}' is not permitted by policy (allowed: {allowed_note}; blocked: {blocked_note})"
        )
    }
}

fn domain_matches(host: &str, pat: &str) -> bool {
    host == pat || host.ends_with(&format!(".{pat}"))
}

pub fn ensure_allowed(policy: &DomainPolicy, url: &Url) -> McpResult<()> {
    if !policy.allows_url(url) {
        let host = url.host_str().unwrap_or("");
        return Err(McpError::validation(policy.validation_error_message(host)));
    }
    Ok(())
}
