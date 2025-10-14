use super::meta::ToolMeta;
use super::utils::{fetch_html, required_str_arg, text_tool_result};
use async_trait::async_trait;
use mcp_protocol_sdk::prelude::*;
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::OnceLock;

static META: OnceLock<ToolMeta> = OnceLock::new();

pub fn meta() -> ToolMeta {
    META.get_or_init(|| {
        ToolMeta::new_with_default_schema(
            "fetch_url_text",
            "Fetch URL Text",
            "Fetches the text content of a URL",
        )
    })
    .clone()
}
pub struct FetchTextHandler {
    pub client: Client,
}

#[async_trait]
impl ToolHandler for FetchTextHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let url = required_str_arg(&arguments, "url")?;
        let html = fetch_html(&self.client, &url).await?;
        let doc = Html::parse_document(&html);

        let selectors = ["main", "article", "section", "body"];
        let blocks = selectors
            .iter()
            .find_map(|selector| {
                Selector::parse(selector).ok().map(|sel| {
                    doc.select(&sel)
                        .flat_map(|node| extract_blocks(&node))
                        .collect::<Vec<_>>()
                })
            })
            .unwrap_or_default();

        let text = blocks.join("\n");
        Ok(text_tool_result(text))
    }
}

const BLOCK_TAGS: &[&str] = &[
    "p",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "li",
    "blockquote",
    "pre",
];
fn is_block_tag(name: &str) -> bool {
    BLOCK_TAGS.contains(&name)
}

fn extract_blocks(node: &scraper::ElementRef) -> Vec<String> {
    let name = node.value().name();
    if matches!(name, "script" | "style") {
        return vec![];
    }
    if is_block_tag(name) {
        let text = node.text().collect::<Vec<_>>().join(" ").trim().to_string();
        if text.len() > 30 {
            return vec![text];
        }
    }
    node.children()
        .filter_map(scraper::ElementRef::wrap)
        .flat_map(|child| extract_blocks(&child))
        .collect()
}
