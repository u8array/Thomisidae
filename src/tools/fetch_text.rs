use async_trait::async_trait;
use mcp_protocol_sdk::prelude::*;
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;

pub struct FetchTextHandler {
    pub client: Client,
}

#[async_trait]
impl ToolHandler for FetchTextHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let url = arguments
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| mcp_protocol_sdk::McpError::validation("Missing 'url' parameter".to_string()))?;

        let html = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| mcp_protocol_sdk::McpError::internal(e.to_string()))?
            .text()
            .await
            .map_err(|e| mcp_protocol_sdk::McpError::internal(e.to_string()))?;
        let doc = Html::parse_document(&html);
        let sel = Selector::parse("body").map_err(|e| McpError::internal(e.to_string()))?;
        let text = doc
            .select(&sel)
            .map(|n| n.text().collect::<Vec<_>>().join(" "))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(ToolResult {
            content: vec![Content::Text { text, annotations: None }],
            is_error: Some(false),
            meta: None,
        })
    }
}
