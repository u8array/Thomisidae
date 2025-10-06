use anyhow::Result;
use reqwest::Client;
use mcp_protocol_sdk::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::io::{self, Write, BufRead};
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, BufReader};
use std::sync::mpsc;
use std::time::Duration;

mod tools;
use tools::{FetchLinksHandler, FetchTextHandler};

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let stdin = io::stdin();
        let mut lock = stdin.lock();
        let mut line = String::new();
        let _ = lock.read_line(&mut line);
        let _ = tx.send(line);
    });

    if let Ok(buffer) = rx.recv_timeout(Duration::from_millis(500)) {
        let trimmed = buffer.trim();
        if !trimmed.is_empty() {
            if let Ok(val) = serde_json::from_str::<Value>(trimmed) {
                if val.get("method").and_then(|m| m.as_str()) == Some("initialize") {
                    let id = val.get("id").cloned().unwrap_or(Value::Number(1.into()));
                    let result = json!({
                        "protocolVersion": "2025-06-18",
                        "serverInfo": {
                            "name": "url-fetcher",
                            "version": "0.1.0"
                        },
                        "capabilities": {
                            "tools": { "listChanged": false }
                        }
                    });
                    let resp = json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": result
                    });
                    let mut stdout = io::stdout();
                    writeln!(stdout, "{}", resp.to_string())?;
                    stdout.flush()?;
                }
            }
        }
    }

    let client = Client::new();
    let fetch_text_handler = FetchTextHandler { client: client.clone() };
    let fetch_links_handler = FetchLinksHandler { client: client.clone() };

    let mut tools_meta: Vec<Value> = Vec::new();
    tools_meta.push(json!({
        "name": "fetch_url_text",
        "title": "Fetch URL Text",
        "description": "Fetches the text content of a URL",
        "inputSchema": {
            "type": "object",
            "properties": { "url": { "type": "string" } },
            "required": ["url"]
        }
    }));
    tools_meta.push(json!({
        "name": "fetch_page_links",
        "title": "Fetch Page Links",
        "description": "Fetches links from a page",
        "inputSchema": {
            "type": "object",
            "properties": { "url": { "type": "string" } },
            "required": ["url"]
        }
    }));

    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();

    enum HandlerKind {
        FetchText(FetchTextHandler),
        FetchLinks(FetchLinksHandler),
    }
    let mut handlers: HashMap<String, HandlerKind> = HashMap::new();
    handlers.insert("fetch_url_text".to_string(), HandlerKind::FetchText(fetch_text_handler));
    handlers.insert("fetch_page_links".to_string(), HandlerKind::FetchLinks(fetch_links_handler));

    while let Some(line) = lines.next_line().await? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let msg: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let method = msg.get("method").and_then(|m| m.as_str()).map(|s| s.to_string());
        let id = msg.get("id").cloned();

        match method.as_deref() {
            Some("initialize") => {
                let result = json!({
                    "protocolVersion": "2025-06-18",
                    "serverInfo": { "name": "url-fetcher", "version": "0.1.0" },
                    "capabilities": { "tools": { "listChanged": false } }
                });
                if let Some(idv) = id {
                    let resp = json!({ "jsonrpc": "2.0", "id": idv, "result": result });
                    let mut stdout = io::stdout();
                    writeln!(stdout, "{}", resp.to_string())?;
                    stdout.flush()?;
                }
            }
            Some("tools/list") => {
                if let Some(idv) = id {
                    let res = json!({ "tools": tools_meta });
                    let resp = json!({ "jsonrpc": "2.0", "id": idv, "result": res });
                    let mut stdout = io::stdout();
                    writeln!(stdout, "{}", resp.to_string())?;
                    stdout.flush()?;
                }
            }
            Some("tools/call") => {
                let params = msg.get("params").cloned().unwrap_or(json!({}));
                let name = params.get("name").and_then(|n| n.as_str()).map(|s| s.to_string());
                let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
                if let (Some(tool_name), Some(idv)) = (name, id) {
                    if let Some(handler) = handlers.remove(&tool_name) {
                        let arg_map: HashMap<String, Value> = serde_json::from_value(arguments).unwrap_or_default();

                        let result_val = match handler {
                            HandlerKind::FetchText(h) => {
                                match h.call(arg_map).await {
                                    Ok(tr) => serde_json::to_value(tr).unwrap_or(json!(null)),
                                    Err(e) => json!({ "error": e.to_string() }),
                                }
                            }
                            HandlerKind::FetchLinks(h) => {
                                match h.call(arg_map).await {
                                    Ok(tr) => serde_json::to_value(tr).unwrap_or(json!(null)),
                                    Err(e) => json!({ "error": e.to_string() }),
                                }
                            }
                        };

                        let resp = json!({ "jsonrpc": "2.0", "id": idv, "result": result_val });
                        let mut stdout = io::stdout();
                        writeln!(stdout, "{}", resp.to_string())?;
                        stdout.flush()?;
                    } else {
                        let err = json!({ "code": -32601, "message": "Tool not found" });
                        let resp = json!({ "jsonrpc": "2.0", "id": idv, "error": err });
                        let mut stdout = io::stdout();
                        writeln!(stdout, "{}", resp.to_string())?;
                        stdout.flush()?;
                    }
                }
            }
            _ => {
                if let Some(idv) = id {
                    let err = json!({ "code": -32601, "message": "Method not found" });
                    let resp = json!({ "jsonrpc": "2.0", "id": idv, "error": err });
                    let mut stdout = io::stdout();
                    writeln!(stdout, "{}", resp.to_string())?;
                    stdout.flush()?;
                }
            }
        }
    }

    Ok(())
}
