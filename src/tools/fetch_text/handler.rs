use super::super::utils::{fetch_html_with_headers, required_str_arg, text_tool_result, FetchedResponse};
use super::super::robots::Robots;
use super::super::policy::{DomainPolicy, ensure_allowed};
use super::content::{
    is_html_content_type, is_json_content_type, is_markdown_content_type, is_text_plain_content_type,
    is_probably_html, extract_title_or_h1, sanitize_html, pretty_json,
};
use super::extractors::{extract_best_blocks, extract_fallback_blocks};
#[cfg(feature = "readability")]
use super::extractors::extract_readability;
use super::chunk::truncate_with_hint;
use async_trait::async_trait;
use mcp_protocol_sdk::prelude::*;
use reqwest::Client;
use scraper::Html;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use url::Url;

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
        let mode = arguments
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("auto");
        let respect_robots_override = arguments
            .get("respect_robots")
            .and_then(|v| v.as_bool());

        let parsed = Url::parse(&url).map_err(|e| McpError::validation(format!("Invalid url: {e}")))?;
        ensure_allowed(&self.policy, &parsed)?;
        let obey = respect_robots_override.unwrap_or(true);
        if obey && !self.robots.allow(&parsed).await? {
            return Err(McpError::validation("Blocked by robots.txt".to_string()));
        }

        let FetchedResponse { body, content_type } = fetch_html_with_headers(&self.client, &url, self.max_response_size).await?;

        let ct_opt = content_type.as_deref();
        let is_html = is_html_content_type(ct_opt) || is_probably_html(&body);
        let is_json = is_json_content_type(ct_opt);
        let is_markdown = is_markdown_content_type(ct_opt);
        let is_text = is_text_plain_content_type(ct_opt);

        if raw || mode.eq_ignore_ascii_case("raw") {
            let text = truncate_with_hint(&body, start_index, max_length);
            return Ok(text_tool_result(text));
        }

        if is_json {
            let prefix = format!("URL: {}\n\n", url);
            let content = pretty_json(&body).unwrap_or(body);
            let text = prefix + &content;
            return Ok(text_tool_result(truncate_with_hint(&text, start_index, max_length)));
        } else if is_markdown || is_text {
            let prefix = format!("URL: {}\n\n", url);
            let text = prefix + &body;
            return Ok(text_tool_result(truncate_with_hint(&text, start_index, max_length)));
        } else if !is_html {
            let ct_note = ct_opt.unwrap_or("");
            let prefix = if !ct_note.is_empty() {
                format!("Content type {ct_note} cannot be simplified; returning raw content.\n\n")
            } else {
                "Content cannot be simplified; returning raw content.\n\n".to_string()
            };
            let text = prefix + &body;
            return Ok(text_tool_result(truncate_with_hint(&text, start_index, max_length)));
        } else if format.eq_ignore_ascii_case("markdown") {
            let doc = Html::parse_document(&body);
            let prefix = if let Some(title) = extract_title_or_h1(&doc) {
                format!("Title: {}\nURL: {}\n\n", title, url)
            } else {
                format!("URL: {}\n\n", url)
            };
            let clean = sanitize_html(&body);
            let extracted = match htmd::convert(&clean) {
                Ok(md) => md,
                Err(_) => clean,
            };
            let text = prefix + &extracted;
            return Ok(text_tool_result(truncate_with_hint(&text, start_index, max_length)));
        } else {
            let plain_html = if format.eq_ignore_ascii_case("plain") { sanitize_html(&body) } else { body.clone() };
            let doc = Html::parse_document(&plain_html);
            let prefix = if let Some(title) = extract_title_or_h1(&doc) {
                format!("Title: {}\nURL: {}\n\n", title, url)
            } else {
                format!("URL: {}\n\n", url)
            };
            let extracted = match mode.to_ascii_lowercase().as_str() {
                #[cfg(feature = "readability")]
                "readability" => extract_readability(&plain_html, &parsed).unwrap_or_else(|| extract_best_blocks(&doc).unwrap_or_else(|| extract_fallback_blocks(&doc)).join("\n")),
                _ => extract_best_blocks(&doc).unwrap_or_else(|| extract_fallback_blocks(&doc)).join("\n"),
            };
            let text = prefix + &extracted;
            return Ok(text_tool_result(truncate_with_hint(&text, start_index, max_length)));
        }
    }
}
