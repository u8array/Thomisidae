use anyhow::Result;
use jsonrpc_v2::{Data, Error as RpcError, Params};
use serde_json::json;
use std::collections::HashMap;

use super::state::{AppState, CallParams};

pub async fn initialize(
    _: Params<serde_json::Value>,
    _data: Data<AppState>,
) -> Result<serde_json::Value, RpcError> {
    Ok(json!({
        "protocolVersion": "2025-06-18",
        "serverInfo": { "name": "url-fetcher", "version": "0.1.0" },
        "capabilities": { "tools": { "listChanged": false } }
    }))
}

pub async fn tools_list(
    _: Params<serde_json::Value>,
    data: Data<AppState>,
) -> Result<serde_json::Value, RpcError> {
    Ok(json!({ "tools": &data.tools_meta.0 }))
}

pub async fn tools_call(
    params: Params<CallParams>,
    data: Data<AppState>,
) -> Result<serde_json::Value, RpcError> {
    let CallParams { name, arguments } = params.0;
    if name.is_empty() {
        return Err(RpcError::internal("Missing 'name' in params"));
    }

    let arguments = if arguments.is_null() {
        json!({})
    } else {
        arguments
    };
    let arg_map: HashMap<String, serde_json::Value> = serde_json::from_value(arguments)
        .map_err(|e| RpcError::internal(format!("Invalid 'arguments': {e}")))?;

    if let Some(handler) = data.handlers.get(&name) {
        match handler.call(arg_map).await {
            Ok(tr) => Ok(serde_json::to_value(tr).unwrap_or(json!(null))),
            Err(e) => Err(RpcError::internal(e.to_string())),
        }
    } else {
        Err(RpcError::internal("Tool not found"))
    }
}
