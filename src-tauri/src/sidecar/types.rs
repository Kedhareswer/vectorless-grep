use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarDocument {
    pub title: String,
    pub pages: i64,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarNode {
    pub id: String,
    pub parent_id: Option<String>,
    pub node_type: String,
    pub title: String,
    pub text: String,
    pub page_start: Option<i64>,
    pub page_end: Option<i64>,
    pub ordinal_path: String,
    pub bbox: Value,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedPayload {
    pub document: SidecarDocument,
    pub nodes: Vec<SidecarNode>,
    pub edges: Vec<SidecarEdge>,
}
