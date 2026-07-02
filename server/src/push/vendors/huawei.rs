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

const HUAWEI_OAUTH_URL: &str = "https://oauth-login.cloud.huawei.com/oauth2/v3/token";
const HUAWEI_PUSH_URL: &str = "https://push-api.cloud.huawei.com/v1/{app_id}/messages:send";

/// 华为 Android 通知 click_action.type
const HUAWEI_CLICK_INTENT: i64 = 1;
const HUAWEI_CLICK_URL: i64 = 2;
const HUAWEI_CLICK_APP: i64 = 3;

pub struct HuaweiPushProvider {
    client: Client,
    app_id: String,
    oauth_client_id: String,
    app_secret: String,
    default_package_name: String,
    token_cache: Mutex<Option<CachedToken>>,
}

struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

impl HuaweiPushProvider {
    pub fn new(
        app_id: String,
        oauth_client_id: String,
        app_secret: String,
        default_package_name: String,
    ) -> Self {
        Self {
            client: Client::new(),
            app_id,
            oauth_client_id,
            app_secret,
            default_package_name,
            token_cache: Mutex::new(None),
        }
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

        let response = self
            .client
            .post(HUAWEI_OAUTH_URL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", self.oauth_client_id.as_str()),
                ("client_secret", self.app_secret.as_str()),
            ])
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;
        let parsed: HuaweiOAuthResponse = serde_json::from_str(&body).map_err(|err| {
            AppError::Push(format!(
                "invalid huawei oauth response (status={status}): {err}; body={body}"
            ))
        })?;

        let access_token = match parsed.access_token {
            Some(token) => token,
            None => {
                return Err(AppError::Push(format_oauth_failure(
                    &parsed, status, &body,
                )))
            }
        };

        let ttl = parsed.expires_in.unwrap_or(3600).max(60);
        let expires_at = Instant::now() + Duration::from_secs((ttl - 60) as u64);
        *self.token_cache.lock().await = Some(CachedToken {
            access_token: access_token.clone(),
            expires_at,
        });

        Ok(access_token)
    }

    pub async fn validate_credentials(&self) -> AppResult<()> {
        self.access_token().await.map(|_| ())
    }
}

#[async_trait]
impl PushProvider for HuaweiPushProvider {
    fn platform(&self) -> &'static str {
        "huawei"
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
            .huawei_category()
            .map(|c| c.trim().to_uppercase())
            .filter(|c| !c.is_empty())
            .ok_or_else(|| {
                AppError::BadRequest(
                    "template channels.huawei.category is required for huawei push".into(),
                )
            })?;

        let access_token = self.access_token().await?;
        let url = HUAWEI_PUSH_URL.replace("{app_id}", &self.app_id);
        let android_notification = build_android_notification(
            notification,
            &notification.click_action,
            &package_name,
            notification.notify_id,
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
                &category,
            )
        };

        let body = json!({
            "validate_only": false,
            "message": message,
        });

        tracing::debug!(
            app_id = %self.app_id,
            category = %category,
            token_count = push_tokens.len(),
            huawei_body = %body,
            "huawei push request"
        );

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {access_token}"))
            .header("Content-Type", "application/json; charset=UTF-8")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        let text = response.text().await?;
        let result: HuaweiSendResponse = serde_json::from_str(&text).map_err(|err| {
            AppError::Push(format!(
                "invalid huawei response (status={status}): {err}; body={text}"
            ))
        })?;

        if result.code != "80000000" {
            return Err(AppError::Push(format_send_failure(
                &result,
                &self.app_id,
                push_tokens.len(),
            )));
        }

        Ok(ProviderSendResult {
            success_count: push_tokens.len(),
            failure_count: 0,
            message_id: result.request_id,
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
    category: &str,
) -> Value {
    let android = json!({
        "category": category,
        "ttl": "86400s",
        "urgency": "HIGH",
        "notification": android_notification,
    });
    let mut message = json!({
        "notification": {
            "title": notification.title,
            "body": notification.body,
        },
        "android": android,
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
        },
    });
    if !payload.is_empty() {
        message["data"] = Value::String(payload.to_string());
    }
    message
}

