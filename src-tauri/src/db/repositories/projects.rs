use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};

use crate::core::{
    errors::{AppError, AppResult},
    types::ProjectSummary,
};

fn parse_timestamp(value: String) -> AppResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&value)
        .map(|v| v.with_timezone(&Utc))
        .map_err(|err| AppError::Database(format!("invalid timestamp {value}: {err}")))
}

pub async fn list_projects(pool: &SqlitePool) -> AppResult<Vec<ProjectSummary>> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, created_at, updated_at
        FROM projects
        ORDER BY created_at ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(map_project_summary).collect()
}

pub async fn create_project(pool: &SqlitePool, id: &str, name: &str) -> AppResult<ProjectSummary> {
    sqlx::query(
        r#"
        INSERT INTO projects (id, name)
        VALUES (?1, ?2)
        "#,
    )
    .bind(id)
    .bind(name)
    .execute(pool)
    .await?;
    get_project(pool, id).await
}

pub async fn rename_project(pool: &SqlitePool, id: &str, name: &str) -> AppResult<ProjectSummary> {
    let affected = sqlx::query(
        r#"
        UPDATE projects
        SET name = ?2,
            updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .bind(name)
    .execute(pool)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound(format!("project {id}")));
    }
    get_project(pool, id).await
}

pub async fn delete_project(pool: &SqlitePool, id: &str) -> AppResult<bool> {
    let affected = sqlx::query("DELETE FROM projects WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(affected > 0)
}

pub async fn get_project(pool: &SqlitePool, id: &str) -> AppResult<ProjectSummary> {
    let row = sqlx::query(
        r#"
        SELECT id, name, created_at, updated_at
        FROM projects
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("project {id}")))?;

    map_project_summary(row)
}

fn map_project_summary(row: sqlx::sqlite::SqliteRow) -> AppResult<ProjectSummary> {
    let created_at: String = row.try_get("created_at")?;
    let updated_at: String = row.try_get("updated_at")?;
    Ok(ProjectSummary {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        created_at: parse_timestamp(created_at)?,
        updated_at: parse_timestamp(updated_at)?,
    })
}
