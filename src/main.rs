use anyhow::Result;
use reqwest::Client;
use scraper::{Html, Selector};
use mcp_protocol_sdk::prelude::*;
use mcp_protocol_sdk::transport::stdio::StdioServerTransport;
use serde_json::{json, Value};
use std::collections::HashMap;
use async_trait::async_trait;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new();

    struct FetchTextHandler {
        client: Client,
    }

    #[async_trait]
    impl ToolHandler for FetchTextHandler {
        async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
            let url = arguments
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| mcp_protocol_sdk::McpError::validation("Missing 'url' parameter".to_string()))?;

            let html = self.client.get(url).send().await.map_err(|e| mcp_protocol_sdk::McpError::internal(e.to_string()))?.text().await.map_err(|e| mcp_protocol_sdk::McpError::internal(e.to_string()))?;
            let doc = Html::parse_document(&html);
            let sel = Selector::parse("body").unwrap();
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

    struct FetchLinksHandler {
        client: Client,
    }

    #[async_trait]
    impl ToolHandler for FetchLinksHandler {
        async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
            let url = arguments
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| mcp_protocol_sdk::McpError::validation("Missing 'url' parameter".to_string()))?;

            let html = self.client.get(url).send().await.map_err(|e| mcp_protocol_sdk::McpError::internal(e.to_string()))?.text().await.map_err(|e| mcp_protocol_sdk::McpError::internal(e.to_string()))?;
            let doc = Html::parse_document(&html);
            let a = Selector::parse("a[href]").unwrap();
            let links: Vec<String> = doc.select(&a)
                .filter_map(|el| el.value().attr("href").map(|s| s.to_string()))
                .collect();

            let json_text = serde_json::to_string(&links).map_err(|e| McpError::internal(e.to_string()))?;

            Ok(ToolResult {
                content: vec![Content::Text { text: json_text, annotations: None }],
                is_error: Some(false),
                meta: None,
            })
        }
    }

    let mut server = McpServer::new("url-fetcher".to_string(), "0.1.0".to_string());

    server
        .add_tool(
            "fetch_url_text".to_string(),
            Some("Fetches the text content of a URL".to_string()),
            json!({
                "type": "object",
                "properties": { "url": { "type": "string" } },
                "required": ["url"]
            }),
            FetchTextHandler { client: client.clone() },
        )
        .await?;

    server
        .add_tool(
            "fetch_page_links".to_string(),
            Some("Fetches links from a page".to_string()),
            json!({
                "type": "object",
                "properties": { "url": { "type": "string" } },
                "required": ["url"]
            }),
            FetchLinksHandler { client: client.clone() },
        )
        .await?;


    let transport = StdioServerTransport::new();
    server.start(transport).await?;

    Ok(())
}
