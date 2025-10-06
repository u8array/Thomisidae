use mcp_protocol_sdk::prelude::ToolHandler;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

use crate::ToolsMeta;

#[derive(Clone)]
pub struct AppState {
    pub tools_meta: ToolsMeta,
    pub handlers: HashMap<String, Arc<dyn ToolHandler + Send + Sync>>,
}

#[derive(Deserialize, Default)]
pub struct CallParams {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub arguments: serde_json::Value,
}
