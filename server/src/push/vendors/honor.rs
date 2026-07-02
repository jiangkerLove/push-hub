use std::time::{Duration, Instant};

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::Mutex;

use crate::models::{
    build_intent_uri, ClickAction, ClickActionType, RenderedNotification,
};
use crate::push::{ProviderSendResult, PushProvider};
use crate::AppError;
use crate::AppResult;

/// 官方「获取鉴权接口」：验证凭证与发送推送均通过此地址换取 access_token。
const HONOR_OAUTH_URL: &str = "https://iam.developer.hihonor.com/auth/token";
/// 官方下行消息接口：https://developer.honor.com/cn/docs/11002/reference/downlink-message
const HONOR_PUSH_URL: &str =
    "https://push-api.cloud.honor.com/api/v1/{app_id}/sendMessage";
const HONOR_SUCCESS_CODE: &str = "80000000";

const HONOR_CLICK_INTENT: i64 = 1;
const HONOR_CLICK_URL: i64 = 2;
const HONOR_CLICK_APP: i64 = 3;
/// 荣耀无控制台通道配置；未指定模板/API 覆盖时使用。
pub const HONOR_DEFAULT_CATEGORY: &str = "NORMAL";
/// 消息缓存 TTL，与华为 Push Kit 一致；默认 1 天。
const HONOR_DEFAULT_TTL: &str = "86400s";

pub struct HonorPushProvider {
    client: Client,
    app_id: String,
    oauth_client_id: String,
    client_secret: String,
    default_package_name: String,
    token_cache: Mutex<Option<CachedToken>>,
}

/// 荣耀控制台三项独立凭证：App ID（推送 URL / 客户端）、Client ID、Client Secret（OAuth）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HonorCredentials {
    pub app_id: String,
    pub oauth_client_id: String,
    pub client_secret: String,
}

pub fn resolve_honor_credentials(
    app_id: Option<String>,
    oauth_client_id: Option<String>,
    client_secret: Option<String>,
) -> AppResult<HonorCredentials> {
    let app_id = app_id
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| {
            AppError::BadRequest("honor app_id is required (push AppId from Honor console)".into())
        })?;
    let oauth_client_id = oauth_client_id
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| {
            AppError::BadRequest(
                "honor oauth client_id is required (separate from App ID on Honor console)".into(),
            )
        })?;
    let client_secret = client_secret
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| {
            AppError::BadRequest("honor client_secret is required".into())
        })?;
    Ok(HonorCredentials {
        app_id,
        oauth_client_id,
        client_secret,
    })
}

struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

struct OAuthTokenResult {
    access_token: String,
    expires_in: Option<i64>,
}

impl HonorPushProvider {
    pub fn new(
        app_id: String,
        oauth_client_id: String,
        client_secret: String,
        default_package_name: String,
    ) -> Self {
        Self {
            client: Client::new(),
            app_id,
            oauth_client_id,
            client_secret,
            default_package_name,
            token_cache: Mutex::new(None),
        }
    }

    pub fn from_credentials(credentials: HonorCredentials, default_package_name: String) -> Self {
        Self::new(
            credentials.app_id,
            credentials.oauth_client_id,
            credentials.client_secret,
            default_package_name,
        )
    }

    async fn access_token(&self) -> AppResult<String> {
        {
            let cache = self.token_cache.lock().await;
            if let Some(entry) = cache.as_ref() {
                if Instant::now() < entry.expires_at {
                    return Ok(entry.access_token.clone());
                }
            }
        }

        let token = self.request_oauth_token().await?;
        let ttl = token.expires_in.unwrap_or(3600).max(60);
        let expires_at = Instant::now() + Duration::from_secs((ttl - 60) as u64);
        *self.token_cache.lock().await = Some(CachedToken {
            access_token: token.access_token.clone(),
            expires_at,
        });
        Ok(token.access_token)
    }

    async fn request_oauth_token(&self) -> AppResult<OAuthTokenResult> {
        let response = self
            .client
            .post(HONOR_OAUTH_URL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", self.oauth_client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
            ])
            .send()
            .await?;

