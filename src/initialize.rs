use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, Write};
use std::sync::mpsc;
use std::time::Duration;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub tools: ToolsCapability,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsCapability {
    pub list_changed: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol_version: String,
    pub server_info: ServerInfo,
    pub capabilities: Capabilities,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonRpcResponse<T> {
    pub jsonrpc: String,
    pub id: Value,
    pub result: T,
}


pub fn maybe_respond_to_initialize() -> Result<()> {
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
                    let id = val
                        .get("id")
                        .cloned()
                        .unwrap_or(Value::Number(1.into()));
                    let result = InitializeResult {
                        protocol_version: "2025-06-18".to_string(),
                        server_info: ServerInfo {
                            name: "url-fetcher".to_string(),
                            version: "0.1.0".to_string(),
                        },
                        capabilities: Capabilities {
                            tools: ToolsCapability { list_changed: false },
                        },
                    };
                    let resp = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result,
                    };
                    let mut stdout = io::stdout();
                    writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
                    stdout.flush()?;
                }
            }
        }
    }

    Ok(())
}
