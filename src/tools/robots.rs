use std::collections::HashMap;
use std::time::{Duration, Instant};

use mcp_protocol_sdk::prelude::*;
use reqwest::Client;
use robotstxt::DefaultMatcher;
use tokio::sync::RwLock;
use url::Url;

#[derive(Debug)]
pub struct Robots {
    client: Client,
    user_agent: String,
    obey: bool,
    ttl: Duration,
    cache: RwLock<HashMap<String, CacheEntry>>,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    fetched_at: Instant,
    body: String,
}

impl Robots {
    pub fn new(client: Client, user_agent: String, obey: bool, ttl_secs: u64) -> Self {
        Self {
            client,
            user_agent,
            obey,
            ttl: Duration::from_secs(ttl_secs),
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub async fn allow(&self, url: &Url) -> McpResult<bool> {
        if !self.obey {
            return Ok(true);
        }
        if !matches!(url.scheme(), "http" | "https") {
            return Ok(true);
        }
        let origin = origin_key(url);

        if let Some(allow) = {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(&origin) {
                if entry.fresh(self.ttl) {
                    let mut matcher = DefaultMatcher::default();
                    Some(matcher.one_agent_allowed_by_robots(&entry.body, &self.user_agent, url.as_str()))
                } else {
                    None
                }
            } else {
                None
            }
        } {
            return Ok(allow);
        }

        let body: String = self.fetch_robots_body_for(&origin).await.unwrap_or_default();

        {
            let mut cache = self.cache.write().await;
            cache.insert(
                origin.clone(),
                CacheEntry { fetched_at: Instant::now(), body: body.clone() },
            );
        }

        if body.is_empty() {
            return Ok(true);
        }
        let mut matcher = DefaultMatcher::default();
        Ok(matcher.one_agent_allowed_by_robots(&body, &self.user_agent, url.as_str()))
    }

    async fn fetch_robots_body_for(&self, origin: &str) -> McpResult<String> {
        let robots_url = format!("{origin}/robots.txt");
        let resp = self
            .client
            .get(&robots_url)
            .send()
            .await
            .map_err(|e| McpError::internal(e.to_string()))?;

        if !resp.status().is_success() {
            return Ok(String::new());
        }

        let text = resp
            .text()
            .await
            .map_err(|e| McpError::internal(e.to_string()))?;
        Ok(text)
    }
}

impl CacheEntry {
    fn fresh(&self, ttl: Duration) -> bool {
        self.fetched_at.elapsed() < ttl
    }
}

fn origin_key(url: &Url) -> String {
    let scheme = url.scheme();
    let host = url.host_str().unwrap_or("");
    let port = url.port_or_known_default().unwrap_or_default();
    if port == 0 {
        format!("{scheme}://{host}")
    } else {
        format!("{scheme}://{host}:{port}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn robotstxt_quickstart() {
        let body = "user-agent: FooBot\ndisallow: /\n";
        let mut matcher = DefaultMatcher::default();
        assert_eq!(false, matcher.one_agent_allowed_by_robots(body, "FooBot", "https://foo.com/"));
    }
}
