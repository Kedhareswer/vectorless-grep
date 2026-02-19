use serde::{Deserialize, Serialize};

use crate::providers::gemini::GeminiPlannerStep;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    ScanRoot,
    SelectSections,
    DrillDown,
    ExtractEvidence,
    Synthesize,
    SelfCheck,
}

impl StepType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ScanRoot => "scan_root",
            Self::SelectSections => "select_sections",
            Self::DrillDown => "drill_down",
            Self::ExtractEvidence => "extract_evidence",
            Self::Synthesize => "synthesize",
            Self::SelfCheck => "self_check",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlannerDecision {
    Continue,
    Backtrack,
    Stop,
}

#[derive(Debug, Clone)]
pub struct PlannerConfig {
    pub max_steps: usize,
    pub max_backtracks: usize,
    pub confidence_threshold: f64,
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self {
            max_steps: 6,
            max_backtracks: 2,
            confidence_threshold: 0.70,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlannerInput {
    pub query: String,
    pub last_confidence: Option<f64>,
    pub explored_sections: Vec<String>,
    pub has_evidence: bool,
    pub step_count: usize,
    pub backtrack_count: usize,
}

#[derive(Debug, Clone)]
pub struct PlannedStep {
    pub step_type: StepType,
    pub objective: String,
}

#[derive(Debug, Clone)]
pub struct PlannedSequence {
    pub decision: PlannerDecision,
    pub steps: Vec<PlannedStep>,
}

#[derive(Debug, Clone)]
pub struct Planner {
    config: PlannerConfig,
}

impl Planner {
    pub fn new(config: PlannerConfig) -> Self {
        Self { config }
    }

    pub fn next_steps(&self, input: &PlannerInput) -> PlannedSequence {
        if input.step_count >= self.config.max_steps {
            return PlannedSequence {
                decision: PlannerDecision::Stop,
                steps: vec![],
            };
        }

        if let Some(confidence) = input.last_confidence {
            if confidence < self.config.confidence_threshold
                && input.backtrack_count < self.config.max_backtracks
            {
                return PlannedSequence {
                    decision: PlannerDecision::Backtrack,
                    steps: vec![
                        PlannedStep {
                            step_type: StepType::SelectSections,
                            objective: format!(
                                "Re-select sections for query '{}' skipping explored branches",
                                input.query
                            ),
                        },
                        PlannedStep {
                            step_type: StepType::DrillDown,
                            objective: "Drill into candidate subsections".to_string(),
                        },
                        PlannedStep {
                            step_type: StepType::ExtractEvidence,
                            objective: "Extract stronger evidence nodes".to_string(),
                        },
                        PlannedStep {
                            step_type: StepType::Synthesize,
                            objective: "Synthesize revised answer".to_string(),
                        },
                        PlannedStep {
                            step_type: StepType::SelfCheck,
                            objective: "Estimate grounded confidence".to_string(),
                        },
                    ],
                };
            }
        }

        if input.has_evidence {
            return PlannedSequence {
                decision: PlannerDecision::Continue,
                steps: vec![
                    PlannedStep {
                        step_type: StepType::Synthesize,
                        objective: "Build answer from evidence".to_string(),
                    },
                    PlannedStep {
                        step_type: StepType::SelfCheck,
                        objective: "Check grounding and confidence".to_string(),
                    },
                ],
            };
        }

        let mut objective = "Scan root table-of-contents for broad candidates".to_string();
        if !input.explored_sections.is_empty() {
            objective.push_str("; avoid previously explored sections");
        }

        PlannedSequence {
            decision: PlannerDecision::Continue,
            steps: vec![
                PlannedStep {
                    step_type: StepType::ScanRoot,
                    objective,
                },
                PlannedStep {
                    step_type: StepType::SelectSections,
                    objective: "Select sections relevant to user query".to_string(),
                },
                PlannedStep {
                    step_type: StepType::DrillDown,
                    objective: "Navigate into subsections and atomic nodes".to_string(),
                },
                PlannedStep {
                    step_type: StepType::ExtractEvidence,
                    objective: "Extract claim/table/equation evidence".to_string(),
                },
                PlannedStep {
                    step_type: StepType::Synthesize,
                    objective: "Synthesize grounded answer".to_string(),
                },
                PlannedStep {
                    step_type: StepType::SelfCheck,
                    objective: "Measure confidence and decide if re-traversal is needed"
                        .to_string(),
                },
            ],
        }
    }

    pub fn next_steps_from_model(
        &self,
        input: &PlannerInput,
        model_step: &GeminiPlannerStep,
    ) -> Option<PlannedSequence> {
        if input.step_count >= self.config.max_steps {
            return Some(PlannedSequence {
                decision: PlannerDecision::Stop,
                steps: vec![],
            });
        }

        let decision = parse_decision(&model_step.decision);
        if matches!(decision, PlannerDecision::Stop) {
            if !input.has_evidence {
                return Some(PlannedSequence {
                    decision: PlannerDecision::Continue,
                    steps: vec![
                        PlannedStep {
                            step_type: StepType::ScanRoot,
                            objective: "Need evidence before finishing".to_string(),
                        },
                        PlannedStep {
                            step_type: StepType::SelectSections,
                            objective: "Find relevant candidate sections".to_string(),
                        },
                    ],
                });
            }
            return Some(PlannedSequence {
                decision,
                steps: vec![],
            });
        }

        if matches!(decision, PlannerDecision::Backtrack) {
            return Some(PlannedSequence {
                decision,
                steps: vec![
                    PlannedStep {
                        step_type: StepType::SelectSections,
                        objective: model_step.objective.clone(),
                    },
                    PlannedStep {
                        step_type: StepType::DrillDown,
                        objective: "Re-check alternate branches".to_string(),
                    },
                    PlannedStep {
                        step_type: StepType::ExtractEvidence,
                        objective: "Collect stronger supporting evidence".to_string(),
                    },
                    PlannedStep {
                        step_type: StepType::Synthesize,
                        objective: "Regenerate answer from revised evidence".to_string(),
                    },
                    PlannedStep {
                        step_type: StepType::SelfCheck,
                        objective: "Validate revised answer quality".to_string(),
                    },
                ],
            });
        }

        let steps = match parse_step_kind(&model_step.step_type)? {
            StepType::ScanRoot => vec![
                PlannedStep {
                    step_type: StepType::ScanRoot,
                    objective: model_step.objective.clone(),
                },
                PlannedStep {
                    step_type: StepType::SelectSections,
                    objective: "Select high-signal sections".to_string(),
                },
            ],
            StepType::DrillDown => vec![
                PlannedStep {
                    step_type: StepType::DrillDown,
                    objective: model_step.objective.clone(),
                },
                PlannedStep {
                    step_type: StepType::ExtractEvidence,
                    objective: "Extract concrete supporting claims".to_string(),
                },
            ],
            StepType::Synthesize => vec![PlannedStep {
                step_type: StepType::Synthesize,
                objective: model_step.objective.clone(),
            }],
            StepType::SelfCheck => vec![PlannedStep {
                step_type: StepType::SelfCheck,
                objective: model_step.objective.clone(),
            }],
            StepType::SelectSections | StepType::ExtractEvidence => vec![PlannedStep {
                step_type: parse_step_kind(&model_step.step_type)?,
                objective: model_step.objective.clone(),
            }],
        };

        Some(PlannedSequence {
            decision: PlannerDecision::Continue,
            steps,
        })
    }
}

fn parse_decision(raw: &str) -> PlannerDecision {
    match raw.trim().to_ascii_lowercase().as_str() {
        "stop" | "finish" | "done" => PlannerDecision::Stop,
        "backtrack" | "revise" | "retry" => PlannerDecision::Backtrack,
        _ => PlannerDecision::Continue,
    }
}

fn parse_step_kind(raw: &str) -> Option<StepType> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "search" | "scan_root" => Some(StepType::ScanRoot),
        "select_sections" => Some(StepType::SelectSections),
        "inspect" | "drill_down" => Some(StepType::DrillDown),
        "extract_evidence" => Some(StepType::ExtractEvidence),
        "synthesize" => Some(StepType::Synthesize),
        "self_check" | "validate" => Some(StepType::SelfCheck),
        "finish" => Some(StepType::SelfCheck),
        _ => None,
    }
}
