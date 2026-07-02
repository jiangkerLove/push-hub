use std::time::{Duration, Instant};

use async_trait::async_trait;
use md5;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::models::{
    click_params_object, ClickAction, ClickActionType, RenderedNotification,
};
use crate::push::{ProviderSendResult, PushProvider};
use crate::AppError;
use crate::AppResult;

const VIVO_AUTH_URL: &str = "https://api-push.vivo.com.cn/message/auth";
const VIVO_SEND_URL: &str = "https://api-push.vivo.com.cn/message/send";

const VIVO_SKIP_APP_HOME: i64 = 1;
const VIVO_SKIP_URL: i64 = 2;
const VIVO_SKIP_ACTIVITY: i64 = 4;

pub struct VivoPushProvider {
    client: Client,
    app_id: i64,
    app_key: String,
    app_secret: String,
    token_cache: Mutex<Option<CachedToken>>,
}

struct CachedToken {
    auth_token: String,
    expires_at: Instant,
}

impl VivoPushProvider {
    pub fn new(app_id: String, app_key: String, app_secret: String) -> AppResult<Self> {
        let app_id = app_id.trim().parse::<i64>().map_err(|_| {
            AppError::BadRequest(format!("vivo_app_id must be numeric, got: {app_id}"))
        })?;
        Ok(Self {
            client: Client::new(),
            app_id,
            app_key,
            app_secret,
            token_cache: Mutex::new(None),
        })
    }

    async fn auth_token(&self) -> AppResult<String> {
        {
            let cache = self.token_cache.lock().await;
            if let Some(entry) = cache.as_ref() {
                if Instant::now() < entry.expires_at {
                    return Ok(entry.auth_token.clone());
                }
            }
        }

        let timestamp = chrono::Utc::now().timestamp_millis();
        let sign = compute_sign(self.app_id, &self.app_key, timestamp, &self.app_secret);
        let body = json!({
            "appId": self.app_id,
            "appKey": self.app_key,
            "timestamp": timestamp,
            "sign": sign,
        });

        let response = self
            .client
            .post(VIVO_AUTH_URL)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        let text = response.text().await?;
        let parsed: VivoAuthResponse = serde_json::from_str(&text).map_err(|err| {
            AppError::Push(format!(
                "invalid vivo auth response (status={status}): {err}; body={text}"
            ))
        })?;

        if parsed.result != 0 {
            return Err(AppError::Push(format!(
                "vivo auth failed result={}: {}",
                parsed.result,
                parsed.desc.unwrap_or_else(|| "unknown error".into())
            )));
        }

        let auth_token = parsed.auth_token.ok_or_else(|| {
            AppError::Push(format!(
                "vivo auth response missing authToken; body={text}"
            ))
        })?;

        *self.token_cache.lock().await = Some(CachedToken {
            auth_token: auth_token.clone(),
            expires_at: Instant::now() + Duration::from_secs(3600),
        });

        Ok(auth_token)
    }

    pub async fn validate_credentials(&self) -> AppResult<()> {
        self.auth_token().await.map(|_| ())
    }
}

#[async_trait]
impl PushProvider for VivoPushProvider {
    fn platform(&self) -> &'static str {
        "vivo"
    }

    async fn send(
        &self,
        push_tokens: &[String],
        notification: &RenderedNotification,
    ) -> AppResult<ProviderSendResult> {
        if push_tokens.is_empty() {
            return Err(AppError::BadRequest("push_tokens cannot be empty".into()));
        }

        let category = notification
            .channels
            .vivo_category()
            .map(|c| c.trim().to_uppercase())
            .filter(|c| !c.is_empty())
            .ok_or_else(|| {
                AppError::BadRequest(
                    "template channels.vivo.category is required for vivo push".into(),
                )
            })?;
        let classification = vivo_classification(&category);

        let auth_token = self.auth_token().await?;
        let (skip_type, skip_content) = build_skip(&notification.click_action);
        let payload = if notification.payload.is_null() {
            None
        } else {
            Some(notification.payload.to_string())
        };
        let client_custom_map =
            build_client_custom_map(&notification.click_action, payload.as_deref());

        let mut success_count = 0usize;
        let mut failure_count = 0usize;
        let mut last_task_id = None;

        for reg_id in push_tokens {
            let mut body = json!({
                "appId": self.app_id,
                "regId": reg_id,
                "notifyType": 4,
                "title": notification.title,
                "content": notification.body,
                "timeToLive": 86400,
                "skipType": skip_type,
                "networkType": -1,
                "requestId": Uuid::new_v4().to_string(),
                // classification：0 运营消息 / 1 系统消息，与 category 一一对应
                "classification": classification,
                "category": category.clone(),
            });
            if let Some(content) = skip_content.as_ref() {
                body["skipContent"] = Value::String(content.clone());
            }
            if let Some(custom) = client_custom_map.as_ref() {
                body["clientCustomMap"] = custom.clone();
            }
            if let Some(notify_id) = vivo_notify_id(notification.notify_id) {
                body["notifyId"] = json!(notify_id);
            }

            let response = self
                .client
                .post(VIVO_SEND_URL)
                .header("authToken", auth_token.as_str())
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await?;

            let status = response.status();
            let text = response.text().await?;
            let parsed: VivoSendResponse = serde_json::from_str(&text).map_err(|err| {
                AppError::Push(format!(
                    "invalid vivo send response (status={status}): {err}; body={text}"
                ))
            })?;

            if parsed.result == 0 {
                success_count += 1;
                last_task_id = parsed.task_id.clone();
            } else {
                failure_count += 1;
                tracing::warn!(
                    reg_id = %reg_id,
                    result = parsed.result,
                    desc = ?parsed.desc,
                    "vivo send failed for token"
                );
            }
        }

        if success_count == 0 {
            return Err(AppError::Push(format!(
                "vivo send failed for all {} tokens",
                push_tokens.len()
            )));
        }

        Ok(ProviderSendResult {
            success_count,
            failure_count,
            message_id: last_task_id,
            outbox_ids: vec![],
            ws_delivered: 0,
        })
    }
}

