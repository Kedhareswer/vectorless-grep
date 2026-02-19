use sqlx::Row;

use vectorless_lib::{
    db::Database,
    reasoner::agent_schema::{AgentPlannedStep, RunPhase},
};

#[tokio::test]
async fn reasoning_runs_table_has_workflow_columns() {
    let db = Database::in_memory().await.expect("db should initialize");

    let columns = sqlx::query("PRAGMA table_info(reasoning_runs);")
        .fetch_all(db.pool())
        .await
        .expect("table info");

    let names = columns
        .iter()
        .map(|row| row.get::<String, _>("name"))
        .collect::<Vec<_>>();

    assert!(names.iter().any(|name| name == "phase"));
    assert!(names.iter().any(|name| name == "quality_json"));
    assert!(names.iter().any(|name| name == "planner_trace_json"));
}

#[test]
fn agent_planned_step_requires_valid_step_type_and_required_fields() {
    let valid = r#"{
      "stepType": "search",
      "objective": "Find relevant sections",
      "reasoning": "Need broad coverage first",
      "params": { "limit": 6 },
      "stop": false
    }"#;
    let parsed: AgentPlannedStep = serde_json::from_str(valid).expect("valid step");
    assert_eq!(parsed.phase(), RunPhase::Retrieval);

    let invalid = r#"{
      "stepType": "nonsense",
      "objective": "Bad",
      "reasoning": "Bad",
      "params": {},
      "stop": false
    }"#;
    assert!(serde_json::from_str::<AgentPlannedStep>(invalid).is_err());

    let missing_field = r#"{
      "stepType": "search",
      "objective": "Bad",
      "params": {},
      "stop": false
    }"#;
    assert!(serde_json::from_str::<AgentPlannedStep>(missing_field).is_err());
}
