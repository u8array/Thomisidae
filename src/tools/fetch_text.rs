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

        let blocks = extract_main_blocks(&doc).unwrap_or_else(|| extract_fallback_blocks(&doc));

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
    "div",
];

fn is_block_tag(name: &str) -> bool {
    BLOCK_TAGS.iter().any(|tag| name.eq_ignore_ascii_case(tag))
}

fn normalize_space(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn extract_main_blocks(doc: &Html) -> Option<Vec<String>> {
    ["main", "article", "section", "body"]
        .iter()
        .filter_map(|sel_str| Selector::parse(sel_str).ok())
        .filter_map(|sel| {
            let mut blocks = Vec::new();
            for node in doc.select(&sel) {
                extract_blocks(&node, &mut blocks);
            }
            (!blocks.is_empty()).then_some(blocks)
        })
        .next()
}

fn extract_fallback_blocks(doc: &Html) -> Vec<String> {
    Selector::parse("p, h1, h2, h3, h4, h5, h6, li, blockquote, pre, div")
        .ok()
        .map(|sel| {
            doc.select(&sel)
                .filter_map(|node| {
                    let text = node.text().collect::<Vec<_>>().join(" ");
                    let normalized = normalize_space(&text);
                    (normalized.len() > 30).then_some(normalized)
                })
                .collect()
        })
        .unwrap_or_default()
}

fn extract_blocks(node: &scraper::ElementRef, out: &mut Vec<String>) {
    let name = node.value().name();
    if matches!(name, "script" | "style" | "noscript") {
        return;
    }
    if is_block_tag(name) {
        let text = node
            .text()
            .filter(|t| !t.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        let normalized = normalize_space(&text);
        if normalized.len() > 30 {
            out.push(normalized);
            return;
        }
    }
    node.children()
        .filter_map(scraper::ElementRef::wrap)
        .for_each(|child| extract_blocks(&child, out));
}