fn build_android_notification(
    notification: &RenderedNotification,
    action: &ClickAction,
    package_name: &str,
    notify_id: Option<i32>,
) -> Value {
    let mut android = json!({
        "title": notification.title,
        "body": notification.body,
        "click_action": build_click_action(action, package_name),
        "foreground_show": true,
        "importance": "NORMAL",
        "default_sound": true,
        "auto_cancel": true,
        "use_default_vibrate": true,
        "use_default_light": true,
    });
    if let Some(id) = notify_id {
        android["notify_id"] = json!(id);
    }
    android
}

fn build_click_action(action: &ClickAction, package_name: &str) -> Value {
    match action.r#type {
        ClickActionType::OpenApp => json!({ "type": HUAWEI_CLICK_APP }),
        ClickActionType::OpenPage => {
            let activity = action
                .activity_class()
                .expect("validated open_page requires FQCN activity");
            json!({
                "type": HUAWEI_CLICK_INTENT,
                "intent": build_intent_uri(package_name, activity, &action.params),
            })
        }
        ClickActionType::OpenWeb => {
            let url = action
                .url_str()
                .expect("validated open_web requires url");
            json!({
                "type": HUAWEI_CLICK_URL,
                "url": url,
            })
        }
    }
}

fn format_send_failure(result: &HuaweiSendResponse, app_id: &str, token_count: usize) -> String {
    let msg = result.msg.as_deref().unwrap_or("unknown error");
    let mut message = format!(
        "huawei api error code={}: {} (send_app_id={app_id}, token_count={token_count})",
        result.code, msg
    );
    if result.code == "80300002" {
        message.push_str(
            "; hint: 80300002 表示 token 与发送 URL 中的 app_id 不匹配。\
             HUAWEI_APP_ID 必须是 AppGallery Connect「应用 ID」（与 Android BuildConfig.HUAWEI_APP_ID 相同，如 108525411），\
             不是 OAuth Client ID。若 OAuth 需单独 Client ID，请配置 HUAWEI_OAUTH_CLIENT_ID。\
             修改配置后请让客户端重新获取 token 并重新注册设备。",
        );
    } else if result.code == "80300007" {
        message.push_str(
            "; hint: 80300007 表示消息分类 category 无效或未开通，请在 AppGallery Connect 申请对应分类（如 WORK / MARKETING）并在模板中配置。",
        );
    }
    message
}

