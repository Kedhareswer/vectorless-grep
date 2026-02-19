use vectorless_lib::reasoner::query_scope::requires_project_scope;

#[test]
fn detects_multi_document_relation_queries() {
    assert!(requires_project_scope(
        "Explain what these files are about and how they are related"
    ));
    assert!(requires_project_scope(
        "Compare the documents and summarize differences"
    ));
}

#[test]
fn keeps_single_document_queries_focused() {
    assert!(!requires_project_scope(
        "What does slide 8 say about the model?"
    ));
    assert!(!requires_project_scope(
        "Summarize this document in five bullets"
    ));
}
