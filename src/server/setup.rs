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

pub fn build_state(client: &Client) -> AppState {
    let fetch_text_handler = Arc::new(FetchTextHandler {
        client: client.clone(),
    });
    let fetch_links_handler = Arc::new(FetchLinksHandler {
        client: client.clone(),
    });


    let tools_meta = ToolsMeta(vec![fetch_text_meta(), fetch_links_meta()]);

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
