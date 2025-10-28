use mcp_protocol_sdk::prelude::ToolHandler;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    FetchLinksHandler,
    FetchTextHandler,
    GoogleSearchHandler,
    ToolsMeta,
    fetch_links_meta,
    fetch_text_meta,
    google_search_meta,
};

use super::state::AppState;
use tokio::sync::Semaphore;

use crate::config::Config;

pub fn build_state(client: &Client, config: &Config) -> AppState {
    let fetch_text_handler = Arc::new(FetchTextHandler {
        client: client.clone(),
    });
    let fetch_links_handler = Arc::new(FetchLinksHandler {
        client: client.clone(),
    });
    let google_search_handler = Arc::new(GoogleSearchHandler::from_config(client.clone(), config));

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

    if config.is_enabled("google_search") {
        metas.push(google_search_meta());
        handlers.insert(
            "google_search".into(),
            google_search_handler as Arc<dyn ToolHandler + Send + Sync>,
        );
    }

    let tools_meta = ToolsMeta(metas);

    let concurrency = Arc::new(Semaphore::new(4));

    AppState {
        tools_meta,
        handlers,
        concurrency,
    }
}
