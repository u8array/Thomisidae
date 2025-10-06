
use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use jsonrpc_v2::{Data, Error as RpcError, Server, Params, ResponseObjects};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, BufReader};

mod tools;
mod tool_meta;
use tools::{FetchLinksHandler, FetchTextHandler};
use mcp_protocol_sdk::prelude::ToolHandler;
mod initialize;
use initialize::maybe_respond_to_initialize;

use crate::tool_meta::{ToolMeta, ToolInputSchema, ToolsMeta};


#[derive(Clone)]
struct AppState {
    tools_meta: ToolsMeta,
    handlers: HashMap<String, Arc<dyn ToolHandler + Send + Sync>>,
}

#[derive(Deserialize, Default)]
struct CallParams {
    #[serde(default)]
    name: String,
    #[serde(default)]
    arguments: serde_json::Value,
}

async fn initialize(_: Params<serde_json::Value>, _data: Data<AppState>) -> Result<serde_json::Value, RpcError> {
    Ok(json!({
        "protocolVersion": "2025-06-18",
        "serverInfo": { "name": "url-fetcher", "version": "0.1.0" },
        "capabilities": { "tools": { "listChanged": false } }
    }))
}

async fn tools_list(_: Params<serde_json::Value>, data: Data<AppState>) -> Result<serde_json::Value, RpcError> {
    Ok(json!({ "tools": &data.tools_meta.0 }))
}

async fn tools_call(params: Params<serde_json::Value>, data: Data<AppState>) -> Result<serde_json::Value, RpcError> {
    let raw = params.0;
    let parsed: CallParams = match serde_json::from_value(raw.clone()) {
        Ok(cp) => cp,
        Err(_) => CallParams::default(),
    };
    let name = parsed.name;
    let arguments = if parsed.arguments.is_null() { serde_json::json!({}) } else { parsed.arguments };
    let arg_map: HashMap<String, serde_json::Value> = serde_json::from_value(arguments).unwrap_or_default();
    if name.is_empty() {
        return Err(RpcError::internal("Missing 'name' in params"));
    }

    if let Some(handler) = data.handlers.get(&name) {
        match handler.call(arg_map).await {
            Ok(tr) => Ok(serde_json::to_value(tr).unwrap_or(json!(null))),
            Err(e) => Err(RpcError::internal(e.to_string())),
        }
    } else {
        Err(RpcError::internal("Tool not found"))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    maybe_respond_to_initialize()?;

    let client = Client::new();
    let fetch_text_handler = Arc::new(FetchTextHandler { client: client.clone() });
    let fetch_links_handler = Arc::new(FetchLinksHandler { client: client.clone() });

    let fetch_url_text_meta = ToolMeta::builder()
        .name("fetch_url_text")
        .title("Fetch URL Text")
        .description("Fetches the text content of a URL")
        .input_schema(ToolInputSchema::default())
        .build();

    let fetch_page_links_meta = ToolMeta::builder()
        .name("fetch_page_links")
        .title("Fetch Page Links")
        .description("Fetches links from a page")
        .input_schema(ToolInputSchema::default())
        .build();

    let tools_meta = ToolsMeta(vec![fetch_url_text_meta, fetch_page_links_meta]);

    let mut handlers: HashMap<String, Arc<dyn ToolHandler + Send + Sync>> = HashMap::new();
    handlers.insert("fetch_url_text".to_string(), fetch_text_handler.clone() as Arc<dyn ToolHandler + Send + Sync>);
    handlers.insert("fetch_page_links".to_string(), fetch_links_handler.clone() as Arc<dyn ToolHandler + Send + Sync>);

    let state = AppState {
        tools_meta,
        handlers,
    };

    let server = Server::new()
        .with_data(Data::new(state))
        .with_method("initialize", initialize)
        .with_method("tools/list", tools_list)
        .with_method("tools/call", tools_call)
        .finish();

    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();

    while let Some(line) = lines.next_line().await? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let response = server.handle(trimmed.as_bytes()).await;
        match response {
            ResponseObjects::One(obj) => {
                if let Ok(s) = serde_json::to_string(&obj) {
                    println!("{}", s);
                }
            }
            ResponseObjects::Many(list) => {
                if let Ok(s) = serde_json::to_string(&list) {
                    println!("{}", s);
                }
            }
            _ => {}
        }
    }

    Ok(())
}
