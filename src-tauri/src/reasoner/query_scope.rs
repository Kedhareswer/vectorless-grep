const RELATION_HINTS: &[&str] = &[
    "related",
    "relationship",
    "relationships",
    "compare",
    "comparison",
    "differences",
    "similarities",
    "across",
    "between",
    "connect",
    "overlap",
    "fit together",
    "how they",
];

const MULTI_DOC_HINTS: &[&str] = &[
    "files",
    "documents",
    "docs",
    "papers",
    "slides",
    "presentations",
    "sources",
    "these files",
    "these documents",
    "all files",
    "all documents",
];

const SINGLE_DOC_HINTS: &[&str] = &[
    "this file",
    "this document",
    "this slide",
    "slide ",
    "page ",
    "section ",
];

pub fn requires_project_scope(query: &str) -> bool {
    let normalized = format!(" {} ", query.to_ascii_lowercase());
    let has_relation_hint = RELATION_HINTS.iter().any(|hint| normalized.contains(hint));
    let has_multi_doc_hint = MULTI_DOC_HINTS.iter().any(|hint| normalized.contains(hint));
    let has_single_doc_hint = SINGLE_DOC_HINTS
        .iter()
        .any(|hint| normalized.contains(hint));
    let has_plural_pronoun = normalized.contains(" they ") || normalized.contains(" them ");

    if has_multi_doc_hint && (has_relation_hint || has_plural_pronoun) {
        return true;
    }

    if normalized.contains("across documents") || normalized.contains("across files") {
        return true;
    }

    if has_relation_hint && has_plural_pronoun {
        return true;
    }

    has_relation_hint && has_multi_doc_hint && !has_single_doc_hint
}
