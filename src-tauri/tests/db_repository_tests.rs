use vectorless_lib::{
    core::types::GraphNodePosition,
    db::{repositories::documents, Database},
    sidecar::types::SidecarNode,
};

#[tokio::test]
async fn document_repository_persists_tree_nodes() {
    let db = Database::in_memory().await.expect("db should initialize");
    let doc_id = "doc-1";
    documents::insert_document(
        db.pool(),
        doc_id,
        "project-default",
        "Spec.pdf",
        "application/pdf",
        "checksum-1",
        3,
    )
    .await
    .expect("insert document");

    let nodes = vec![
        SidecarNode {
            id: "root-1".to_string(),
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
            id: "sec-1".to_string(),
            parent_id: Some("root-1".to_string()),
            node_type: "Section".to_string(),
            title: "Introduction".to_string(),
            text: "Intro text".to_string(),
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

    let tree = documents::get_tree(db.pool(), doc_id, None, 6)
        .await
        .expect("query tree");
    assert_eq!(tree.len(), 2);
    assert_eq!(tree[0].id, "root-1");
    assert_eq!(tree[1].id, "sec-1");
}

#[tokio::test]
async fn graph_layout_upsert_and_read_roundtrip() {
    let db = Database::in_memory().await.expect("db should initialize");
    let doc_id = "doc-graph-1";
    documents::insert_document(
        db.pool(),
        doc_id,
        "project-default",
        "Spec.pdf",
        "application/pdf",
        "checksum-graph-1",
        3,
    )
    .await
    .expect("insert document");

    let nodes = vec![
        SidecarNode {
            id: "root-graph-1".to_string(),
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
            id: "sec-graph-1".to_string(),
            parent_id: Some("root-graph-1".to_string()),
            node_type: "Section".to_string(),
            title: "Introduction".to_string(),
            text: "Intro text".to_string(),
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

    let positions = vec![
        GraphNodePosition {
            node_id: "root-graph-1".to_string(),
            x: 10.0,
            y: 20.0,
        },
        GraphNodePosition {
            node_id: "sec-graph-1".to_string(),
            x: 30.0,
            y: 40.0,
        },
    ];

    documents::save_graph_layout(db.pool(), doc_id, &positions)
        .await
        .expect("save graph layout");
    let loaded = documents::get_graph_layout(db.pool(), doc_id)
        .await
        .expect("load graph layout");

    assert_eq!(loaded.len(), 2);
    assert!(loaded.iter().any(|position| {
        position.node_id == "root-graph-1" && position.x == 10.0 && position.y == 20.0
    }));
    assert!(loaded.iter().any(|position| {
        position.node_id == "sec-graph-1" && position.x == 30.0 && position.y == 40.0
    }));
}

#[tokio::test]
async fn graph_layout_overwrite_updates_existing_positions() {
    let db = Database::in_memory().await.expect("db should initialize");
    let doc_id = "doc-graph-2";
    documents::insert_document(
        db.pool(),
        doc_id,
        "project-default",
        "Spec.pdf",
        "application/pdf",
        "checksum-graph-2",
        3,
    )
    .await
    .expect("insert document");

    let nodes = vec![SidecarNode {
        id: "root-graph-2".to_string(),
        parent_id: None,
        node_type: "Document".to_string(),
        title: "Spec".to_string(),
        text: "".to_string(),
        page_start: Some(1),
        page_end: Some(3),
        ordinal_path: "root".to_string(),
        bbox: serde_json::json!({}),
        metadata: serde_json::json!({}),
    }];
    documents::insert_nodes(db.pool(), doc_id, &nodes)
        .await
        .expect("insert nodes");

    let first = vec![GraphNodePosition {
        node_id: "root-graph-2".to_string(),
        x: 5.0,
        y: 8.0,
    }];
    documents::save_graph_layout(db.pool(), doc_id, &first)
        .await
        .expect("save first");

    let second = vec![GraphNodePosition {
        node_id: "root-graph-2".to_string(),
        x: 55.0,
        y: 88.0,
    }];
    documents::save_graph_layout(db.pool(), doc_id, &second)
        .await
        .expect("save second");

    let loaded = documents::get_graph_layout(db.pool(), doc_id)
        .await
        .expect("load graph layout");
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].x, 55.0);
    assert_eq!(loaded[0].y, 88.0);
}

#[tokio::test]
async fn graph_layout_deleted_with_document_cascade() {
    let db = Database::in_memory().await.expect("db should initialize");
    let doc_id = "doc-graph-3";
    documents::insert_document(
        db.pool(),
        doc_id,
        "project-default",
        "Spec.pdf",
        "application/pdf",
        "checksum-graph-3",
        3,
    )
    .await
    .expect("insert document");

    let nodes = vec![SidecarNode {
        id: "root-graph-3".to_string(),
        parent_id: None,
        node_type: "Document".to_string(),
        title: "Spec".to_string(),
        text: "".to_string(),
        page_start: Some(1),
        page_end: Some(3),
        ordinal_path: "root".to_string(),
        bbox: serde_json::json!({}),
        metadata: serde_json::json!({}),
    }];
    documents::insert_nodes(db.pool(), doc_id, &nodes)
        .await
        .expect("insert nodes");

    let positions = vec![GraphNodePosition {
        node_id: "root-graph-3".to_string(),
        x: 77.0,
        y: 99.0,
    }];
    documents::save_graph_layout(db.pool(), doc_id, &positions)
        .await
        .expect("save graph layout");

    let deleted = documents::delete_document(db.pool(), doc_id)
        .await
        .expect("delete doc");
    assert!(deleted);

    let loaded = documents::get_graph_layout(db.pool(), doc_id)
        .await
        .expect("load graph layout");
    assert!(loaded.is_empty());
}
