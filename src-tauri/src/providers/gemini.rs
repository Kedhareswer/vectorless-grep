use std::time::Duration;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::errors::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct GeminiClient {
    http: reqwest::Client,
    model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiAnswer {
    pub answer_markdown: String,
    pub confidence: f64,
    pub citations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiOutput {
    pub answer: GeminiAnswer,
    pub token_usage: Value,
    pub estimated_cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiPlannerStep {
    #[serde(alias = "step_type")]
    pub step_type: String,
    pub objective: String,
    #[serde(default)]
    pub reasoning: String,
    #[serde(default = "default_planner_decision")]
    pub decision: String,
}

fn default_planner_decision() -> String {
    "continue".to_string()
}

impl GeminiClient {
    pub fn new(model: impl Into<String>) -> AppResult<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|err| AppError::Network(err.to_string()))?;
        Ok(Self {
            http,
            model: model.into(),
        })
    }

    pub async fn generate_answer(&self, api_key: &str, prompt: &str) -> AppResult<GeminiOutput> {
        let endpoint = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, api_key
        );
        let payload = serde_json::json!({
            "contents": [
                {
                    "role": "user",
                    "parts": [{"text": prompt}]
                }
            ],
            "generationConfig": {
                "temperature": 0.2,
                "responseMimeType": "application/json"
            }
        });

        let response = self
            .http
            .post(endpoint)
            .json(&payload)
            .send()
            .await
            .map_err(|err| {
                if err.is_timeout() {
                    AppError::ProviderTimeout
                } else {
                    AppError::Network(err.to_string())
                }
            })?;

        match response.status() {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => return Err(AppError::ProviderAuth),
            StatusCode::TOO_MANY_REQUESTS => return Err(AppError::ProviderRateLimited),
            status if !status.is_success() => {
                let body = response.text().await.unwrap_or_default();
                return Err(AppError::ProviderInvalidResponse(format!(
                    "status {status} body {body}"
                )));
            }
            _ => {}
        }

        let body: Value = response
            .json()
            .await
            .map_err(|err| AppError::ProviderInvalidResponse(err.to_string()))?;
        let text = body
            .get("candidates")
            .and_then(Value::as_array)
            .and_then(|items: &Vec<Value>| items.first())
            .and_then(|item: &Value| item.get("content"))
            .and_then(|content: &Value| content.get("parts"))
            .and_then(Value::as_array)
            .and_then(|parts: &Vec<Value>| parts.first())
            .and_then(|part: &Value| part.get("text"))
            .and_then(Value::as_str)
            .ok_or_else(|| AppError::ProviderInvalidResponse("missing text candidate".to_string()))?;

        let parsed_json: Value = serde_json::from_str(text)
            .map_err(|err| AppError::ProviderInvalidResponse(format!("model output not JSON: {err}")))?;
        let answer_markdown = parsed_json
            .get("answer_markdown")
            .and_then(Value::as_str)
            .unwrap_or("No grounded answer could be generated.")
            .to_string();
        let confidence = parsed_json
            .get("confidence")
            .and_then(Value::as_f64)
            .unwrap_or(0.5);
        let citations = parsed_json
            .get("citations")
            .and_then(Value::as_array)
            .map(|items: &Vec<Value>| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToString::to_string)
                    .collect()
            })
            .unwrap_or_default();

        let token_usage = body
            .get("usageMetadata")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        let input_tokens = token_usage
            .get("promptTokenCount")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        let output_tokens = token_usage
            .get("candidatesTokenCount")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);

        // Light-weight estimate for surfaced telemetry in v1.
        let estimated_cost_usd = ((input_tokens * 0.0000003) + (output_tokens * 0.0000012)) as f64;

        Ok(GeminiOutput {
            answer: GeminiAnswer {
                answer_markdown,
                confidence,
                citations,
            },
            token_usage,
            estimated_cost_usd,
        })
    }

    pub async fn generate_plan_step(
        &self,
        api_key: &str,
        prompt: &str,
    ) -> AppResult<GeminiPlannerStep> {
        let endpoint = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, api_key
        );
        let payload = serde_json::json!({
            "contents": [
                {
                    "role": "user",
                    "parts": [{"text": prompt}]
                }
            ],
            "generationConfig": {
                "temperature": 0.1,
                "responseMimeType": "application/json"
            }
        });

        let response = self
            .http
            .post(endpoint)
            .json(&payload)
            .send()
            .await
            .map_err(|err| {
                if err.is_timeout() {
                    AppError::ProviderTimeout
                } else {
                    AppError::Network(err.to_string())
                }
            })?;

        match response.status() {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => return Err(AppError::ProviderAuth),
            StatusCode::TOO_MANY_REQUESTS => return Err(AppError::ProviderRateLimited),
            status if !status.is_success() => {
                let body = response.text().await.unwrap_or_default();
                return Err(AppError::ProviderInvalidResponse(format!(
                    "status {status} body {body}"
                )));
            }
            _ => {}
        }

        let body: Value = response
            .json()
            .await
            .map_err(|err| AppError::ProviderInvalidResponse(err.to_string()))?;
        let text = body
            .get("candidates")
            .and_then(Value::as_array)
            .and_then(|items: &Vec<Value>| items.first())
            .and_then(|item: &Value| item.get("content"))
            .and_then(|content: &Value| content.get("parts"))
            .and_then(Value::as_array)
            .and_then(|parts: &Vec<Value>| parts.first())
            .and_then(|part: &Value| part.get("text"))
            .and_then(Value::as_str)
            .ok_or_else(|| AppError::ProviderInvalidResponse("missing text candidate".to_string()))?;

        let parsed: GeminiPlannerStep = serde_json::from_str(text)
            .map_err(|err| AppError::ProviderInvalidResponse(format!("planner output not JSON: {err}")))?;

        if parsed.step_type.trim().is_empty() || parsed.objective.trim().is_empty() {
            return Err(AppError::ProviderInvalidResponse(
                "planner output missing required fields".to_string(),
            ));
        }

        Ok(parsed)
    }
}