        let body = response.text().await?;
        let parsed: HonorOAuthResponse = serde_json::from_str(&body).map_err(|_| {
            AppError::Push("荣耀鉴权响应异常，请稍后重试".into())
        })?;

        let Some(access_token) = parsed.access_token else {
            return Err(AppError::Push(format_oauth_failure(&parsed)));
        };

        Ok(OAuthTokenResult {
            access_token,
            expires_in: parsed.expires_in,
        })
    }

    pub async fn validate_credentials(&self) -> AppResult<()> {
        self.access_token().await.map(|_| ())
    }
}

#[async_trait]
impl PushProvider for HonorPushProvider {
    fn platform(&self) -> &'static str {
        "honor"
    }

    async fn send(
        &self,
        push_tokens: &[String],
        notification: &RenderedNotification,
    ) -> AppResult<ProviderSendResult> {
        if push_tokens.is_empty() {
            return Err(AppError::BadRequest("push_tokens cannot be empty".into()));
        }

        let package_name = if notification.package_name.is_empty() {
            self.default_package_name.clone()
        } else {
            notification.package_name.clone()
        };

        let category = notification
            .channels
            .honor_category()
            .map(|c| c.trim().to_uppercase())
            .filter(|c| !c.is_empty())
            .unwrap_or_else(|| HONOR_DEFAULT_CATEGORY.to_string());

        let access_token = self.access_token().await?;
        let url = HONOR_PUSH_URL.replace("{app_id}", &self.app_id);
        let when = honor_notification_when(chrono::Utc::now());
        let android_notification = build_android_notification(
            &notification.title,
            &notification.body,
            &notification.click_action,
            &package_name,
            &category,
            notification.notify_id,
            &when,
        );

        let payload = if notification.payload.is_null() {
            String::new()
        } else {
            notification.payload.to_string()
        };

        let message = if notification.delivery_mode.is_pass_through() {
            build_data_message(push_tokens, &payload, &category)
        } else {
            build_notification_message(
                push_tokens,
                notification,
                &android_notification,
                &payload,
            )
        };

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {access_token}"))
            .header("timestamp", honor_request_timestamp())
            .header("Content-Type", "application/json; charset=UTF-8")
            .json(&message)
            .send()
            .await?;

        let text = response.text().await?;
        let result: HonorSendResponse = serde_json::from_str(&text).map_err(|_| {
            AppError::Push("荣耀推送响应异常，请稍后重试".into())
        })?;

        if !result.is_success() {
            return Err(AppError::Push(format_send_failure(&result)));
        }

        Ok(ProviderSendResult {
            success_count: push_tokens.len(),
            failure_count: 0,
            message_id: result.request_id(),
            outbox_ids: vec![],
            ws_delivered: 0,
        })
    }
}

fn build_notification_message(
    push_tokens: &[String],
    notification: &RenderedNotification,
    android_notification: &Value,
    payload: &str,
) -> Value {
    let mut message = json!({
        "notification": {
            "title": notification.title,
            "body": notification.body,
        },
        "android": {
            "ttl": HONOR_DEFAULT_TTL,
            "notification": android_notification,
        },
        "token": push_tokens,
    });
    if !payload.is_empty() {
        message["data"] = Value::String(payload.to_string());
    }
    message
}

fn build_data_message(push_tokens: &[String], payload: &str, category: &str) -> Value {
    let mut message = json!({
        "token": push_tokens,
        "android": {
            "category": category,
            "ttl": HONOR_DEFAULT_TTL,
        },
    });
    if !payload.is_empty() {
        message["data"] = Value::String(payload.to_string());
    }
    message
}

fn build_android_notification(
    title: &str,
    body: &str,
    action: &ClickAction,
    package_name: &str,
    importance: &str,
    notify_id: Option<i32>,
    when: &str,
) -> Value {
    let mut notification = json!({
        "title": title,
        "body": body,
        "clickAction": build_click_action(action, package_name),
        "importance": importance,
        "style": 0,
        "when": when,
    });
    if let Some(id) = notify_id {
        notification["notifyId"] = json!(id);
    }
    notification
}

