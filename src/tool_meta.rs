use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolInputSchema {
    #[serde(rename = "type")]
    pub type_: String,
    pub properties: Value,
    pub required: Vec<String>,
}

impl Default for ToolInputSchema {
    fn default() -> Self {
        ToolInputSchema {
            type_: "object".to_string(),
            properties: serde_json::json!({ "url": { "type": "string" } }),
            required: vec!["url".to_string()],
        }
    }
}

#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ToolMeta {
    pub name: String,
    pub title: String,
    pub description: String,
    pub input_schema: ToolInputSchema,
}

#[derive(Serialize, Default, Clone)]
#[serde(transparent)]
pub struct ToolsMeta(pub Vec<ToolMeta>);

impl ToolMeta {
    pub fn new_with_default_schema(name: &str, title: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            input_schema: ToolInputSchema::default(),
        }
    }
}