use async_trait::async_trait;
use mcp_protocol_sdk::prelude::*;
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;
use super::utils::{fetch_html, required_str_arg, text_tool_result};

pub struct FetchLinksHandler {
    pub client: Client,
}

#[async_trait]
impl ToolHandler for FetchLinksHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let url = required_str_arg(&arguments, "url")?;
        let html = fetch_html(&self.client, &url).await?;
        let doc = Html::parse_document(&html);
        let a = Selector::parse("a[href]").map_err(|e| McpError::internal(e.to_string()))?;
        let links: Vec<String> = doc
            .select(&a)
            .filter_map(|el| el.value().attr("href").map(|s| s.to_string()))
            .collect();

        let json_text = serde_json::to_string(&links)
            .map_err(|e| McpError::internal(e.to_string()))?;

        Ok(text_tool_result(json_text))
    }
}
