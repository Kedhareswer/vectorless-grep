use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};

use crate::core::{
    errors::{AppError, AppResult},
    types::{AnswerRecord, GetRunResponse, ReasoningRun, ReasoningStep, RunStatus},
};

#[derive(Debug, Clone)]
pub struct NewStep<'a> {
    pub run_id: &'a str,
    pub idx: i64,
    pub step_type: &'a str,
    pub thought: &'a str,
    pub action: &'a str,
    pub observation: &'a str,
    pub node_refs: Vec<String>,
    pub confidence: f64,
    pub latency_ms: i64,
}

fn parse_timestamp(value: String) -> AppResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&value)
        .map(|v| v.with_timezone(&Utc))
        .map_err(|err| AppError::Database(format!("invalid timestamp {value}: {err}")))
}

pub async fn create_run(
    pool: &SqlitePool,
    run_id: &str,
    project_id: &str,
    document_id: Option<&str>,
    query: &str,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO reasoning_runs (id, project_id, document_id, query, status)
        VALUES (?1, ?2, ?3, ?4, 'running')
        "#,
    )
    .bind(run_id)
    .bind(project_id)
    .bind(document_id)
    .bind(query)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn add_step(pool: &SqlitePool, step: NewStep<'_>) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO reasoning_steps (
          run_id, idx, step_type, thought, action, observation, node_refs_json, confidence, latency_ms
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#,
    )
    .bind(step.run_id)
    .bind(step.idx)
    .bind(step.step_type)
    .bind(step.thought)
    .bind(step.action)
    .bind(step.observation)
    .bind(
        serde_json::to_string(&step.node_refs)
            .map_err(|err: serde_json::Error| AppError::Internal(err.to_string()))?,
    )
    .bind(step.confidence)
    .bind(step.latency_ms)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn complete_run(
    pool: &SqlitePool,
    run_id: &str,
    total_latency_ms: i64,
    token_usage_json: serde_json::Value,
    cost_usd: f64,
    answer_markdown: &str,
    citations: Vec<String>,
    confidence: f64,
    grounded: bool,
) -> AppResult<()> {
    let mut tx = pool.begin().await?;
    sqlx::query(
        r#"
        UPDATE reasoning_runs
        SET status = 'completed',
            ended_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            total_latency_ms = ?2,
            token_usage_json = ?3,
            cost_usd = ?4
        WHERE id = ?1
        "#,
    )
    .bind(run_id)
    .bind(total_latency_ms)
    .bind(token_usage_json.to_string())
    .bind(cost_usd)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        INSERT OR REPLACE INTO answers (run_id, answer_markdown, citations_json, confidence, grounded)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
    )
    .bind(run_id)
    .bind(answer_markdown)
    .bind(
        serde_json::to_string(&citations)
            .map_err(|err: serde_json::Error| AppError::Internal(err.to_string()))?,
    )
    .bind(confidence)
    .bind(if grounded { 1 } else { 0 })
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn fail_run(pool: &SqlitePool, run_id: &str) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE reasoning_runs
        SET status = 'failed',
            ended_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        WHERE id = ?1
        "#,
    )
    .bind(run_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_run(pool: &SqlitePool, run_id: &str) -> AppResult<GetRunResponse> {
    let run_row = sqlx::query(
        r#"
        SELECT id, project_id, document_id, query, status, started_at, ended_at, total_latency_ms, token_usage_json, cost_usd
        FROM reasoning_runs
        WHERE id = ?1
        "#,
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("run {run_id}")))?;

    let status_raw: String = run_row.try_get("status")?;
    let started_at: String = run_row.try_get("started_at")?;
    let ended_at: Option<String> = run_row.try_get("ended_at")?;
    let token_usage_raw: String = run_row.try_get("token_usage_json")?;
    let run = ReasoningRun {
        id: run_row.try_get("id")?,
        project_id: run_row.try_get("project_id")?,
        document_id: run_row.try_get("document_id")?,
        query: run_row.try_get("query")?,
        status: match status_raw.as_str() {
            "completed" => RunStatus::Completed,
            "failed" => RunStatus::Failed,
            _ => RunStatus::Running,
        },
        started_at: parse_timestamp(started_at)?,
        ended_at: ended_at.map(parse_timestamp).transpose()?,
        total_latency_ms: run_row.try_get("total_latency_ms")?,
        token_usage_json: serde_json::from_str(&token_usage_raw)
            .unwrap_or_else(|_| serde_json::json!({})),
        cost_usd: run_row.try_get("cost_usd")?,
    };

    let step_rows = sqlx::query(
        r#"
        SELECT run_id, idx, step_type, thought, action, observation, node_refs_json, confidence, latency_ms
        FROM reasoning_steps
        WHERE run_id = ?1
        ORDER BY idx ASC
        "#,
    )
    .bind(run_id)
    .fetch_all(pool)
    .await?;

    let mut steps = Vec::with_capacity(step_rows.len());
    for row in step_rows {
        let node_refs_raw: String = row.try_get("node_refs_json")?;
        steps.push(ReasoningStep {
            run_id: row.try_get("run_id")?,
            idx: row.try_get("idx")?,
            step_type: row.try_get("step_type")?,
            thought: row.try_get("thought")?,
            action: row.try_get("action")?,
            observation: row.try_get("observation")?,
            node_refs: serde_json::from_str(&node_refs_raw).unwrap_or_else(|_| vec![]),
            confidence: row.try_get("confidence")?,
            latency_ms: row.try_get("latency_ms")?,
        });
    }

    let answer = sqlx::query(
        "SELECT run_id, answer_markdown, citations_json, confidence, grounded FROM answers WHERE run_id = ?1",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await?
    .map(|row| -> AppResult<AnswerRecord> {
        let citations_raw: String = row.try_get("citations_json")?;
        Ok(AnswerRecord {
            run_id: row.try_get("run_id")?,
            answer_markdown: row.try_get("answer_markdown")?,
            citations: serde_json::from_str(&citations_raw).unwrap_or_else(|_| vec![]),
            confidence: row.try_get("confidence")?,
            grounded: row.try_get::<i64, _>("grounded")? == 1,
        })
    })
    .transpose()?;

    Ok(GetRunResponse { run, steps, answer })
}
