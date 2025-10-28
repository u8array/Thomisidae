use mcp_protocol_sdk::prelude::ToolHandler;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    FetchLinksHandler,
    FetchTextHandler,
    ToolsMeta,
    fetch_links_meta,
    fetch_text_meta,
};

use super::state::AppState;

use crate::config::Config;

pub fn build_state(client: &Client, config: &Config) -> AppState {
    let fetch_text_handler = Arc::new(FetchTextHandler {
        client: client.clone(),
    });
    let fetch_links_handler = Arc::new(FetchLinksHandler {
        client: client.clone(),
    });

    let mut metas = Vec::new();
    let mut handlers: HashMap<String, Arc<dyn ToolHandler + Send + Sync>> = HashMap::new();

    if config.is_enabled("fetch_url_text") {
        metas.push(fetch_text_meta());
        handlers.insert(
            "fetch_url_text".into(),
            fetch_text_handler as Arc<dyn ToolHandler + Send + Sync>,
        );
    }

    if config.is_enabled("fetch_page_links") {
        metas.push(fetch_links_meta());
        handlers.insert(
            "fetch_page_links".into(),
            fetch_links_handler as Arc<dyn ToolHandler + Send + Sync>,
        );
    }

    let tools_meta = ToolsMeta(metas);

    AppState {
        tools_meta,
        handlers,
    }
}
