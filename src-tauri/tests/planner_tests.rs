use vectorless_lib::reasoner::planner::{
    Planner, PlannerConfig, PlannerDecision, PlannerInput, StepType,
};

#[test]
fn planner_emits_default_reasoning_sequence() {
    let planner = Planner::new(PlannerConfig::default());
    let input = PlannerInput {
        query: "What are the limitations of this approach?".to_string(),
        last_confidence: None,
        explored_sections: vec![],
        has_evidence: false,
        step_count: 0,
        backtrack_count: 0,
    };

    let plan = planner.next_steps(&input);
    let kinds: Vec<StepType> = plan.steps.iter().map(|item| item.step_type.clone()).collect();

    assert_eq!(
        kinds,
        vec![
            StepType::ScanRoot,
            StepType::SelectSections,
            StepType::DrillDown,
            StepType::ExtractEvidence,
            StepType::Synthesize,
            StepType::SelfCheck,
        ]
    );
}

#[test]
fn planner_backtracks_when_confidence_is_low() {
    let planner = Planner::new(PlannerConfig::default());
    let input = PlannerInput {
        query: "Find the read latency limit".to_string(),
        last_confidence: Some(0.42),
        explored_sections: vec!["2.0 Overview".to_string()],
        has_evidence: true,
        step_count: 5,
        backtrack_count: 0,
    };

    let plan = planner.next_steps(&input);

    assert_eq!(plan.decision, PlannerDecision::Backtrack);
    assert!(plan
        .steps
        .iter()
        .any(|step| step.step_type == StepType::SelectSections));
}

#[test]
fn planner_stops_after_max_steps() {
    let planner = Planner::new(PlannerConfig {
        max_steps: 6,
        max_backtracks: 2,
        confidence_threshold: 0.70,
    });

    let input = PlannerInput {
        query: "Any query".to_string(),
        last_confidence: Some(0.20),
        explored_sections: vec![],
        has_evidence: false,
        step_count: 6,
        backtrack_count: 0,
    };

    let plan = planner.next_steps(&input);

    assert_eq!(plan.decision, PlannerDecision::Stop);
    assert!(plan.steps.is_empty());
}
