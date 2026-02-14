use vectorless_lib::{
    core::errors::AppError,
    db::{repositories::documents, Database},
    sidecar::types::SidecarNode,
};

fn node(
    id: &str,
    parent_id: Option<&str>,
    node_type: &str,
    title: &str,
    text: &str,
    ordinal_path: &str,
) -> SidecarNode {
    SidecarNode {
        id: id.to_string(),
        parent_id: parent_id.map(ToString::to_string),
        node_type: node_type.to_string(),
        title: title.to_string(),
        text: text.to_string(),
        page_start: Some(1),
        page_end: Some(1),
        ordinal_path: ordinal_path.to_string(),
        bbox: serde_json::json!({}),
        metadata: serde_json::json!({}),
    }
}

#[tokio::test]
async fn get_document_preview_returns_all_nodes_in_ordinal_order() {
    let db = Database::in_memory().await.expect("db should initialize");
    let document_id = "doc-preview-order";

    documents::insert_document(
        db.pool(),
        document_id,
        "project-default",
        "order.txt",
        "text/plain",
        "checksum-order",
        1,
    )
    .await
    .expect("insert document");

    documents::insert_nodes(
        db.pool(),
        document_id,
        &[
            node("root-order", None, "Document", "Order Doc", "", "root"),
            node("sec-2", Some("root-order"), "Section", "Second", "second", "2"),
            node("sec-1", Some("root-order"), "Section", "First", "first", "1"),
            node("para-1", Some("sec-1"), "Paragraph", "", "paragraph", "1.1"),
        ],
    )
    .await
    .expect("insert nodes");

    let preview = documents::get_document_preview(db.pool(), document_id)
        .await
        .expect("preview query");

    let ordered_ids: Vec<&str> = preview.iter().map(|node| node.id.as_str()).collect();
    assert_eq!(ordered_ids, vec!["root-order", "sec-1", "para-1", "sec-2"]);
}

#[tokio::test]
async fn get_document_preview_filters_by_document_id() {
    let db = Database::in_memory().await.expect("db should initialize");

    documents::insert_document(
        db.pool(),
        "doc-preview-a",
        "project-default",
        "a.txt",
        "text/plain",
        "checksum-a",
        1,
    )
    .await
    .expect("insert doc a");
    documents::insert_document(
        db.pool(),
        "doc-preview-b",
        "project-default",
        "b.txt",
        "text/plain",
        "checksum-b",
        1,
    )
    .await
    .expect("insert doc b");

    documents::insert_nodes(
        db.pool(),
        "doc-preview-a",
        &[node("root-a", None, "Document", "Doc A", "", "root")],
    )
    .await
    .expect("insert nodes a");

    documents::insert_nodes(
        db.pool(),
        "doc-preview-b",
        &[node("root-b", None, "Document", "Doc B", "", "root")],
    )
    .await
    .expect("insert nodes b");

    let preview = documents::get_document_preview(db.pool(), "doc-preview-a")
        .await
        .expect("preview query");

    assert_eq!(preview.len(), 1);
    assert_eq!(preview[0].id, "root-a");
    assert!(preview.iter().all(|item| item.document_id == "doc-preview-a"));
}

#[tokio::test]
async fn get_document_preview_handles_missing_document() {
    let db = Database::in_memory().await.expect("db should initialize");

    let err = documents::get_document_preview(db.pool(), "missing-doc")
        .await
        .expect_err("missing document should fail");

    match err {
        AppError::NotFound(message) => assert!(message.contains("missing-doc")),
        other => panic!("expected AppError::NotFound, got {other:?}"),
    }
}
