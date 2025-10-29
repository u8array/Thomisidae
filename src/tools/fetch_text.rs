use super::meta::{ToolInputSchema, ToolMeta};
use super::utils::{fetch_html_with_headers, required_str_arg, text_tool_result, FetchedResponse};
use super::robots::Robots;
use super::policy::{DomainPolicy, ensure_allowed};
use async_trait::async_trait;
use mcp_protocol_sdk::prelude::*;
use reqwest::Client;
use scraper::{Html, Selector};
use scraper::node::Node as ScraperNode;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use url::Url;

static META: OnceLock<ToolMeta> = OnceLock::new();

pub fn meta() -> ToolMeta {
    META
        .get_or_init(|| {
            let schema = ToolInputSchema::new(
                "object",
                serde_json::json!({
                    "url": { "type": "string" },
                    "max_length": { "type": "integer", "minimum": 1 },
                    "start_index": { "type": "integer", "minimum": 0 },
                    "raw": { "type": "boolean" },
                    "format": { "type": "string", "enum": ["plain", "markdown"], "default": "plain" }
                }),
                vec!["url".to_string()],
            );
            ToolMeta::new(
                "fetch_url_text",
                "Fetch URL Text",
                "Fetches the text content of a URL (optionally chunked and as raw HTML)",
                schema,
            )
        })
    .clone()
}
pub struct FetchTextHandler {
    pub client: Client,
    pub robots: Arc<Robots>,
    pub max_response_size: usize,
    pub policy: Arc<DomainPolicy>,
}

#[async_trait]
impl ToolHandler for FetchTextHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let url = required_str_arg(&arguments, "url")?;
        let max_length: usize = arguments
            .get("max_length")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(5000);
        let start_index: usize = arguments
            .get("start_index")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(0);
        let raw: bool = arguments
            .get("raw")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let format = arguments
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("plain");

        let parsed = Url::parse(&url).map_err(|e| McpError::validation(format!("Invalid url: {e}")))?;
        ensure_allowed(&self.policy, &parsed)?;
        if !self.robots.allow(&parsed).await? {
            return Err(McpError::validation("Blocked by robots.txt".to_string()));
        }
    let FetchedResponse { body: html, content_type } = fetch_html_with_headers(&self.client, &url, self.max_response_size).await?;

    let looks_like_html = is_html_content_type(content_type.as_deref()) || is_probably_html(&html);

        let mut prefix = String::new();
        let content = if raw {
            html
        } else if !looks_like_html {
            let ct_note = content_type.as_deref().unwrap_or("");
            prefix = if !ct_note.is_empty() {
                format!("Content type {ct_note} cannot be simplified; returning raw content.\n\n")
            } else {
                "Content cannot be simplified; returning raw content.\n\n".to_string()
            };
            html
        } else if format.eq_ignore_ascii_case("markdown") {
            let doc = Html::parse_document(&html);
            if let Some(title) = extract_title_or_h1(&doc) {
                prefix = format!("Title: {}\nURL: {}\n\n", title, url);
            } else {
                prefix = format!("URL: {}\n\n", url);
            }
            let clean = sanitize_html(&html);
            html2md::parse_html(&clean)
        } else {
            let plain_html = if format.eq_ignore_ascii_case("plain") { sanitize_html(&html) } else { html.clone() };
            let doc = Html::parse_document(&plain_html);
            if let Some(title) = extract_title_or_h1(&doc) {
                prefix = format!("Title: {}\nURL: {}\n\n", title, url);
            } else {
                prefix = format!("URL: {}\n\n", url);
            }
            let blocks = extract_best_blocks(&doc).unwrap_or_else(|| extract_fallback_blocks(&doc));
            blocks.join("\n")
        };

    let text = truncate_with_hint(&(prefix + &content), start_index, max_length);

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

fn extract_best_blocks(doc: &Html) -> Option<Vec<String>> {
    let mut best: Option<(usize, Vec<String>)> = None;
    for sel_str in ["main", "article", "section", "body"].iter() {
        let sel = match Selector::parse(sel_str) { Ok(s) => s, Err(_) => continue };
        let mut blocks = Vec::new();
        for node in doc.select(&sel) {
            extract_blocks(&node, &mut blocks);
        }
        if !blocks.is_empty() {
            let total_len: usize = blocks.iter().map(|b| b.len()).sum();
            match &mut best {
                Some((best_len, _)) if total_len <= *best_len => {}
                _ => best = Some((total_len, blocks)),
            }
        }
    }
    best.map(|(_, v)| v)
}

