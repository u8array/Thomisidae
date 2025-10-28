use async_trait::async_trait;
use mcp_protocol_sdk::prelude::*;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::config::Config;
use super::meta::{ToolInputSchema, ToolMeta};
use super::utils::{required_str_arg, text_tool_result};

static META: OnceLock<ToolMeta> = OnceLock::new();

pub fn meta() -> ToolMeta {
    META.get_or_init(|| {
        let properties = serde_json::json!({
            "query": { "type": "string", "description": "The search query (Google)." },
            "num": { "type": "integer", "minimum": 1, "maximum": 10, "default": 5, "description": "Number of results to return (1-10)." },
            "site": { "type": "string", "description": "Optional site/domain to restrict results, e.g., 'example.com'." },
            "format": { "type": "string", "enum": ["text", "json"], "default": "text" }
        });
        let schema = ToolInputSchema::new("object", properties, vec!["query".to_string()]);
        ToolMeta::new(
            "google_search",
            "Google Search",
            "Search the web using Google Programmable Search (Custom Search API).",
            schema,
        )
    })
    .clone()
}

pub struct GoogleSearchHandler {
    pub client: Client,
    pub api_key: Option<String>,
    pub cse_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleSearchResponse {
    items: Option<Vec<GoogleItem>>,    
}

#[derive(Debug, Deserialize)]
struct GoogleItem {
    title: Option<String>,
    link: Option<String>,
    snippet: Option<String>,
}

#[async_trait]
impl ToolHandler for GoogleSearchHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let query = required_str_arg(&arguments, "query")?;
        let num = arguments.get("num").and_then(|v| v.as_u64()).unwrap_or(5).min(10) as u8;
        let site = arguments.get("site").and_then(|v| v.as_str()).map(|s| s.trim()).filter(|s| !s.is_empty());
        let format = arguments.get("format").and_then(|v| v.as_str()).unwrap_or("text");

        let api_key_owned = self
            .api_key
            .clone()
            .or_else(|| std::env::var("GOOGLE_API_KEY").ok());
        let cse_id_owned = self
            .cse_id
            .clone()
            .or_else(|| std::env::var("GOOGLE_CSE_ID").ok());

        let api_key = api_key_owned.ok_or_else(|| McpError::validation("Google API key not configured (set in config or GOOGLE_API_KEY env)".to_string()))?;
        let cse_id = cse_id_owned.ok_or_else(|| McpError::validation("Google CSE ID not configured (set in config or GOOGLE_CSE_ID env)".to_string()))?;

        let q = if let Some(site_str) = site { format!("site:{} {}", site_str, query) } else { query };

        let num_s = num.to_string();
        let q_s = q;

        let resp = self.client
            .get("https://www.googleapis.com/customsearch/v1")
            .query(&[
                ("key", api_key.as_str()),
                ("cx", cse_id.as_str()),
                ("q", q_s.as_str()),
                ("num", num_s.as_str()),
            ])
            .send()
            .await
            .map_err(|e| McpError::internal(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(McpError::internal(format!("Google API error: {status} - {body}")));
        }

        let data: GoogleSearchResponse = resp.json().await.map_err(|e| McpError::internal(e.to_string()))?;
        let items = data.items.unwrap_or_default();

        match format {
            "json" => {
                let out = serde_json::json!({
                    "results": items.iter().map(|it| serde_json::json!({
                        "title": it.title,
                        "link": it.link,
                        "snippet": it.snippet,
                    })).collect::<Vec<_>>()
                });
                Ok(text_tool_result(out.to_string()))
            }
            _ => {
                if items.is_empty() {
                    return Ok(text_tool_result("No results."));
                }
                let text_lines: Vec<String> = items
                    .iter()
                    .enumerate()
                    .map(|(i, it)| {
                        let title = it.title.as_deref().unwrap_or("");
                        let link = it.link.as_deref().unwrap_or("");
                        let snippet = it.snippet.as_deref().unwrap_or("");
                        format!("{}. {}\n{}\n{}\n", i + 1, title, link, snippet)
                    })
                    .collect();
                Ok(text_tool_result(text_lines.join("\n")))
            }
        }
    }
}

impl GoogleSearchHandler {
    pub fn from_config(client: Client, cfg: &Config) -> Self {
        let env_key = std::env::var("GOOGLE_API_KEY").ok();
        let env_cx = std::env::var("GOOGLE_CSE_ID").ok();
        let cfg_nested_key = cfg.google_search.as_ref().and_then(|g| g.api_key.clone());
        let cfg_nested_cx = cfg.google_search.as_ref().and_then(|g| g.cse_id.clone());

        Self { client, api_key: env_key.or(cfg_nested_key), cse_id: env_cx.or(cfg_nested_cx) }
    }
}
