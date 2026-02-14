use tauri::State;

use crate::{
    core::{
        errors::{AppError, AppResult},
        types::{Provider, SetProviderKeyResponse},
    },
    security::keyring,
    AppState,
};

#[tauri::command]
pub async fn set_provider_key(
    _state: State<'_, AppState>,
    provider: Provider,
    api_key: String,
) -> AppResult<SetProviderKeyResponse> {
    if api_key.trim().is_empty() {
        return Err(AppError::InvalidInput("api key cannot be empty".to_string()));
    }
    keyring::set_provider_key(provider, &api_key)?;
    Ok(SetProviderKeyResponse { stored: true })
}
