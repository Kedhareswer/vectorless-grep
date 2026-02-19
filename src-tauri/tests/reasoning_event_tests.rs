use std::sync::{Arc, Mutex};

use vectorless_lib::{
    core::errors::AppError,
    db::{repositories::documents, Database},
    providers::gemini::GeminiClient,
    reasoner::executor::ReasoningExecutor,
    sidecar::types::SidecarNode,
};

#[tokio::test]
async fn reasoning_step_event_includes_node_refs() {
    let db = Database::in_memory().await.expect("db should initialize");
    let doc_id = "doc-reasoning-1";
    documents::insert_document(
        db.pool(),
        doc_id,
        "project-default",
        "Spec.pdf",
        "application/pdf",
        "checksum-reasoning-1",
        3,
    )
    .await
    .expect("insert document");

    let nodes = vec![
        SidecarNode {
            id: "root-reasoning-1".to_string(),
            parent_id: None,
            node_type: "Document".to_string(),
            title: "Spec".to_string(),
            text: "".to_string(),
            page_start: Some(1),
            page_end: Some(3),
            ordinal_path: "root".to_string(),
            bbox: serde_json::json!({}),
            metadata: serde_json::json!({}),
        },
        SidecarNode {
            id: "sec-reasoning-1".to_string(),
            parent_id: Some("root-reasoning-1".to_string()),
            node_type: "Section".to_string(),
            title: "Latency".to_string(),
            text: "Latency dropped to 50ms p99.".to_string(),
            page_start: Some(1),
            page_end: Some(1),
            ordinal_path: "1".to_string(),
            bbox: serde_json::json!({}),
            metadata: serde_json::json!({}),
        },
    ];
    documents::insert_nodes(db.pool(), doc_id, &nodes)
        .await
        .expect("insert nodes");

    let client = GeminiClient::new("gemini-2.0-flash").expect("gemini client");
    let executor = ReasoningExecutor::new(client);
    let events = Arc::new(Mutex::new(vec![]));
    let events_ref = Arc::clone(&events);

    let result = executor
        .run(
            &db,
            "project-default",
            Some(doc_id),
            "run-reasoning-1".to_string(),
            "What is the latency?",
            Some(2),
            "test-key-not-used",
            move |event| {
                events_ref.lock().expect("events lock").push(event);
            },
        )
        .await;

    assert!(
        matches!(result, Err(AppError::QualityGateFailed(_))),
        "quality gate should reject incomplete low-quality runs"
    );

    let observed = events.lock().expect("events lock");
    assert!(!observed.is_empty(), "expected at least one reasoning step event");
    assert!(
        observed.iter().any(|event| !event.node_refs.is_empty()),
        "expected at least one step to include node references",
    );
}

#[tokio::test]
async fn reasoning_run_without_synthesis_is_rejected_by_quality_gate() {
    let db = Database::in_memory().await.expect("db should initialize");
    let doc_id = "doc-reasoning-2";
    documents::insert_document(
        db.pool(),
        doc_id,
        "project-default",
        "Spec.pdf",
        "application/pdf",
        "checksum-reasoning-2",
        3,
    )
    .await
    .expect("insert document");

    let nodes = vec![
        SidecarNode {
            id: "root-reasoning-2".to_string(),
            parent_id: None,
            node_type: "Document".to_string(),
            title: "Spec".to_string(),
            text: "".to_string(),
            page_start: Some(1),
            page_end: Some(3),
            ordinal_path: "root".to_string(),
            bbox: serde_json::json!({}),
            metadata: serde_json::json!({}),
        },
        SidecarNode {
            id: "sec-reasoning-2".to_string(),
            parent_id: Some("root-reasoning-2".to_string()),
            node_type: "Section".to_string(),
            title: "Overview".to_string(),
            text: "This file explains model architecture and results.".to_string(),
            page_start: Some(1),
            page_end: Some(1),
            ordinal_path: "1".to_string(),
            bbox: serde_json::json!({}),
            metadata: serde_json::json!({}),
        },
    ];
    documents::insert_nodes(db.pool(), doc_id, &nodes)
        .await
        .expect("insert nodes");

    let client = GeminiClient::new("gemini-2.0-flash").expect("gemini client");
    let executor = ReasoningExecutor::new(client);

    let result = executor
        .run(
            &db,
            "project-default",
            Some(doc_id),
            "run-reasoning-2".to_string(),
            "Explain this file",
            Some(2),
            "test-key-not-used",
            |_| {},
        )
        .await;

    assert!(
        matches!(result, Err(AppError::QualityGateFailed(_))),
        "quality gate should reject runs that never synthesize grounded answers"
    );
}