fn format_oauth_failure(
    parsed: &HuaweiOAuthResponse,
    status: reqwest::StatusCode,
    body: &str,
) -> String {
    let mut message = format!("huawei oauth failed (status={status})");
    if let Some(code) = &parsed.error {
        message.push_str(&format!(", error={code}"));
    }
    if let Some(desc) = &parsed.error_description {
        message.push_str(&format!(", error_description={desc}"));
    }
    if let Some(sub_error) = parsed.sub_error {
        message.push_str(&format!(", sub_error={sub_error}"));
    }
    if parsed.error.is_none()
        && parsed.error_description.is_none()
        && parsed.sub_error.is_none()
    {
        message.push_str(&format!(", body={body}"));
    }
    if parsed.error_description.as_deref() == Some("invalid client_secret")
        || parsed.sub_error == Some(12304)
    {
        message.push_str(
            "; hint: HUAWEI_APP_SECRET 应为 AppGallery Connect 中该应用的「应用密钥 App secret」，\
             与 HUAWEI_APP_ID（应用 ID）配对使用，不是 agconnect-services.json 里的 client_secret",
        );
    }
    message
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

fn stringify_json_scalar(value: Value) -> String {
    match value {
        Value::String(text) => text,
        Value::Number(number) => number.to_string(),
        other => other.to_string(),
    }
}

#[derive(Debug, Deserialize)]
struct HuaweiOAuthResponse {
    access_token: Option<String>,
    expires_in: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_string_or_number")]
    error: Option<String>,
    error_description: Option<String>,
    sub_error: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct HuaweiSendResponse {
    code: String,
    msg: Option<String>,
    request_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::models::{ClickAction, ClickActionType, DeliveryMode, RenderedNotification, TemplateChannels};

    #[test]
    fn data_message_uses_data_field_only() {
        let message = build_data_message(
            &["token-1".into()],
            r#"{"order_id":"42"}"#,
            "WORK",
        );
        assert!(message.get("notification").is_none());
        assert_eq!(message["data"], r#"{"order_id":"42"}"#);
        assert_eq!(message["android"]["category"], "WORK");
    }

    #[test]
    fn parse_oauth_error_with_integer_code() {
        let body = r#"{"error":1101,"error_description":"invalid client_secret","sub_error":12304}"#;
        let parsed: HuaweiOAuthResponse = serde_json::from_str(body).unwrap();
        assert_eq!(parsed.error.as_deref(), Some("1101"));
        assert_eq!(parsed.error_description.as_deref(), Some("invalid client_secret"));
        assert_eq!(parsed.sub_error, Some(12304));
    }

    #[test]
    fn notification_message_puts_category_on_android_config() {
        let notification = sample_notification("Hello", "World");
        let android = build_android_notification(
            &notification,
            &ClickAction {
                r#type: ClickActionType::OpenApp,
                ..Default::default()
            },
            "com.example.app",
            None,
        );
        let message = build_notification_message(
            &["token-1".into()],
            &notification,
            &android,
            "",
            "WORK",
        );
        assert_eq!(message["android"]["category"], "WORK");
        assert!(message["android"]["notification"]["category"].is_null());
        assert_eq!(message["android"]["notification"]["title"], "Hello");
        assert_eq!(message["notification"]["title"], "Hello");
    }

    #[test]
    fn android_notification_includes_title_body() {
        let notification = sample_notification("Hello", "World");
        let android = build_android_notification(
            &notification,
            &ClickAction {
                r#type: ClickActionType::OpenApp,
                ..Default::default()
            },
            "com.example.app",
            None,
        );
        assert_eq!(android["title"], "Hello");
        assert_eq!(android["body"], "World");
        assert!(android.get("category").is_none());
    }

    #[test]
    fn android_notification_includes_work_category() {
        let notification = sample_notification("t", "b");
        let message = build_notification_message(
            &["token-1".into()],
            &notification,
            &build_android_notification(
                &notification,
                &ClickAction {
                    r#type: ClickActionType::OpenApp,
                    ..Default::default()
                },
                "com.example.app",
                None,
            ),
            "",
            "WORK",
        );
        assert_eq!(message["android"]["category"], "WORK");
    }

    #[test]
    fn android_notification_includes_notify_id_when_set() {
        let notification = sample_notification("Hello", "World");
        let android = build_android_notification(
            &notification,
            &ClickAction {
                r#type: ClickActionType::OpenApp,
                ..Default::default()
            },
            "com.example.app",
            Some(1001),
        );
        assert_eq!(android["notify_id"], 1001);
    }

    fn sample_notification(title: &str, body: &str) -> RenderedNotification {
        RenderedNotification {
            title: title.into(),
            body: body.into(),
            payload: Value::Null,
            click_action: ClickAction::default(),
            package_name: "com.example.app".into(),
            channels: TemplateChannels::default(),
            delivery_mode: DeliveryMode::Notification,
            notify_id: None,
            vendor_fallback: None,
            expires_at: chrono::Utc::now(),
            title_variables: HashMap::new(),
            body_variables: HashMap::new(),
        }
    }

    #[test]
    fn open_app_click_action() {
        let action = build_click_action(
            &ClickAction {
                r#type: ClickActionType::OpenApp,
                ..Default::default()
            },
            "com.example.app",
        );
        assert_eq!(action["type"], HUAWEI_CLICK_APP);
    }

    #[test]
    fn open_page_click_action_contains_intent() {
        let mut params = HashMap::new();
        params.insert("order_id".into(), json!("42"));
        params.insert("count".into(), json!(3));
        params.insert("vip".into(), json!(true));
        let action = build_click_action(
            &ClickAction {
                r#type: ClickActionType::OpenPage,
                activity: Some("com.example.app.OrderDetailActivity".into()),
                params,
                ..Default::default()
            },
            "com.example.app",
        );
        assert_eq!(action["type"], HUAWEI_CLICK_INTENT);
        let intent = action["intent"].as_str().unwrap();
        assert!(intent.contains("component=com.example.app/.OrderDetailActivity"));
        assert!(intent.contains("S.order_id=42"));
        assert!(intent.contains("i.count=3"));
        assert!(intent.contains("B.vip=true"));
        assert!(intent.contains("launchFlags=0x4000000"));
    }
}
