use scraper::{Html, Selector};
use scraper::node::Node as ScraperNode;
use super::content::normalize_space;

const BLOCK_TAGS: &[&str] = &[
    "p", "h1", "h2", "h3", "h4", "h5", "h6", "li", "blockquote", "pre", "div",
];

fn is_block_tag(name: &str) -> bool {
    BLOCK_TAGS.iter().any(|tag| name.eq_ignore_ascii_case(tag))
}

pub fn extract_best_blocks(doc: &Html) -> Option<Vec<String>> {
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

pub fn extract_fallback_blocks(doc: &Html) -> Vec<String> {
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

pub fn collect_visible_text(node: &scraper::ElementRef) -> String {
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

pub const NOISE_SUBSTRINGS: &[&str] = &[
    "{",
    "}",
    "lineargradient",
    "wprm-rating-star",
    "document.getelementbyid",
    "setattribute(\"value\"",
];

pub fn is_noise(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    NOISE_SUBSTRINGS.iter().any(|pat| lower.contains(pat))
}

pub fn format_block(name: &str, normalized: String) -> String {
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

#[cfg(feature = "readability")]
pub fn extract_readability(html: &str, base_url: &url::Url) -> Option<String> {
    let mut cursor = std::io::Cursor::new(html.as_bytes());
    match readability::extractor::extract(&mut cursor, base_url) {
        Ok(article) => Some(article.content),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_space_basic() {
        assert_eq!(super::super::content::normalize_space("a\t b\n c"), "a b c");
    }

    #[test]
    fn is_noise_detects_js_boilerplate() {
        let s = "This is no noise here, just normal text.";
        assert!(!is_noise(s));
        let s2 = "document.getElementById(\"ak_js_1\").setAttribute(\"value\", (new Date()).getTime());";
        assert!(is_noise(s2));
    }
}