fn extract_fallback_blocks(doc: &Html) -> Vec<String> {
    Selector::parse("p, h1, h2, h3, h4, h5, h6, li, blockquote, pre, div")
        .ok()
        .map(|sel| {
            doc.select(&sel)
                .filter_map(|node| {
                    let text = collect_visible_text(&node);
                    let normalized = normalize_space(&text);
                    if normalized.len() > 30 && !is_noise(&normalized) {
                        Some(format_block(node.value().name(), normalized))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

fn extract_blocks(node: &scraper::ElementRef, out: &mut Vec<String>) {
    let name = node.value().name();
    if matches!(name, "script" | "style" | "noscript" | "nav" | "header" | "footer" | "aside" | "form" | "svg" | "iframe") {
        return;
    }
    if is_block_tag(name) {
        let raw_text = node
            .text()
            .filter(|t| !t.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        let normalized = normalize_space(&raw_text);
        if normalized.len() > 30 {
            out.push(format_block(name, normalized));
            return;
        }
    }
    node.children()
        .filter_map(scraper::ElementRef::wrap)
        .for_each(|child| extract_blocks(&child, out));
}

fn is_probably_html(html: &str) -> bool {
    let lower = html.get(0..256).unwrap_or("").to_ascii_lowercase();
    lower.contains("<html") || lower.contains("<body") || lower.contains("<!doctype")
}

fn is_html_content_type(ct: Option<&str>) -> bool {
    match ct {
        Some(v) => {
            let v = v.to_ascii_lowercase();
            v.starts_with("text/html") || v.contains("html")
        }
        None => false,
    }
}

fn extract_title_or_h1(doc: &Html) -> Option<String> {
    if let Ok(sel_title) = Selector::parse("title")
        && let Some(node) = doc.select(&sel_title).next() {
        let t = normalize_space(&node.text().collect::<Vec<_>>().join(" "));
        if !t.is_empty() {
            return Some(t);
        }
    }

    if let Ok(sel_h1) = Selector::parse("h1")
        && let Some(node) = doc.select(&sel_h1).next() {
        let t = normalize_space(&node.text().collect::<Vec<_>>().join(" "));
        if !t.is_empty() {
            return Some(t);
        }
    }
    None
}

fn sanitize_html(html: &str) -> String {
    ammonia::Builder::default().clean(html).to_string()
}

fn collect_visible_text(node: &scraper::ElementRef) -> String {
    let mut out = String::new();
    collect_visible_text_inner(node, &mut out);
    out
}

fn collect_visible_text_inner(node: &scraper::ElementRef, out: &mut String) {
    for child in node.children() {
        match child.value() {
            ScraperNode::Text(t) => {
                let t = t.text.trim();
                if !t.is_empty() {
                    if !out.is_empty() { out.push(' '); }
                    out.push_str(t);
                }
            }
            ScraperNode::Element(_) => {
                if let Some(el) = scraper::ElementRef::wrap(child) {
                    let name = el.value().name();
                    if matches!(name, "script" | "style" | "noscript" | "nav" | "header" | "footer" | "aside" | "form" | "svg" | "iframe") {
                        continue;
                    }
                    collect_visible_text_inner(&el, out);
                }
            }
            _ => {}
        }
    }
}

const NOISE_SUBSTRINGS: &[&str] = &[
    "{",
    "}",
    "lineargradient",
    "wprm-rating-star",
        "document.getelementbyid",
        "setattribute(\"value\"",
    ];

fn is_noise(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    NOISE_SUBSTRINGS.iter().any(|pat| lower.contains(pat))
}

fn truncate_with_hint(content: &str, start_index: usize, max_length: usize) -> String {
    let original_len = content.len();
    if start_index >= original_len {
        return "<error>No more content available.</error>".to_string();
    }
    let end = (start_index + max_length).min(original_len);
    let mut slice = content[start_index..end].to_string();
    if end < original_len {
        slice.push_str(&format!(
            "\n\n<error>Content truncated. Call this tool again with start_index={} to get more.</error>",
            end
        ));
    }
    slice
}

fn format_block(name: &str, normalized: String) -> String {
    match name.to_ascii_lowercase().as_str() {
        "h1" => format!("# {}", normalized),
        "h2" => format!("## {}", normalized),
        "h3" => format!("### {}", normalized),
        "h4" => format!("#### {}", normalized),
        "h5" => format!("##### {}", normalized),
        "h6" => format!("###### {}", normalized),
        "li" => format!("- {}", normalized),
        "blockquote" => format!("> {}", normalized),
        "pre" => format!("```\n{}\n```", normalized),
        _ => normalized,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_space_collapses_whitespace() {
        assert_eq!(normalize_space("a\t b\n c"), "a b c");
    }

    #[test]
    fn truncate_with_hint_handles_bounds() {
        let s = "abcdef";
        assert!(truncate_with_hint(s, 10, 3).contains("No more content"));
        assert_eq!(truncate_with_hint(s, 0, 3)[..3], *"abc");
    }

    #[test]
    fn is_html_content_type_detects_html() {
        assert!(is_html_content_type(Some("text/html; charset=utf-8")));
        assert!(is_html_content_type(Some("application/xhtml+xml")));
        assert!(!is_html_content_type(Some("application/json")));
        assert!(!is_html_content_type(None));
    }

    #[test]
    fn sanitize_html_removes_unsafe() {
        let html = r#"
            <div onclick="alert('x')">Hello <script>alert('bad')</script><a href="/x">link</a></div>
        "#;
        let clean = sanitize_html(html);
        assert!(!clean.contains("<script>"));
        assert!(!clean.contains("onclick"));
        assert!(clean.contains("<a"));
        assert!(clean.contains("Hello"));
    }

    #[test]
    fn is_noise_detects_js_boilerplate() {
        let s = "This is no noise here, just normal text.";
        assert!(!is_noise(s));
        let s2 = "document.getElementById(\"ak_js_1\").setAttribute(\"value\", (new Date()).getTime());";
        assert!(is_noise(s2));
    }

    #[test]
    fn is_noise_does_not_flag_content() {
        let s = "Tuna Cabbage Salad is a simple and tasty dish with sesame dressing.";
        assert!(!is_noise(s));
    }
}
