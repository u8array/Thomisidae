use mcp_protocol_sdk::prelude::ToolHandler;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;

use crate::{FetchLinksHandler, FetchTextHandler, ToolMeta, ToolsMeta};

use super::state::AppState;

pub fn build_state(client: &Client) -> AppState {
    let fetch_text_handler = Arc::new(FetchTextHandler {
        client: client.clone(),
    });
    let fetch_links_handler = Arc::new(FetchLinksHandler {
        client: client.clone(),
    });

    let fetch_url_text_meta = ToolMeta::new_with_default_schema(
        "fetch_url_text",
        "Fetch URL Text",
        "Fetches the text content of a URL",
    );

    let fetch_page_links_meta = ToolMeta::new_with_default_schema(
        "fetch_page_links",
        "Fetch Page Links",
        "Fetches links from a page",
    );

    let tools_meta = ToolsMeta(vec![fetch_url_text_meta, fetch_page_links_meta]);

    let mut handlers: HashMap<String, Arc<dyn ToolHandler + Send + Sync>> = HashMap::new();
    handlers.insert(
        "fetch_url_text".into(),
        fetch_text_handler as Arc<dyn ToolHandler + Send + Sync>,
    );
    handlers.insert(
        "fetch_page_links".into(),
        fetch_links_handler as Arc<dyn ToolHandler + Send + Sync>,
    );

    AppState {
        tools_meta,
        handlers,
    }
}