/// 荣耀 `android.notification.when`：UTC 时间戳，纳秒精度（官方示例 `2014-10-02T15:01:23.045123456Z`）。
fn honor_notification_when(now: chrono::DateTime<chrono::Utc>) -> String {
    format!(
        "{}.{:09}Z",
        now.format("%Y-%m-%dT%H:%M:%S"),
        now.timestamp_subsec_nanos()
    )
}

/// 荣耀下行消息请求头 `timestamp`：当前 UTC 毫秒时间戳（必填）。
fn honor_request_timestamp() -> String {
    chrono::Utc::now().timestamp_millis().to_string()
}

fn build_click_action(action: &ClickAction, package_name: &str) -> Value {
    match action.r#type {
        ClickActionType::OpenApp => json!({ "type": HONOR_CLICK_APP }),
        ClickActionType::OpenPage => {
            let activity = action
                .activity_class()
                .expect("validated open_page requires FQCN activity");
            json!({
                "type": HONOR_CLICK_INTENT,
                "intent": build_intent_uri(package_name, activity, &action.params),
            })
        }
        ClickActionType::OpenWeb => {
            let url = action
                .url_str()
                .expect("validated open_web requires url");
            json!({
                "type": HONOR_CLICK_URL,
                "url": url,
            })
        }
    }
}

fn format_send_failure(result: &HonorSendResponse) -> String {
    let msg = result
        .message_text()
        .unwrap_or("未知错误");
    let code = result.code_text().unwrap_or_else(|| "unknown".into());
    format!("荣耀推送失败（{}）：{}", code, msg)
}

fn deserialize_optional_string_or_number_required<'de, D>(
    deserializer: D,
) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    Ok(stringify_json_scalar(value))
}

fn deserialize_optional_string_or_number<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    Ok(value.map(stringify_json_scalar))
}

fn format_oauth_failure(parsed: &HonorOAuthResponse) -> String {
    let code = parsed.error.as_deref().unwrap_or_default();
    let desc = parsed.error_description.as_deref().unwrap_or_default();

    if matches!(
        code,
        "invalid_client" | "unauthorized_client" | "invalid_grant"
    ) || desc.contains("Invalid client")
    {
        return "Client ID 或 Client Secret 不正确，请从「推送服务 → 应用查看」复制（勿填 App Secret）".into();
    }

    if !desc.is_empty() {
        return format!("荣耀鉴权失败：{}", desc);
    }

    if !code.is_empty() {
        return format!("荣耀鉴权失败（{}）", code);
    }

    "荣耀鉴权失败，请检查凭证后重试".into()
}

fn stringify_json_scalar(value: Value) -> String {
    match value {
        Value::String(text) => text,
        Value::Number(number) => number.to_string(),
        other => other.to_string(),
    }
}

