use std::path::{Path, PathBuf};
use std::str::FromStr;

use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};

use crate::core::errors::{AppError, AppResult};

pub mod repositories;

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(app_data_dir: &Path) -> AppResult<Self> {
        std::fs::create_dir_all(app_data_dir)?;
        let db_path = app_data_dir.join("vectorless.sqlite");
        let connect_options = SqliteConnectOptions::from_str(&format!(
            "sqlite:{}",
            db_path.to_string_lossy().replace('\\', "/")
        ))
        .map_err(|err| AppError::Database(err.to_string()))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal);
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect_with(connect_options)
            .await?;
        sqlx::query("PRAGMA foreign_keys = ON;")
            .execute(&pool)
            .await?;
        sqlx::migrate!("./src/db/migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    pub async fn in_memory() -> AppResult<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;
        sqlx::query("PRAGMA foreign_keys = ON;")
            .execute(&pool)
            .await?;
        sqlx::migrate!("./src/db/migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

pub fn default_data_dir(base: Option<PathBuf>) -> Result<PathBuf, AppError> {
    if let Some(path) = base {
        return Ok(path);
    }
    let mut cwd = std::env::current_dir().map_err(|err| AppError::Io(err.to_string()))?;
    cwd.push(".vectorless");
    Ok(cwd)
}
