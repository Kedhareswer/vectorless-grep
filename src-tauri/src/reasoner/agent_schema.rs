use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use crate::core::types::RunPhase;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentStepType {
    Search,
    Inspect,
    Synthesize,
    SelfCheck,
    Finish,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgentPlannedStep {
    pub step_type: AgentStepType,
    pub objective: String,
    pub reasoning: String,
    pub params: Value,
    pub stop: bool,
}

impl AgentPlannedStep {
    pub fn phase(&self) -> RunPhase {
        match self.step_type {
            AgentStepType::Search | AgentStepType::Inspect => RunPhase::Retrieval,
            AgentStepType::Synthesize => RunPhase::Synthesis,
            AgentStepType::SelfCheck => RunPhase::Validation,
            AgentStepType::Finish => RunPhase::Completed,
        }
    }
}
