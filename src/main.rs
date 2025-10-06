use anyhow::Result;
use reqwest::Client;
use mcp_protocol_sdk::prelude::*;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};

mod tools;
mod tool_meta;
use tools::{FetchLinksHandler, FetchTextHandler};
mod initialize;
use initialize::maybe_respond_to_initialize;
mod response;
use response::{write_response_error, write_response_result};

use crate::tool_meta::{ToolMeta, ToolInputSchema, ToolsMeta};

#[tokio::main]
async fn main() -> Result<()> {
    maybe_respond_to_initialize()?;

    let client = Client::new();
    let fetch_text_handler = FetchTextHandler { client: client.clone() };
    let fetch_links_handler = FetchLinksHandler { client: client.clone() };

    let fetch_url_text_meta = ToolMeta::builder()
        .name("fetch_url_text")
        .title("Fetch URL Text")
        .description("Fetches the text content of a URL")
        .input_schema(
            ToolInputSchema::default()
        )
        .build();

    let fetch_page_links_meta = ToolMeta::builder()
        .name("fetch_page_links")
        .title("Fetch Page Links")
        .description("Fetches links from a page")
        .input_schema(
            ToolInputSchema::default()
        )
        .build();

    let tools_meta = ToolsMeta(vec![fetch_url_text_meta, fetch_page_links_meta]);

    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();

    let mut handlers: HashMap<String, Arc<dyn ToolHandler + Send + Sync>> = HashMap::new();
    handlers.insert("fetch_url_text".to_string(), Arc::new(fetch_text_handler));
    handlers.insert("fetch_page_links".to_string(), Arc::new(fetch_links_handler));

    while let Some(line) = lines.next_line().await? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let msg: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let method = msg.get("method").and_then(Value::as_str);
        let id = msg.get("id").cloned();

        match method {
            Some("initialize") => {
                let result = json!({
                    "protocolVersion": "2025-06-18",
                    "serverInfo": { "name": "url-fetcher", "version": "0.1.0" },
                    "capabilities": { "tools": { "listChanged": false } }
                });
                if let Some(idv) = id {
                    write_response_result(idv, result)?;
                }
            }
            Some("tools/list") => {
                if let Some(idv) = id {
                    let res = json!({ "tools": &tools_meta.0 });
                    write_response_result(idv, res)?;
                }
            }
            Some("tools/call") => {
                let params = msg.get("params").cloned().unwrap_or(json!({}));
                let name = params.get("name").and_then(Value::as_str);
                let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
                if let (Some(tool_name), Some(idv)) = (name, id) {
                    if let Some(handler) = handlers.get(tool_name) {
                        let arg_map: HashMap<String, Value> = serde_json::from_value(arguments).unwrap_or_default();

                        let result_val = match handler.call(arg_map).await {
                            Ok(tr) => serde_json::to_value(tr).unwrap_or(json!(null)),
                            Err(e) => json!({ "error": e.to_string() }),
                        };

                        write_response_result(idv, result_val)?;
                    } else {
                        write_response_error(idv, -32601, "Tool not found")?;
                    }
                }
            }
            _ => {
                if let Some(idv) = id {
                    write_response_error(idv, -32601, "Method not found")?;
                }
            }
        }
    }

    Ok(())
}
