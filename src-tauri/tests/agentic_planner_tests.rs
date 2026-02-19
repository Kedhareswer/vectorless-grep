use vectorless_lib::{
    providers::gemini::GeminiPlannerStep,
    reasoner::planner::{Planner, PlannerConfig, PlannerDecision, PlannerInput, StepType},
};

fn input() -> PlannerInput {
    PlannerInput {
        query: "How are these files related?".to_string(),
        last_confidence: Some(0.41),
        explored_sections: vec![],
        has_evidence: false,
        step_count: 1,
        backtrack_count: 0,
    }
}

fn input_with_evidence(has_evidence: bool) -> PlannerInput {
    PlannerInput {
        query: "How are these files related?".to_string(),
        last_confidence: Some(0.41),
        explored_sections: vec![],
        has_evidence,
        step_count: 1,
        backtrack_count: 0,
    }
}

#[test]
fn model_plan_maps_search_to_retrieval_steps() {
    let planner = Planner::new(PlannerConfig::default());
    let model_step = GeminiPlannerStep {
        step_type: "search".to_string(),
        objective: "Find candidate sections across files".to_string(),
        reasoning: "Need broad context first".to_string(),
        decision: "continue".to_string(),
    };

    let plan = planner
        .next_steps_from_model(&input(), &model_step)
        .expect("valid model plan");

    assert_eq!(plan.decision, PlannerDecision::Continue);
    assert!(plan
        .steps
        .iter()
        .any(|step| step.step_type == StepType::ScanRoot));
    assert!(plan
        .steps
        .iter()
        .any(|step| step.step_type == StepType::SelectSections));
}

#[test]
fn invalid_model_step_is_rejected() {
    let planner = Planner::new(PlannerConfig::default());
    let model_step = GeminiPlannerStep {
        step_type: "nonsense".to_string(),
        objective: "Unknown".to_string(),
        reasoning: "Unknown".to_string(),
        decision: "continue".to_string(),
    };

    assert!(planner
        .next_steps_from_model(&input(), &model_step)
        .is_none());
}

#[test]
fn finish_decision_stops_sequence() {
    let planner = Planner::new(PlannerConfig::default());
    let model_step = GeminiPlannerStep {
        step_type: "finish".to_string(),
        objective: "Stop now".to_string(),
        reasoning: "Answer quality is sufficient".to_string(),
        decision: "stop".to_string(),
    };

    let plan = planner
        .next_steps_from_model(&input_with_evidence(true), &model_step)
        .expect("valid finish plan");
    assert_eq!(plan.decision, PlannerDecision::Stop);
    assert!(plan.steps.is_empty());
}

#[test]
fn finish_without_evidence_falls_back_to_search() {
    let planner = Planner::new(PlannerConfig::default());
    let model_step = GeminiPlannerStep {
        step_type: "finish".to_string(),
        objective: "Stop now".to_string(),
        reasoning: "Done".to_string(),
        decision: "stop".to_string(),
    };

    let plan = planner
        .next_steps_from_model(&input_with_evidence(false), &model_step)
        .expect("fallback plan");

    assert_eq!(plan.decision, PlannerDecision::Continue);
    assert!(plan
        .steps
        .iter()
        .any(|step| step.step_type == StepType::ScanRoot));
}
