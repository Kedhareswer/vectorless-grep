use std::collections::{HashMap, HashSet};

use crate::core::types::QualityMetrics;

pub fn evaluate_answer(
    query: &str,
    answer_markdown: &str,
    citations: &[String],
    evidence_node_ids: &[String],
    citation_document_map: &HashMap<String, String>,
    relation_query: bool,
) -> QualityMetrics {
    let grounded = !answer_markdown.trim().is_empty() && !citations.is_empty();
    let query_alignment = query_alignment_score(query, answer_markdown);

    let evidence_set: HashSet<&str> = evidence_node_ids.iter().map(String::as_str).collect();
    let valid_citations = citations
        .iter()
        .filter(|citation| evidence_set.contains(citation.as_str()))
        .count();
    let citation_coverage = if evidence_node_ids.is_empty() {
        0.0
    } else {
        (valid_citations as f64 / evidence_node_ids.len() as f64).min(1.0)
    };

    let cross_document_coverage = if relation_query {
        let docs = citations
            .iter()
            .filter_map(|citation| citation_document_map.get(citation))
            .collect::<HashSet<_>>();
        if docs.len() >= 2 {
            1.0
        } else if docs.len() == 1 {
            0.5
        } else {
            0.0
        }
    } else {
        1.0
    };

    let grounding_score = if grounded { 1.0 } else { 0.0 };
    let overall = (query_alignment * 0.4)
        + (citation_coverage * 0.25)
        + (cross_document_coverage * 0.2)
        + (grounding_score * 0.15);

    QualityMetrics {
        overall: overall.min(1.0),
        query_alignment,
        citation_coverage,
        cross_document_coverage,
        grounded,
    }
}

fn query_alignment_score(query: &str, answer: &str) -> f64 {
    let answer_lower = answer.to_ascii_lowercase();
    let terms = query
        .split(|value: char| !value.is_ascii_alphanumeric())
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| value.len() > 2)
        .filter(|value| !is_stopword(value))
        .collect::<Vec<_>>();

    if terms.is_empty() {
        return 0.0;
    }

    let matched = terms
        .iter()
        .filter(|term| answer_lower.contains(term.as_str()))
        .count();
    (matched as f64 / terms.len() as f64).min(1.0)
}

fn is_stopword(value: &str) -> bool {
    matches!(
        value,
        "the"
            | "and"
            | "for"
            | "are"
            | "how"
            | "what"
            | "with"
            | "about"
            | "that"
            | "this"
            | "these"
            | "from"
            | "into"
            | "their"
            | "they"
    )
}

#[cfg(test)]
mod tests {
    use super::evaluate_answer;
    use std::collections::HashMap;

    #[test]
    fn evaluator_scores_grounded_cross_document_relation_answer_higher() {
        let citations = vec!["n1".to_string(), "n2".to_string()];
        let evidence = vec!["n1".to_string(), "n2".to_string(), "n3".to_string()];
        let mut doc_map = HashMap::new();
        doc_map.insert("n1".to_string(), "doc-a".to_string());
        doc_map.insert("n2".to_string(), "doc-b".to_string());

        let metrics = evaluate_answer(
            "Explain what these files are about and how they are related",
            "File A describes architecture. File B describes experiments. They are related through shared U-Net components.",
            &citations,
            &evidence,
            &doc_map,
            true,
        );

        assert!(metrics.grounded);
        assert!(metrics.cross_document_coverage >= 1.0);
        assert!(metrics.overall >= 0.55);
    }

    #[test]
    fn evaluator_penalizes_ungrounded_answer() {
        let metrics = evaluate_answer(
            "What is this file about?",
            "",
            &[],
            &[],
            &HashMap::new(),
            false,
        );

        assert!(!metrics.grounded);
        assert!(metrics.overall < 0.3);
    }
}
