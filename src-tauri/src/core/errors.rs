use serde::ser::SerializeStruct;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("database error: {0}")]
    Database(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("sidecar error: {0}")]
    Sidecar(String),
    #[error("provider auth failed")]
    ProviderAuth,
    #[error("provider rate limited")]
    ProviderRateLimited,
    #[error("provider timeout")]
    ProviderTimeout,
    #[error("provider invalid response: {0}")]
    ProviderInvalidResponse(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("quality gate failed: {0}")]
    QualityGateFailed(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidInput(_) => "INVALID_INPUT",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Io(_) => "IO_ERROR",
            Self::Sidecar(_) => "SIDECAR_ERROR",
            Self::ProviderAuth => "PROVIDER_AUTH",
            Self::ProviderRateLimited => "PROVIDER_RATE_LIMITED",
            Self::ProviderTimeout => "PROVIDER_TIMEOUT",
            Self::ProviderInvalidResponse(_) => "PROVIDER_INVALID_RESPONSE",
            Self::Network(_) => "NETWORK_ERROR",
            Self::QualityGateFailed(_) => "QUALITY_GATE_FAILED",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    pub fn retryable(&self) -> bool {
        matches!(
            self,
            Self::ProviderRateLimited | Self::ProviderTimeout | Self::Network(_)
        )
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.to_string())
    }
}

impl From<sqlx::migrate::MigrateError> for AppError {
    fn from(value: sqlx::migrate::MigrateError) -> Self {
        Self::Database(value.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::InvalidInput(value.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
