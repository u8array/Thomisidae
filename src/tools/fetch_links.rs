use async_trait::async_trait;
use mcp_protocol_sdk::prelude::*;
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::Value;
use std::{collections::{HashMap, HashSet}, sync::Arc};
use super::utils::{fetch_html, required_str_arg, text_tool_result};
use url::Url;
use super::meta::ToolMeta;
use std::sync::OnceLock;
use super::robots::Robots;
use super::policy::{DomainPolicy, ensure_allowed};

static META: OnceLock<ToolMeta> = OnceLock::new();

pub fn meta() -> ToolMeta {
    META.get_or_init(|| {
        ToolMeta::new_with_default_schema(
            "fetch_page_links",
            "Fetch Page Links",
            "Fetches links from a page",
        )
    })
    .clone()
}

pub struct FetchLinksHandler {
    pub client: Client,
    pub robots: Arc<Robots>,
    pub max_response_size: usize,
    pub policy: Arc<DomainPolicy>,
}

#[async_trait]
impl ToolHandler for FetchLinksHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {

        let url = required_str_arg(&arguments, "url")?;
        let base_url = Url::parse(&url).map_err(|e| McpError::validation(format!("Invalid url: {e}")))?;
        ensure_allowed(&self.policy, &base_url)?;

        let same_domain = arguments.get("same_domain").and_then(|v| v.as_bool()).unwrap_or(false);
        let format = arguments
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("text");

        if !self.robots.allow(&base_url).await? {
            return Err(McpError::validation("Blocked by robots.txt".to_string()));
        }
    let html = fetch_html(&self.client, &url, self.max_response_size).await?;
        let doc = Html::parse_document(&html);
        let a = Selector::parse("a[href]").map_err(|e| McpError::internal(e.to_string()))?;
        let base_domain = base_url.domain();
        let mut seen: HashSet<String> = HashSet::new();
        let links: Vec<String> = doc
            .select(&a)
            .filter_map(|el| el.value().attr("href"))
            .filter_map(|href| base_url.join(href).or_else(|_| Url::parse(href)).ok())
            .filter(|u| matches!(u.scheme(), "http" | "https"))
            .filter(|u| !same_domain || u.domain() == base_domain)
            .map(|mut u| {
                u.set_fragment(None);
                u.to_string()
            })
            .filter_map(|s| if seen.insert(s.clone()) { Some(s) } else { None })
            .filter(|s| Url::parse(s).map_or(true, |u| self.policy.allows_url(&u)))
            .collect();

        match format {
            "json" => {
                let json_text = serde_json::to_string(&links)
                    .map_err(|e| McpError::internal(e.to_string()))?;
                Ok(text_tool_result(json_text))
            }
            _ => {
                let text = links.join("\n");
                Ok(text_tool_result(text))
            }
        }
    }
}
