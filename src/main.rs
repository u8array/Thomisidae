use anyhow::Result;
use reqwest::Client;
use mcp_protocol_sdk::prelude::*;
use mcp_protocol_sdk::transport::stdio::StdioServerTransport;
use serde_json::json;

mod tools;
use tools::{FetchLinksHandler, FetchTextHandler};

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new();
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
