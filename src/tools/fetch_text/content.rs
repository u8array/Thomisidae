use scraper::{Html, Selector};

pub fn is_probably_html(html: &str) -> bool {
    let lower = html.get(0..256).unwrap_or("").to_ascii_lowercase();
    lower.contains("<html") || lower.contains("<body") || lower.contains("<!doctype")
}

pub fn is_html_content_type(ct: Option<&str>) -> bool {
    match ct {
        Some(v) => {
            let v = v.to_ascii_lowercase();
            v.starts_with("text/html") || v.contains("html")
        }
        None => false,
    }
}

pub fn is_json_content_type(ct: Option<&str>) -> bool {
    match ct {
        Some(v) => {
            let v = v.to_ascii_lowercase();
            v.starts_with("application/json") || v.contains("+json")
        }
        None => false,
    }
}

pub fn is_markdown_content_type(ct: Option<&str>) -> bool {
    match ct {
        Some(v) => v.to_ascii_lowercase().starts_with("text/markdown"),
        None => false,
    }
}

pub fn is_text_plain_content_type(ct: Option<&str>) -> bool {
    match ct {
        Some(v) => v.to_ascii_lowercase().starts_with("text/plain"),
        None => false,
    }
}

pub fn extract_title_or_h1(doc: &Html) -> Option<String> {
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

pub fn sanitize_html(html: &str) -> String {
    ammonia::Builder::default().clean(html).to_string()
}

pub fn normalize_space(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn pretty_json(s: &str) -> Result<String, serde_json::Error> {
    let v: serde_json::Value = serde_json::from_str(s)?;
    serde_json::to_string_pretty(&v)
}

#[cfg(test)]
mod tests {
    use super::*;

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
            <div onclick=\"alert('x')\">Hello <script>alert('bad')</script><a href=\"/x\">link</a></div>
        "#;
        let clean = sanitize_html(html);
        assert!(!clean.contains("<script>"));
        assert!(!clean.contains("onclick"));
        assert!(clean.contains("<a"));
        assert!(clean.contains("Hello"));
    }
}
