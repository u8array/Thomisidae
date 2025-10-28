use mcp_protocol_sdk::prelude::ToolHandler;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::ToolsMeta;

#[derive(Clone)]
pub struct AppState {
    pub tools_meta: ToolsMeta,
    pub handlers: HashMap<String, Arc<dyn ToolHandler + Send + Sync>>,
    pub concurrency: Arc<Semaphore>,
}

#[derive(Deserialize, Default)]
pub struct CallParams {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub arguments: serde_json::Value,
}
