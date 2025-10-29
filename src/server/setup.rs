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
    ToolMeta,
    Robots,
};

use super::state::AppState;
use tokio::sync::Semaphore;

use crate::config::Config;
use crate::tools::DomainPolicy;

pub fn build_state(client: &Client, config: &Config) -> AppState {
    let ua = config
        .robots
        .user_agent
        .clone()
        .unwrap_or_else(|| "thomisidae/0.1.0".to_string());

    let robots = Arc::new(Robots::new(
        client.clone(),
        ua,
        config.robots.obey,
        config.robots.cache_ttl_secs,
    ));

    let policy = Arc::new(DomainPolicy::from_config(config));

    let fetch_text_handler = Arc::new(FetchTextHandler {
        client: client.clone(),
        robots: robots.clone(),
        max_response_size: config.max_response_size,
        policy: policy.clone(),
    });
    let fetch_links_handler = Arc::new(FetchLinksHandler {
        client: client.clone(),
        robots: robots.clone(),
        max_response_size: config.max_response_size,
        policy: policy.clone(),
    });
    let google_search_handler = Arc::new(GoogleSearchHandler::from_config(client.clone(), config));

    let mut metas = Vec::new();
    let mut handlers: HashMap<String, Arc<dyn ToolHandler + Send + Sync>> = HashMap::new();

    fn maybe_annotate_policy(mut m: ToolMeta, policy: &crate::tools::DomainPolicy, header: &str) -> ToolMeta {
        if !policy.is_empty() {
            m.description = format!(
                "{}\n\n{}\n{}",
                m.description,
                header,
                policy.describe()
            );
        }
        m
    }

    if config.is_enabled("fetch_url_text") {
        let m = maybe_annotate_policy(fetch_text_meta(), &policy, "Domain policy:");
        metas.push(m);
        handlers.insert(
            "fetch_url_text".into(),
            fetch_text_handler as Arc<dyn ToolHandler + Send + Sync>,
        );
    }

    if config.is_enabled("fetch_page_links") {
        let m = maybe_annotate_policy(fetch_links_meta(), &policy, "Domain policy:");
        metas.push(m);
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
