use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Gemini,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Document,
    Section,
    Subsection,
    Paragraph,
    Claim,
    Table,
    Figure,
    Equation,
    Caption,
    Reference,
    Unknown,
}

impl NodeType {
    pub fn from_str(raw: &str) -> Self {
        match raw.to_ascii_lowercase().as_str() {
            "document" => Self::Document,
            "section" => Self::Section,
            "subsection" => Self::Subsection,
            "paragraph" => Self::Paragraph,
            "claim" => Self::Claim,
            "table" => Self::Table,
            "figure" => Self::Figure,
            "equation" => Self::Equation,
            "caption" => Self::Caption,
            "reference" => Self::Reference,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetProviderKeyResponse {
    pub stored: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestDocumentResponse {
    pub document_id: String,
    pub root_node_id: String,
    pub node_count: usize,
    pub section_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListProjectsResponse {
    pub projects: Vec<ProjectSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectResponse {
    pub project: ProjectSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameProjectResponse {
    pub project: ProjectSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteProjectResponse {
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSummary {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub mime: String,
    pub checksum: String,
    pub pages: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDocumentsResponse {
    pub documents: Vec<DocumentSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenDocumentResponse {
    pub document: DocumentSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocNodeSummary {
    pub id: String,
    pub document_id: String,
    pub parent_id: Option<String>,
    pub node_type: NodeType,
    pub title: String,
    pub text: String,
    pub ordinal_path: String,
    pub page_start: Option<i64>,
    pub page_end: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocNodeDetail {
    pub id: String,
    pub document_id: String,
    pub parent_id: Option<String>,
    pub node_type: NodeType,
    pub title: String,
    pub text: String,
    pub ordinal_path: String,
    pub page_start: Option<i64>,
    pub page_end: Option<i64>,
    pub bbox_json: Value,
    pub metadata_json: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTreeResponse {
    pub nodes: Vec<DocNodeSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetNodeResponse {
    pub node: DocNodeDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReasoningRun {
    pub id: String,
    pub project_id: String,
    pub document_id: Option<String>,
    pub query: String,
    pub status: RunStatus,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub total_latency_ms: Option<i64>,
    pub token_usage_json: Value,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReasoningStep {
    pub run_id: String,
    pub idx: i64,
    pub step_type: String,
    pub thought: String,
    pub action: String,
    pub observation: String,
    pub node_refs: Vec<String>,
    pub confidence: f64,
    pub latency_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnswerRecord {
    pub run_id: String,
    pub answer_markdown: String,
    pub citations: Vec<String>,
    pub confidence: f64,
    pub grounded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunReasoningQueryResponse {
    pub run_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRunResponse {
    pub run: ReasoningRun,
    pub steps: Vec<ReasoningStep>,
    pub answer: Option<AnswerRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportMarkdownResponse {
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteDocumentResponse {
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestProgressEvent {
    pub job_id: String,
    pub stage: String,
    pub percent: i64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReasoningStepEvent {
    pub run_id: String,
    pub step_index: i64,
    pub step_type: String,
    pub thought: String,
    pub action: String,
    pub observation: String,
    pub node_refs: Vec<String>,
    pub latency_ms: i64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphNodePosition {
    pub node_id: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetGraphLayoutResponse {
    pub document_id: String,
    pub positions: Vec<GraphNodePosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveGraphLayoutResponse {
    pub saved: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReasoningCompleteEvent {
    pub run_id: String,
    pub answer_id: String,
    pub final_confidence: f64,
    pub total_latency_ms: i64,
    pub token_usage: Value,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReasoningErrorEvent {
    pub run_id: String,
    pub code: String,
    pub message: String,
    pub retryable: bool,
}
