use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolInputSchema {
    #[serde(rename = "type")]
    pub type_: String,
    pub properties: Value,
    pub required: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolMeta {
    pub name: String,
    pub title: String,
    pub description: String,
    pub input_schema: ToolInputSchema,
}
