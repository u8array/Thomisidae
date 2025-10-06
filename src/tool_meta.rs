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

#[derive(Serialize, Clone)]
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

impl Default for ToolMeta {
    fn default() -> Self {
        ToolMeta {
            name: String::new(),
            title: String::new(),
            description: String::new(),
            input_schema: ToolInputSchema::default(),
        }
    }
}

impl ToolMeta {
    pub fn builder() -> ToolMetaBuilder {
        ToolMetaBuilder::default()
    }
}

#[derive(Default)]
pub struct ToolMetaBuilder {
    name: String,
    title: String,
    description: String,
    input_schema: Option<ToolInputSchema>,
}

impl ToolMetaBuilder {
    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }
    pub fn description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }
    pub fn input_schema(mut self, input_schema: ToolInputSchema) -> Self {
        self.input_schema = Some(input_schema);
        self
    }
    pub fn build(self) -> ToolMeta {
        ToolMeta {
            name: self.name,
            title: self.title,
            description: self.description,
            input_schema: self.input_schema.unwrap_or_else(ToolInputSchema::default),
        }
    }
}