fn vivo_notify_id(notify_id: Option<i32>) -> Option<i32> {
    match notify_id {
        None | Some(0) => None,
        Some(id) => Some(id),
    }
}

fn compute_sign(app_id: i64, app_key: &str, timestamp: i64, app_secret: &str) -> String {
    let raw = format!("{app_id}{app_key}{timestamp}{app_secret}");
    format!("{:x}", md5::compute(raw.as_bytes()))
}

/// vivo 一级分类：0 = 运营消息，1 = 系统消息。须与二级分类 category 对应。
fn vivo_classification(category: &str) -> i64 {
    match category {
        "IM" | "ACCOUNT" | "TODO" | "DEVICE_REMINDER" | "ORDER" | "SUBSCRIPTION" => 1,
        "NEWS" | "CONTENT" | "MARKETING" | "SOCIAL" => 0,
        // 自定义申请的分类默认按系统消息
        _ => 1,
    }
}

fn build_skip(action: &ClickAction) -> (i64, Option<String>) {
    match action.r#type {
        ClickActionType::OpenApp => (VIVO_SKIP_APP_HOME, None),
        ClickActionType::OpenWeb => {
            let url = action
                .url_str()
                .expect("validated open_web requires url");
            (VIVO_SKIP_URL, Some(url.to_string()))
        }
        ClickActionType::OpenPage => {
            let activity = action
                .activity_class()
                .expect("validated open_page requires FQCN activity");
            (VIVO_SKIP_ACTIVITY, Some(activity.to_string()))
        }
    }
}

fn build_client_custom_map(action: &ClickAction, payload: Option<&str>) -> Option<Value> {
    let mut map = Map::new();
    if let Value::Object(params) = click_params_object(&action.params) {
        for (key, value) in params {
            map.insert(key, value);
        }
    }
    if let Some(payload) = payload.filter(|p| !p.is_empty()) {
        map.insert("payload".into(), Value::String(payload.to_string()));
    }
    if map.is_empty() {
        None
    } else {
        Some(Value::Object(map))
    }
}

#[derive(Debug, Deserialize)]
struct VivoAuthResponse {
    result: i64,
    desc: Option<String>,
    #[serde(rename = "authToken")]
    auth_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VivoSendResponse {
    result: i64,
    desc: Option<String>,
    #[serde(rename = "taskId")]
    task_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_auth_response_uses_camel_case_auth_token() {
        let body = r#"{"result":0,"desc":"请求成功","authToken":"token-abc"}"#;
        let parsed: VivoAuthResponse = serde_json::from_str(body).unwrap();
        assert_eq!(parsed.auth_token.as_deref(), Some("token-abc"));
    }

    #[test]
    fn parse_send_response_uses_camel_case_task_id() {
        let body = r#"{"result":0,"desc":"请求成功","taskId":"121397329"}"#;
        let parsed: VivoSendResponse = serde_json::from_str(body).unwrap();
        assert_eq!(parsed.task_id.as_deref(), Some("121397329"));
    }

    #[test]
    fn compute_sign_uses_md5_lowercase_hex() {
        let sign = compute_sign(10004, "app-key", 1501484120000, "secret");
        assert_eq!(sign.len(), 32);
        assert!(sign.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn classification_matches_builtin_categories() {
        assert_eq!(vivo_classification("IM"), 1);
        assert_eq!(vivo_classification("ORDER"), 1);
        assert_eq!(vivo_classification("MARKETING"), 0);
        assert_eq!(vivo_classification("SOCIAL"), 0);
    }
}
