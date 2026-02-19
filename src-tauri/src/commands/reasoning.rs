use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

use crate::{
    core::{
        errors::{AppError, AppResult},
        types::{Provider, ReasoningCompleteEvent, ReasoningErrorEvent, RunReasoningQueryResponse},
    },
    db::repositories::reasoning,
    reasoner::query_scope::requires_project_scope,
    security::keyring,
    AppState,
};

#[tauri::command]
pub async fn run_reasoning_query(
    app: AppHandle,
    state: State<'_, AppState>,
    project_id: String,
    query: String,
    max_steps: Option<i64>,
    focus_document_id: Option<String>,
) -> AppResult<RunReasoningQueryResponse> {
    if query.trim().is_empty() {
        return Err(AppError::InvalidInput("query cannot be empty".to_string()));
    }

    let run_id = Uuid::new_v4().to_string();
    let api_key = keyring::get_provider_key(Provider::Gemini)?;
    let effective_focus_document_id = if requires_project_scope(&query) {
        None
    } else {
        focus_document_id.clone()
    };
    let db = state.db.clone();
    let executor = state.executor.clone();
    let run_id_for_task = run_id.clone();
    let project_id_for_task = project_id.clone();
    let focus_document_id_for_task = effective_focus_document_id.clone();
    let query_for_task = query.clone();
    let app_for_task = app.clone();

    tauri::async_runtime::spawn(async move {
        let outcome = executor
            .run(
                &db,
                &project_id_for_task,
                focus_document_id_for_task.as_deref(),
                run_id_for_task.clone(),
                &query_for_task,
                max_steps.map(|value| value.max(1) as usize),
                &api_key,
                |step_event| {
                    let _ = app_for_task.emit("reasoning/step", step_event);
                },
            )
            .await;

        match outcome {
            Ok(result) => {
                let _ = app_for_task.emit(
                    "reasoning/complete",
                    ReasoningCompleteEvent {
                        run_id: result.run_id,
                        answer_id: result.answer_id,
                        final_confidence: result.final_confidence,
                        total_latency_ms: result.total_latency_ms,
                        token_usage: result.token_usage,
                        cost_usd: result.cost_usd,
                    },
                );
            }
            Err(err) => {
                let _ = reasoning::fail_run(db.pool(), &run_id_for_task).await;
                let _ = app_for_task.emit(
                    "reasoning/error",
                    ReasoningErrorEvent {
                        run_id: run_id_for_task,
                        code: err.code().to_string(),
                        message: err.to_string(),
                        retryable: err.retryable(),
                    },
                );
            }
        }
    });

    Ok(RunReasoningQueryResponse {
        run_id,
        status: "started".to_string(),
    })
}

#[tauri::command]
pub async fn get_run(state: State<'_, AppState>, run_id: String) -> AppResult<crate::core::types::GetRunResponse> {
    reasoning::get_run(state.db.pool(), &run_id).await
}
