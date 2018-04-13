
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpecFormat {
    Block,
    Inline
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpecPriority {
    Required,
    Optional
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecTemplate {
    #[serde(rename = "id")]
    pub identifier: String,
    pub description: String,
    pub names: Vec<String>,
    pub attributes: Vec<SpecAttribute>,
    pub format: SpecFormat,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecAttribute {
    #[serde(rename = "id")]
    pub identifier: String,
    pub names: Vec<String>,
    pub priority: SpecPriority,
    pub predicate: String,
}
