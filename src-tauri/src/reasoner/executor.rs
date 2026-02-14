use std::{collections::HashSet, time::Instant};

use serde_json::Value;

use crate::{
    core::{
        errors::{AppError, AppResult},
        types::ReasoningStepEvent,
    },
    db::{
        repositories::{
            documents,
            reasoning::{self, NewStep},
        },
        Database,
    },
    providers::gemini::GeminiClient,
    reasoner::{
        planner::{Planner, PlannerConfig, PlannerDecision, PlannerInput, StepType},
        prompts::synthesis_prompt,
    },
};

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub run_id: String,
    pub answer_id: String,
    pub final_confidence: f64,
    pub total_latency_ms: i64,
    pub token_usage: Value,
    pub cost_usd: f64,
}

#[derive(Clone)]
pub struct ReasoningExecutor {
    planner: Planner,
    gemini: GeminiClient,
}

impl ReasoningExecutor {
    pub fn new(gemini: GeminiClient) -> Self {
        Self {
            planner: Planner::new(PlannerConfig::default()),
            gemini,
        }
    }

    pub async fn run<F>(
        &self,
        db: &Database,
        project_id: &str,
        focus_document_id: Option<&str>,
        run_id: String,
        query: &str,
        max_steps: Option<usize>,
        api_key: &str,
        mut on_step: F,
    ) -> AppResult<ExecutionResult>
    where
        F: FnMut(ReasoningStepEvent) + Send,
    {
        reasoning::create_run(db.pool(), &run_id, project_id, focus_document_id, query).await?;

        let started = Instant::now();
        let max_steps = max_steps.unwrap_or(6).max(2);
        let mut step_count: usize = 0;
        let mut backtrack_count: usize = 0;
        let mut explored_sections: Vec<String> = vec![];
        let mut confidence: Option<f64> = None;
        let mut evidence_ids: Vec<String> = vec![];
        let mut evidence_snippets: Vec<String> = vec![];
        let mut answer_markdown = String::new();
        let mut token_usage = serde_json::json!({});
        let mut cost_usd = 0.0_f64;

        loop {
            let plan = self.planner.next_steps(&PlannerInput {
                query: query.to_string(),
                last_confidence: confidence,
                explored_sections: explored_sections.clone(),
                has_evidence: !evidence_ids.is_empty(),
                step_count,
                backtrack_count,
            });

            if matches!(plan.decision, PlannerDecision::Stop) {
                break;
            }
            if matches!(plan.decision, PlannerDecision::Backtrack) {
                backtrack_count += 1;
            }

            for planned in plan.steps {
                if step_count >= max_steps {
                    break;
                }
                step_count += 1;

                let step_started = Instant::now();
                let (thought, action, observation, node_refs, local_confidence) = match planned.step_type
                {
                    StepType::ScanRoot => {
                        let nodes = scope_nodes(db, project_id, focus_document_id, 2).await?;
                        let observed = format!("Scanned {} top-level nodes", nodes.len());
                        let refs = nodes.iter().take(3).map(|node| node.id.clone()).collect::<Vec<_>>();
                        (
                            "Need to establish broad candidate scope from document root".to_string(),
                            "Scan_Root()".to_string(),
                            observed,
                            refs,
                            0.25,
                        )
                    }
                    StepType::SelectSections => {
                        let candidates =
                            pick_candidates(db, project_id, focus_document_id, query, 6).await?;
                        explored_sections = candidates
                            .iter()
                            .map(|node| node.title.clone())
                            .filter(|title| !title.is_empty())
                            .take(6)
                            .collect();
                        let refs = candidates.iter().map(|node| node.id.clone()).collect::<Vec<_>>();
                        (
                            "Selecting sections that semantically align with query intent".to_string(),
                            "Select_Sections()".to_string(),
                            format!("Selected {} candidate nodes", refs.len()),
                            refs,
                            0.45,
                        )
                    }
                    StepType::DrillDown => {
                        let candidates =
                            pick_candidates(db, project_id, focus_document_id, query, 12).await?;
                        let refs = candidates.iter().map(|node| node.id.clone()).collect::<Vec<_>>();
                        (
                            "Drilling down into subsection-level detail".to_string(),
                            "Drill_Down()".to_string(),
                            format!("Focused on {} atomic nodes", refs.len()),
                            refs,
                            0.58,
                        )
                    }
                    StepType::ExtractEvidence => {
                        let candidates =
                            pick_candidates(db, project_id, focus_document_id, query, 8).await?;
                        evidence_ids = candidates.iter().map(|node| node.id.clone()).collect();
                        evidence_snippets = candidates
                            .iter()
                            .map(|node| {
                                let mut text = node.text.clone();
                                if text.len() > 500 {
                                    text.truncate(500);
                                }
                                format!(
                                    "[doc:{} node:{}] {} :: {}",
                                    node.document_id,
                                    node.id,
                                    node.title,
                                    text.replace('\n', " ")
                                )
                            })
                            .collect();
                        (
                            "Extracting evidence claims and table rows from selected nodes".to_string(),
                            "Extract_Evidence()".to_string(),
                            format!("Captured {} evidence snippets", evidence_snippets.len()),
                            evidence_ids.clone(),
                            0.72,
                        )
                    }
                    StepType::Synthesize => {
                        if evidence_snippets.is_empty() {
                            return Err(AppError::NotFound(
                                "no evidence nodes found for query".to_string(),
                            ));
                        }
                        let prompt = synthesis_prompt(query, &evidence_snippets);
                        let output = self.gemini.generate_answer(api_key, &prompt).await?;
                        answer_markdown = output.answer.answer_markdown.clone();
                        token_usage = output.token_usage.clone();
                        cost_usd = output.estimated_cost_usd;
                        let references = if output.answer.citations.is_empty() {
                            evidence_ids.clone()
                        } else {
                            output.answer.citations.clone()
                        };
                        (
                            "Synthesizing answer from grounded evidence using Gemini".to_string(),
                            "Synthesize()".to_string(),
                            "Generated grounded answer draft".to_string(),
                            references,
                            output.answer.confidence,
                        )
                    }
                    StepType::SelfCheck => {
                        let has_refs = !evidence_ids.is_empty();
                        let estimated = if has_refs {
                            local_confidence_for_answer(&answer_markdown, evidence_ids.len())
                        } else {
                            0.2
                        };
                        (
                            "Checking whether answer is grounded and sufficiently supported".to_string(),
                            "Self_Check()".to_string(),
                            format!("Grounded citations: {}", evidence_ids.len()),
                            evidence_ids.clone(),
                            estimated,
                        )
                    }
                };

                confidence = Some(local_confidence);
                let latency_ms = step_started.elapsed().as_millis() as i64;
                reasoning::add_step(
                    db.pool(),
                    NewStep {
                        run_id: &run_id,
                        idx: step_count as i64,
                        step_type: planned.step_type.as_str(),
                        thought: &thought,
                        action: &action,
                        observation: &observation,
                        node_refs: node_refs.clone(),
                        confidence: local_confidence,
                        latency_ms,
                    },
                )
                .await?;

                on_step(ReasoningStepEvent {
                    run_id: run_id.clone(),
                    step_index: step_count as i64,
                    step_type: planned.step_type.as_str().to_string(),
                    thought,
                    action,
                    observation,
                    node_refs: node_refs.clone(),
                    latency_ms,
                    confidence: local_confidence,
                });
            }

            let done = confidence.unwrap_or_default() >= 0.70
                || step_count >= max_steps
                || backtrack_count >= 2;
            if done {
                break;
            }
        }

        let final_confidence = confidence.unwrap_or(0.3);
        let total_latency_ms = started.elapsed().as_millis() as i64;
        let citations = dedupe_citations(evidence_ids.clone());
        let answer_id = run_id.clone();
        reasoning::complete_run(
            db.pool(),
            &run_id,
            total_latency_ms,
            token_usage.clone(),
            cost_usd,
            &answer_markdown,
            citations,
            final_confidence,
            true,
        )
        .await?;

        Ok(ExecutionResult {
            run_id,
            answer_id,
            final_confidence,
            total_latency_ms,
            token_usage,
            cost_usd,
        })
    }

}

