use async_trait::async_trait;
use mcp_protocol_sdk::prelude::*;
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;
use super::utils::{fetch_html, required_str_arg, text_tool_result};

pub struct FetchTextHandler {
    pub client: Client,
}

#[async_trait]
impl ToolHandler for FetchTextHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let url = required_str_arg(&arguments, "url")?;
        let html = fetch_html(&self.client, &url).await?;
        let doc = Html::parse_document(&html);
        let sel = Selector::parse("body").map_err(|e| McpError::internal(e.to_string()))?;
        let text = doc
            .select(&sel)
            .map(|n| n.text().collect::<Vec<_>>().join(" "))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(text_tool_result(text))
    }
}
