use mcp_protocol_sdk::prelude::*;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;


pub fn required_str_arg(
    arguments: &HashMap<String, Value>,
    key: &str,
) -> McpResult<String> {
    arguments
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| McpError::validation(format!("Missing '{key}' parameter")))
}


pub async fn fetch_html(client: &Client, url: &str) -> McpResult<String> {
    client
        .get(url)
        .send()
        .await
        .map_err(|e| McpError::internal(e.to_string()))?
        .text()
        .await
        .map_err(|e| McpError::internal(e.to_string()))
}


pub fn text_tool_result<T: Into<String>>(text: T) -> ToolResult {
    ToolResult {
        content: vec![Content::Text {
            text: text.into(),
            annotations: None,
        }],
        is_error: Some(false),
        meta: None,
    }
}
