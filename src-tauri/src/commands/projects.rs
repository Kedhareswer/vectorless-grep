use tauri::State;
use uuid::Uuid;

use crate::{
    core::{
        errors::{AppError, AppResult},
        types::{CreateProjectResponse, DeleteProjectResponse, ListProjectsResponse, RenameProjectResponse},
    },
    db::repositories::projects,
    AppState,
};

fn normalized_name(name: &str) -> AppResult<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput("project name cannot be empty".to_string()));
    }
    Ok(trimmed.to_string())
}

#[tauri::command]
pub async fn list_projects(state: State<'_, AppState>) -> AppResult<ListProjectsResponse> {
    let projects = projects::list_projects(state.db.pool()).await?;
    Ok(ListProjectsResponse { projects })
}

#[tauri::command]
pub async fn create_project(
    state: State<'_, AppState>,
    name: String,
) -> AppResult<CreateProjectResponse> {
    let id = Uuid::new_v4().to_string();
    let normalized = normalized_name(&name)?;
    let project = projects::create_project(state.db.pool(), &id, &normalized).await?;
    Ok(CreateProjectResponse { project })
}

#[tauri::command]
pub async fn rename_project(
    state: State<'_, AppState>,
    project_id: String,
    name: String,
) -> AppResult<RenameProjectResponse> {
    let normalized = normalized_name(&name)?;
    let project = projects::rename_project(state.db.pool(), &project_id, &normalized).await?;
    Ok(RenameProjectResponse { project })
}

#[tauri::command]
pub async fn delete_project(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<DeleteProjectResponse> {
    let deleted = projects::delete_project(state.db.pool(), &project_id).await?;
    Ok(DeleteProjectResponse { deleted })
}
