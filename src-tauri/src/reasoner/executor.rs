use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

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
        evaluator::evaluate_answer,
        planner::{Planner, PlannerConfig, PlannerDecision, PlannerInput, StepType},
        prompts::{planner_prompt, synthesis_prompt},
        query_scope::requires_project_scope,
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

const MIN_QUALITY_SCORE: f64 = 0.60;
const MIN_RELATION_QUALITY_SCORE: f64 = 0.70;

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
        let mut evidence_doc_map: HashMap<String, String> = HashMap::new();
        let mut answer_markdown = String::new();
        let mut token_usage = serde_json::json!({});
        let mut cost_usd = 0.0_f64;
        let mut planner_trace: Vec<Value> = vec![];

        loop {
            let planner_input = PlannerInput {
                query: query.to_string(),
                last_confidence: confidence,
                explored_sections: explored_sections.clone(),
                has_evidence: !evidence_ids.is_empty(),
                step_count,
                backtrack_count,
            };

            let plan = match self
                .gemini
                .generate_plan_step(api_key, &planner_prompt(&planner_input))
                .await
            {
                Ok(model_step) => self
                    .planner
                    .next_steps_from_model(&planner_input, &model_step)
                    .unwrap_or_else(|| self.planner.next_steps(&planner_input)),
                Err(_) => self.planner.next_steps(&planner_input),
            };

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

                reasoning::update_run_phase(db.pool(), &run_id, phase_for_step(&planned.step_type))
                    .await?;

                planner_trace.push(serde_json::json!({
                    "step": planned.step_type.as_str(),
                    "objective": planned.objective.clone(),
                    "decision": match plan.decision {
                        PlannerDecision::Continue => "continue",
                        PlannerDecision::Backtrack => "backtrack",
                        PlannerDecision::Stop => "stop",
                    }
                }));

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
                        evidence_doc_map = candidates
                            .iter()
                            .map(|node| (node.id.clone(), node.document_id.clone()))
                            .collect();
                        evidence_snippets = candidates
                            .iter()
                            .map(|node| {
                                let mut text = node.text.clone();
                                if text.len() > 500 {
                                    text.truncate(500);
                                }
                                format!(
                                    "[citation:{}] document={} path={} type={} title={} excerpt={} ",
                                    node.id,
                                    node.document_id,
                                    node.ordinal_path,
                                    node_type_name(&node.node_type),
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
                        answer_markdown = output.answer.answer_markdown.trim().to_string();
                        token_usage = output.token_usage.clone();
                        cost_usd = output.estimated_cost_usd;
                        let normalized = normalize_citations(&output.answer.citations, &evidence_ids);
                        let references = if normalized.is_empty() {
                            evidence_ids.iter().take(4).cloned().collect::<Vec<_>>()
                        } else {
                            normalized
                        };
                        if answer_markdown.is_empty() {
                            answer_markdown =
                                "I could not produce a grounded answer from the available evidence."
                                    .to_string();
                        }
                        (
                            "Synthesizing answer from grounded evidence using Gemini".to_string(),
                            "Synthesize()".to_string(),
                            format!(
                                "Generated answer draft with {} citation(s)",
                                references.len()
                            ),
                            references.clone(),
                            output.answer.confidence,
                        )
                    }
                    StepType::SelfCheck => {
                        let grounded = is_answer_grounded(&answer_markdown, &evidence_ids);
                        let estimated = if grounded {
                            local_confidence_for_answer(&answer_markdown, evidence_ids.len())
                        } else {
                            0.28
                        };
                        (
                            "Checking whether answer is grounded and sufficiently supported".to_string(),
                            "Self_Check()".to_string(),
                            format!(
                                "Grounded: {} • citations: {}",
                                grounded,
                                evidence_ids.len()
                            ),
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
        let relation_query = focus_document_id.is_none() && requires_project_scope(query);
        let quality = evaluate_answer(
            query,
            &answer_markdown,
            &citations,
            &evidence_ids,
            &evidence_doc_map,
            relation_query,
        );
        let grounded = quality.grounded && is_answer_grounded(&answer_markdown, &citations);
        let min_quality_score = if relation_query {
            MIN_RELATION_QUALITY_SCORE
        } else {
            MIN_QUALITY_SCORE
        };
        let quality_gate_passed = grounded && quality.overall >= min_quality_score;

        if !quality_gate_passed {
            return Err(AppError::QualityGateFailed(format!(
                "Insufficient answer quality ({:.0}% < {:.0}%). No answer returned; refine the question or add clearer source evidence.",
                quality.overall * 100.0,
                min_quality_score * 100.0
            )));
        }

        let final_confidence = if grounded {
            final_confidence.max(quality.overall)
        } else {
            final_confidence.min(0.45).min(quality.overall.max(0.25))
        };
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
            grounded,
            serde_json::to_value(quality).unwrap_or_else(|_| serde_json::json!({})),
            serde_json::Value::Array(planner_trace),
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
    if answer.trim().is_empty() {
        return 0.15;
    }
    let citation_bonus = (citation_count as f64 * 0.08).min(0.40);
    let content_bonus = if answer.len() > 120 { 0.20 } else { 0.10 };
    (0.15 + citation_bonus + content_bonus).min(0.92)
}

async fn pick_candidates(
    db: &Database,
    project_id: &str,
    focus_document_id: Option<&str>,
    query: &str,
    limit: usize,
) -> AppResult<Vec<crate::core::types::DocNodeSummary>> {
    let mut ranked = documents::search_project_nodes(
        db.pool(),
        project_id,
        focus_document_id,
        query,
        limit.saturating_mul(4).max(12),
    )
    .await?;

    if ranked.is_empty() {
        ranked = scope_nodes(db, project_id, focus_document_id, 2).await?;
    }

    if ranked.is_empty() {
        return Ok(vec![]);
    }

    let mut selected = Vec::new();
    let mut per_document = HashMap::<String, usize>::new();
    let max_per_document = if focus_document_id.is_some() {
        limit.max(1)
    } else {
        (limit / 2).max(2)
    };

    for node in ranked {
        if selected.len() >= limit {
            break;
        }
        let seen_for_document = per_document.get(&node.document_id).copied().unwrap_or(0);
        if focus_document_id.is_none() && seen_for_document >= max_per_document {
            continue;
        }
        per_document.insert(node.document_id.clone(), seen_for_document + 1);
        selected.push(node);
    }

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

fn is_answer_grounded(answer_markdown: &str, citations: &[String]) -> bool {
    if answer_markdown.trim().is_empty() {
        return false;
    }
    if citations.is_empty() {
        return false;
    }
    !answer_markdown
        .to_ascii_lowercase()
        .contains("could not produce a grounded answer")
}

fn normalize_citations(raw: &[String], evidence_ids: &[String]) -> Vec<String> {
    let allowed: HashSet<&str> = evidence_ids.iter().map(String::as_str).collect();
    raw.iter()
        .filter(|value| allowed.contains(value.as_str()))
        .cloned()
        .collect::<Vec<_>>()
}

fn node_type_name(node_type: &crate::core::types::NodeType) -> &'static str {
    match node_type {
        crate::core::types::NodeType::Document => "document",
        crate::core::types::NodeType::Section => "section",
        crate::core::types::NodeType::Subsection => "subsection",
        crate::core::types::NodeType::Paragraph => "paragraph",
        crate::core::types::NodeType::Claim => "claim",
        crate::core::types::NodeType::Table => "table",
        crate::core::types::NodeType::Figure => "figure",
        crate::core::types::NodeType::Equation => "equation",
        crate::core::types::NodeType::Caption => "caption",
        crate::core::types::NodeType::Reference => "reference",
        crate::core::types::NodeType::Unknown => "unknown",
    }
}

fn phase_for_step(step_type: &StepType) -> &'static str {
    match step_type {
        StepType::ScanRoot | StepType::SelectSections | StepType::DrillDown | StepType::ExtractEvidence => {
            "retrieval"
        }
        StepType::Synthesize => "synthesis",
        StepType::SelfCheck => "validation",
    }
}
