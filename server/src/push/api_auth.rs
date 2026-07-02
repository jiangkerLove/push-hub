use axum::http::HeaderMap;

use crate::models::PushApp;
use crate::AppError;

pub fn generate_push_api_key() -> String {
    format!("phk_{}", uuid::Uuid::new_v4().simple())
}

pub fn extract_push_api_key(headers: &HeaderMap) -> Option<String> {
    if let Some(value) = headers
        .get("x-push-hub-api-key")
        .and_then(|v| v.to_str().ok())
    {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}

pub fn verify_push_api_key(app: &PushApp, provided: &str) -> Result<(), AppError> {
    if provided.trim().is_empty() {
        return Err(AppError::Unauthorized(
            "缺少 Push API Key，请在请求头携带 Authorization: Bearer <push_api_key>".into(),
        ));
    }
    if provided != app.push_api_key {
        return Err(AppError::Unauthorized("Push API Key 无效".into()));
    }
    Ok(())
}
