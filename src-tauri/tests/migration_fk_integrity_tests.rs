use sqlx::Row;
use vectorless_lib::{
    db::{repositories::documents, Database},
    sidecar::types::SidecarNode,
};

fn has_legacy_fk_target(rows: Vec<sqlx::sqlite::SqliteRow>) -> bool {
    rows.into_iter().any(|row| {
        let table: String = row.get("table");
        table.ends_with("_old")
    })
}

#[tokio::test]
async fn migrated_schema_has_no_old_foreign_key_targets() {
    let db = Database::in_memory().await.expect("db should initialize");

    let doc_nodes_fks = sqlx::query("PRAGMA foreign_key_list(doc_nodes);")
        .fetch_all(db.pool())
        .await
        .expect("doc_nodes fk list");
    let graph_layouts_fks = sqlx::query("PRAGMA foreign_key_list(graph_layouts);")
        .fetch_all(db.pool())
        .await
        .expect("graph_layouts fk list");
    let reasoning_steps_fks = sqlx::query("PRAGMA foreign_key_list(reasoning_steps);")
        .fetch_all(db.pool())
        .await
        .expect("reasoning_steps fk list");
    let answers_fks = sqlx::query("PRAGMA foreign_key_list(answers);")
        .fetch_all(db.pool())
        .await
        .expect("answers fk list");

    assert!(
        !has_legacy_fk_target(doc_nodes_fks),
        "doc_nodes should not reference *_old tables"
    );
    assert!(
        !has_legacy_fk_target(graph_layouts_fks),
        "graph_layouts should not reference *_old tables"
    );
    assert!(
        !has_legacy_fk_target(reasoning_steps_fks),
        "reasoning_steps should not reference *_old tables"
    );
    assert!(
        !has_legacy_fk_target(answers_fks),
        "answers should not reference *_old tables"
    );
}

#[tokio::test]
async fn insert_document_then_nodes_succeeds_after_migrations() {
    let db = Database::in_memory().await.expect("db should initialize");
    let doc_id = "doc-migration-check";

    documents::insert_document(
        db.pool(),
        doc_id,
        "project-default",
        "migration-check.txt",
        "text/plain",
        "migration-check-checksum",
        1,
    )
    .await
    .expect("insert document");

    let nodes = vec![
        SidecarNode {
            id: "root-migration-check".to_string(),
            parent_id: None,
            node_type: "Document".to_string(),
            title: "migration-check".to_string(),
            text: "".to_string(),
            page_start: Some(1),
            page_end: Some(1),
            ordinal_path: "root".to_string(),
            bbox: serde_json::json!({}),
            metadata: serde_json::json!({}),
        },
        SidecarNode {
            id: "sec-migration-check".to_string(),
            parent_id: Some("root-migration-check".to_string()),
            node_type: "Section".to_string(),
            title: "Section".to_string(),
            text: "content".to_string(),
            page_start: Some(1),
            page_end: Some(1),
            ordinal_path: "1".to_string(),
            bbox: serde_json::json!({}),
            metadata: serde_json::json!({}),
        },
    ];

    documents::insert_nodes(db.pool(), doc_id, &nodes)
        .await
        .expect("insert nodes should succeed without legacy fk errors");
}
