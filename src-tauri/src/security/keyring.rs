use crate::core::{
    errors::{AppError, AppResult},
    types::Provider,
};

fn username_for_provider(provider: &Provider) -> &'static str {
    match provider {
        Provider::Gemini => "gemini",
    }
}

pub fn set_provider_key(provider: Provider, api_key: &str) -> AppResult<()> {
    let entry = keyring::Entry::new("vectorless", username_for_provider(&provider))
        .map_err(|err| AppError::Internal(err.to_string()))?;
    entry
        .set_password(api_key)
        .map_err(|err| AppError::Internal(err.to_string()))
}

pub fn get_provider_key(provider: Provider) -> AppResult<String> {
    let entry = keyring::Entry::new("vectorless", username_for_provider(&provider))
        .map_err(|err| AppError::Internal(err.to_string()))?;
    entry
        .get_password()
        .map_err(|_err| AppError::ProviderAuth)
}