fn dedupe_citations(citations: Vec<String>) -> Vec<String> {
    let mut set = HashSet::new();
    let mut ordered = vec![];
    for citation in citations {
        if set.insert(citation.clone()) {
            ordered.push(citation);
        }
    }
    ordered
}

fn local_confidence_for_answer(answer: &str, citation_count: usize) -> f64 {
    let citation_bonus = (citation_count as f64 * 0.06).min(0.25);
    let content_bonus = if answer.len() > 40 { 0.15 } else { 0.0 };
    (0.45 + citation_bonus + content_bonus).min(0.95)
}

async fn pick_candidates(
    db: &Database,
    project_id: &str,
    focus_document_id: Option<&str>,
    query: &str,
    limit: usize,
) -> AppResult<Vec<crate::core::types::DocNodeSummary>> {
    let nodes = scope_nodes(db, project_id, focus_document_id, 6).await?;
    let query_terms: Vec<String> = query
        .split_whitespace()
        .map(|value| value.to_ascii_lowercase())
        .collect();
    let mut scored = nodes
        .into_iter()
        .map(|node| {
            let haystack = format!("{} {}", node.title, node.text).to_ascii_lowercase();
            let mut score = 0_i64;
            for term in &query_terms {
                if haystack.contains(term) {
                    score += 3;
                }
            }
            if node.node_type == crate::core::types::NodeType::Section {
                score += 1;
            }
            (score, node)
        })
        .collect::<Vec<_>>();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.ordinal_path.cmp(&b.1.ordinal_path)));
    let selected = scored
        .into_iter()
        .filter(|(score, _)| *score > 0)
        .take(limit)
        .map(|(_, node)| node)
        .collect::<Vec<_>>();
    if selected.is_empty() {
        return scope_nodes(db, project_id, focus_document_id, 2).await;
    }
    Ok(selected)
}

async fn scope_nodes(
    db: &Database,
    project_id: &str,
    focus_document_id: Option<&str>,
    depth: i64,
) -> AppResult<Vec<crate::core::types::DocNodeSummary>> {
    if let Some(document_id) = focus_document_id {
        return documents::get_tree(db.pool(), document_id, None, depth).await;
    }
    documents::get_project_tree(db.pool(), project_id, depth).await
}
