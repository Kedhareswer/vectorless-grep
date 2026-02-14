use vectorless_lib::sidecar::docling_client::normalize_sidecar_payload;

#[test]
fn sidecar_normalization_promotes_root_document_and_orders_nodes() {
    let payload = serde_json::json!({
        "document": {
            "title": "Spec",
            "pages": 4,
            "metadata": {"source": "spec.pdf"}
        },
        "nodes": [
            {
                "id": "n2",
                "parent_id": "n1",
                "type": "Paragraph",
                "title": "Body",
                "text": "Latency details",
                "page_start": 2,
                "page_end": 2,
                "ordinal_path": "2.1.p.1"
            },
            {
                "id": "n1",
                "parent_id": null,
                "type": "Section",
                "title": "Performance",
                "text": "",
                "page_start": 2,
                "page_end": 3,
                "ordinal_path": "2.1"
            }
        ],
        "edges": []
    });

    let normalized = normalize_sidecar_payload(&payload).expect("payload should normalize");

    assert_eq!(normalized.document.title, "Spec");
    assert_eq!(normalized.nodes[0].ordinal_path, "root");
    assert_eq!(normalized.nodes[1].ordinal_path, "2.1");
    assert_eq!(normalized.nodes[2].ordinal_path, "2.1.p.1");
}

#[test]
fn sidecar_normalization_rejects_empty_nodes() {
    let payload = serde_json::json!({
        "document": {"title": "Empty", "pages": 1, "metadata": {}},
        "nodes": [],
        "edges": []
    });

    let err = normalize_sidecar_payload(&payload).expect_err("expected parse error");
    assert!(err.to_string().contains("at least one node"));
}
