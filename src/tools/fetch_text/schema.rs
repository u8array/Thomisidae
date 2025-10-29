use std::sync::OnceLock;
use super::super::meta::{ToolInputSchema, ToolMeta};

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
                    "format": { "type": "string", "enum": ["plain", "markdown"], "default": "plain" },
                    "mode": { "type": "string", "enum": ["auto", "best_blocks", "readability", "raw"], "default": "auto" },
                    "respect_robots": { "type": ["boolean", "null"], "description": "Override robots behavior for this call" }
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