#[derive(Debug, Deserialize)]
struct HonorOAuthResponse {
    access_token: Option<String>,
    expires_in: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_string_or_number")]
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HonorSendData {
    #[serde(default, alias = "requestId")]
    request_id: Option<String>,
    #[serde(default, alias = "sendResult")]
    send_result: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct HonorSendResponse {
    #[serde(default, deserialize_with = "deserialize_optional_string_or_number_required")]
    code: String,
    #[serde(default, alias = "msg")]
    message: Option<String>,
    #[serde(default, alias = "requestId")]
    request_id: Option<String>,
    #[serde(default)]
    data: Option<HonorSendData>,
}

impl HonorSendResponse {
    fn is_success(&self) -> bool {
        if self.code == HONOR_SUCCESS_CODE || self.code == "200" {
            return true;
        }
        self.data
            .as_ref()
            .and_then(|d| d.send_result)
            .unwrap_or(false)
    }

    fn code_text(&self) -> Option<String> {
        Some(self.code.clone())
    }

    fn message_text(&self) -> Option<&str> {
        self.message.as_deref()
    }

    fn request_id(&self) -> Option<String> {
        self.request_id
            .clone()
            .or_else(|| self.data.as_ref().and_then(|d| d.request_id.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ClickAction, ClickActionType, DeliveryMode, RenderedNotification, TemplateChannels};
    use serde_json::json;

    #[test]
    fn resolve_honor_credentials_requires_all_three_fields() {
        let err = resolve_honor_credentials(
            Some("app-1".into()),
            None,
            Some("secret".into()),
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("client_id"));

        let creds = resolve_honor_credentials(
            Some(" 104436234 ".into()),
            Some(" client-abc ".into()),
            Some(" secret ".into()),
        )
        .expect("credentials");
        assert_eq!(creds.app_id, "104436234");
        assert_eq!(creds.oauth_client_id, "client-abc");
        assert_eq!(creds.client_secret, "secret");
    }

    #[test]
    fn format_oauth_failure_maps_invalid_client_to_friendly_message() {
        let parsed = HonorOAuthResponse {
            access_token: None,
            expires_in: None,
            error: Some("unauthorized_client".into()),
            error_description: Some(
                "Invalid client or Invalid client credentials".into(),
            ),
        };
        let message = format_oauth_failure(&parsed);
        assert!(message.contains("Client ID"));
        assert!(!message.contains("url="));
    }

    #[test]
    fn notification_message_uses_honor_send_message_shape() {
        let when = "2014-10-02T15:01:23.045123456Z";
        let android = build_android_notification(
            "title",
            "body",
            &ClickAction {
                r#type: ClickActionType::OpenApp,
                ..Default::default()
            },
            "com.example.app",
            "NORMAL",
            Some(12345),
            when,
        );
        let notification = RenderedNotification {
            title: "title".into(),
            body: "body".into(),
            package_name: "com.example.app".into(),
            payload: json!({"k": "v"}),
            click_action: ClickAction {
                r#type: ClickActionType::OpenApp,
                ..Default::default()
            },
            delivery_mode: DeliveryMode::Notification,
            notify_id: None,
            channels: TemplateChannels::default(),
            vendor_fallback: None,
            expires_at: chrono::Utc::now(),
            title_variables: Default::default(),
            body_variables: Default::default(),
        };
        let message = build_notification_message(
            &["token-1".into()],
            &notification,
            &android,
            &notification.payload.to_string(),
        );

        assert_eq!(message["token"], json!(["token-1"]));
        assert_eq!(message["notification"]["title"], "title");
        assert_eq!(message["android"]["ttl"], HONOR_DEFAULT_TTL);
        assert_eq!(message["android"]["notification"]["title"], "title");
        assert_eq!(message["android"]["notification"]["body"], "body");
        assert_eq!(message["android"]["notification"]["when"], when);
        assert_eq!(message["android"]["notification"]["style"], 0);
        assert_eq!(message["android"]["notification"]["clickAction"]["type"], 3);
        assert_eq!(message["android"]["notification"]["importance"], "NORMAL");
        assert_eq!(message["android"]["notification"]["notifyId"], 12345);
        assert!(message.get("validate_only").is_none());
        assert!(message.get("message").is_none());
    }

    #[test]
    fn honor_notification_when_uses_utc_nanoseconds() {
        let when = honor_notification_when(
            chrono::DateTime::parse_from_rfc3339("2014-10-02T15:01:23.045123456Z")
                .expect("parse")
                .with_timezone(&chrono::Utc),
        );
        assert_eq!(when, "2014-10-02T15:01:23.045123456Z");
    }

    #[test]
    fn send_response_accepts_nested_request_id() {
        let parsed: HonorSendResponse = serde_json::from_str(
            r#"{"code":200,"message":"success","data":{"sendResult":true,"requestId":"req-1"}}"#,
        )
        .expect("parse");
        assert!(parsed.is_success());
        assert_eq!(parsed.request_id(), Some("req-1".into()));
    }
}