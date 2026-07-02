use std::time::{Duration, Instant};

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;

use crate::models::{
    click_params_object, ClickAction, ClickActionType, RenderedNotification,
};
use crate::push::{ProviderSendResult, PushProvider};
use crate::AppError;
use crate::AppResult;

const OPPO_AUTH_URL: &str = "https://api.push.oppomobile.com/server/v1/auth";
const OPPO_UNICAST_URL: &str = "https://api.push.oppomobile.com/server/v1/message/notification/unicast";
const OPPO_UNICAST_BATCH_URL: &str =
    "https://api.push.oppomobile.com/server/v1/message/notification/unicast_batch";

/// OPPO 通知 clickActionType
/// 0=启动应用；1=应用内页(intent action)；2=网页；4=应用内页(全类名)；5=Intent scheme
const OPPO_CLICK_APP: i64 = 0;
const OPPO_CLICK_URL: i64 = 2;
const OPPO_CLICK_ACTIVITY_FQCN: i64 = 4;

pub struct OppoPushProvider {
    client: Client,
    app_key: String,
    master_secret: String,
    token_cache: Mutex<Option<CachedToken>>,
}

struct CachedToken {
    auth_token: String,
    expires_at: Instant,
}

impl OppoPushProvider {
    pub fn new(app_key: String, master_secret: String) -> Self {
        Self {
            client: Client::new(),
            app_key,
            master_secret,
            token_cache: Mutex::new(None),
        }
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
        let sign = compute_sign(&self.app_key, timestamp, &self.master_secret);

        let response = self
            .client
            .post(OPPO_AUTH_URL)
            .form(&[
                ("app_key", self.app_key.as_str()),
                ("timestamp", &timestamp.to_string()),
                ("sign", sign.as_str()),
            ])
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;
        let parsed: OppoAuthResponse = serde_json::from_str(&body).map_err(|err| {
            AppError::Push(format!(
                "invalid oppo auth response (status={status}): {err}; body={body}"
            ))
        })?;

        if parsed.code != 0 {
            return Err(AppError::Push(format!(
                "oppo auth failed code={}: {}",
                parsed.code,
                parsed.message.unwrap_or_else(|| "unknown error".into())
            )));
        }

        let auth_token = parsed
            .data
            .and_then(|d| d.auth_token)
            .ok_or_else(|| AppError::Push("oppo auth response missing auth_token".into()))?;

        *self.token_cache.lock().await = Some(CachedToken {
            auth_token: auth_token.clone(),
            expires_at: Instant::now() + Duration::from_secs(23 * 3600),
        });

        Ok(auth_token)
    }

    pub async fn validate_credentials(&self) -> AppResult<()> {
        self.auth_token().await.map(|_| ())
    }
}

#[async_trait]
impl PushProvider for OppoPushProvider {
    fn platform(&self) -> &'static str {
        "oppo"
    }

    async fn send(
        &self,
        push_tokens: &[String],
        notification: &RenderedNotification,
    ) -> AppResult<ProviderSendResult> {
        if push_tokens.is_empty() {
            return Err(AppError::BadRequest("push_tokens cannot be empty".into()));
        }

        if !notification.channels.oppo_can_send() {
            return Err(AppError::BadRequest(
                "OPPO push requires a mapped private template, or IM / public category".into(),
            ));
        }

        let auth_token = self.auth_token().await?;
        let payload = if notification.payload.is_null() {
            String::new()
        } else {
            notification.payload.to_string()
        };

        if push_tokens.len() == 1 {
            let message = build_unicast_message(push_tokens[0].as_str(), notification, &payload)?
                .to_string();
            let response = self
                .client
                .post(OPPO_UNICAST_URL)
                .form(&[
                    ("auth_token", auth_token.as_str()),
                    ("message", message.as_str()),
                ])
                .send()
                .await?;

            let status = response.status();
            let body = response.text().await?;
            let parsed: OppoSendResponse = serde_json::from_str(&body).map_err(|err| {
                AppError::Push(format!(
                    "invalid oppo unicast response (status={status}): {err}; body={body}"
                ))
            })?;

            if parsed.code != 0 {
                return Err(AppError::Push(format!(
                    "oppo api error code={}; body={body}",
                    parsed.code
                )));
            }

            let message_id = parsed.data.and_then(|d| d.message_id);
            return Ok(ProviderSendResult {
                success_count: 1,
                failure_count: 0,
                message_id,
                outbox_ids: vec![],
                ws_delivered: 0,
            });
        }

        let messages: Vec<Value> = push_tokens
            .iter()
            .map(|token| build_unicast_message(token, notification, &payload))
            .collect::<AppResult<Vec<_>>>()?;

        let messages_json = serde_json::to_string(&messages).map_err(|err| {
            AppError::Push(format!("failed to encode oppo batch messages: {err}"))
        })?;

        let response = self
            .client
            .post(OPPO_UNICAST_BATCH_URL)
            .form(&[
                ("auth_token", auth_token.as_str()),
                ("messages", messages_json.as_str()),
            ])
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;
        let parsed: OppoBatchSendResponse = serde_json::from_str(&body).map_err(|err| {
            AppError::Push(format!(
                "invalid oppo batch response (status={status}): {err}; body={body}"
            ))
        })?;

        if parsed.code != 0 {
            return Err(AppError::Push(format!(
                "oppo api error code={}; body={body}",
                parsed.code
            )));
        }

        let mut success_count = 0usize;
        let mut failure_count = 0usize;
        let mut message_id = None;
        if let Some(items) = parsed.data {
            for item in items {
                if item.error_code.unwrap_or(0) == 0 {
                    success_count += 1;
                    message_id.get_or_insert_with(|| item.message_id.clone().unwrap_or_default());
                } else {
                    failure_count += 1;
                }
            }
        } else {
            success_count = push_tokens.len();
        }

        Ok(ProviderSendResult {
            success_count,
            failure_count,
            message_id,
            outbox_ids: vec![],
            ws_delivered: 0,
        })
    }
}

fn compute_sign(app_key: &str, timestamp: i64, master_secret: &str) -> String {
    let raw = format!("{app_key}{timestamp}{master_secret}");
    let hash = Sha256::digest(raw.as_bytes());
    format!("{:x}", hash)
}

fn build_unicast_message(
    registration_id: &str,
    notification: &RenderedNotification,
    payload: &str,
) -> AppResult<Value> {
    let notification_body = build_notification_body(notification, payload)?;
    Ok(json!({
        "target_type": 2,
        "target_value": registration_id,
        "verify_registration_id": false,
        "notification": notification_body,
    }))
}

fn build_notification_body(
    notification: &RenderedNotification,
    payload: &str,
) -> AppResult<Value> {
    let (click_action_type, click_action_activity, click_action_url) =
        map_click_action(&notification.click_action);

    let mut action_parameters = click_params_object(&notification.click_action.params);
    if !payload.is_empty() {
        action_parameters["payload"] = json!(payload);
    }

    let uses_private_template = notification.channels.oppo_private_template_id().is_some();

    // 公信 / IM：下发已拼接 title/content。
    // 私信审核模板：只传 private_msg_template_id + 参数，由 OPPO 按模板渲染，禁止自拟正文。
    let mut body = json!({
        "clickActionType": click_action_type,
        "actionParameters": action_parameters.to_string(),
        "offLine": true,
        "offLineTtl": 86400,
        "showTimeType": 0,
        "style": 1,
    });
    if let Some(activity) = click_action_activity {
        body["clickActionActivity"] = Value::String(activity);
    }
    if let Some(url) = click_action_url {
        body["clickActionUrl"] = Value::String(url);
    }
    if !uses_private_template {
        body["title"] = Value::String(notification.title.clone());
        body["content"] = Value::String(notification.body.clone());
    }
    if let Some(category) = notification.channels.oppo_category() {
        body["category"] = Value::String(category.to_uppercase());
    }
    if let Some(channel_id) = notification.channels.oppo_push_channel_id() {
        body["channelId"] = Value::String(channel_id.to_string());
    }
    apply_oppo_template_mapping(&mut body, notification);
    if let Some(notify_id) = notification.notify_id {
        body["notifyId"] = json!(notify_id);
    }
    Ok(body)
}

/// 我方拼接模板 ↔ OPPO 审核模板一对一：只传模板 ID 与变量参数。
/// 其它厂商仍使用 notification.title / body（已按 {{变量}} 自动拼接）。
fn apply_oppo_template_mapping(body: &mut Value, notification: &RenderedNotification) {
    let Some(template_id) = notification.channels.oppo_private_template_id() else {
        return;
    };
    body["private_msg_template_id"] = Value::String(template_id.to_string());

    // 标题/正文无占位符时不要传空对象；OPPO 固定标题模板传 {} 会校验失败。
    let title_params = nonempty_template_params(&notification.title_variables);
    if !title_params.is_empty() {
        body["private_title_parameters"] = json!(title_params);
    }
    let content_params = nonempty_template_params(&notification.body_variables);
    if !content_params.is_empty() {
        body["private_content_parameters"] = json!(content_params);
    }

    // 通讯与服务默认提醒等级：通知栏+锁屏。
    if body.get("notifyLevel").is_none() {
        body["notifyLevel"] = json!(2);
    }
}

fn nonempty_template_params(
    variables: &std::collections::HashMap<String, String>,
) -> std::collections::HashMap<&str, &str> {
    variables
        .iter()
        .filter_map(|(key, value)| {
            let key = key.trim();
            let value = value.trim();
            if key.is_empty() || value.is_empty() {
                None
            } else {
                Some((key, value))
            }
        })
        .collect()
}

fn map_click_action(action: &ClickAction) -> (i64, Option<String>, Option<String>) {
    match action.r#type {
        ClickActionType::OpenApp => (OPPO_CLICK_APP, None, None),
        ClickActionType::OpenPage => {
            let activity = action
                .activity_class()
                .expect("validated open_page requires FQCN activity");
            // 4 = 全路径类名；1 = intent action（不是类名）
            (OPPO_CLICK_ACTIVITY_FQCN, Some(activity.to_string()), None)
        }
        ClickActionType::OpenWeb => {
            let url = action
                .url_str()
                .expect("validated open_web requires url");
            (OPPO_CLICK_URL, None, Some(url.to_string()))
        }
    }
}

#[derive(Debug, Deserialize)]
struct OppoAuthResponse {
    code: i64,
    message: Option<String>,
    data: Option<OppoAuthData>,
}

#[derive(Debug, Deserialize)]
struct OppoAuthData {
    auth_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OppoSendResponse {
    code: i64,
    message: Option<String>,
    data: Option<OppoSendData>,
}

#[derive(Debug, Deserialize)]
struct OppoSendData {
    message_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OppoBatchSendResponse {
    code: i64,
    message: Option<String>,
    data: Option<Vec<OppoBatchSendItem>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OppoBatchSendItem {
    message_id: Option<String>,
    registration_id: Option<String>,
    error_code: Option<i64>,
    error_message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        ClickAction, ClickActionType, DeliveryMode, OppoChannelConfig, TemplateChannels,
    };
    use std::collections::HashMap;

    #[test]
    fn compute_sign_uses_sha256_hex() {
        let sign = compute_sign("appkey", 1700000000000, "secret");
        assert_eq!(sign.len(), 64);
        assert!(sign.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn private_template_payload_uses_json_objects_and_skips_category_as_channel() {
        let mut title_variables = HashMap::new();
        title_variables.insert("city".into(), "北京".into());
        let mut body_variables = HashMap::new();
        body_variables.insert("userName".into(), "汤姆".into());

        let notification = RenderedNotification {
            title: "欢迎来到北京".into(),
            body: "欢迎汤姆".into(),
            payload: serde_json::Value::Null,
            click_action: ClickAction::default(),
            package_name: "com.test".into(),
            channels: TemplateChannels {
                oppo: Some(OppoChannelConfig {
                    category: Some("ORDER".into()),
                    channel_id: None,
                    private_template_id: Some(
                        "ac20d345c501088a844ce859e253e261718154058221aa87c0ba418a547c9a6c".into(),
                    ),
                }),
                ..Default::default()
            },
            delivery_mode: DeliveryMode::Notification,
            notify_id: None,
            vendor_fallback: None,
            expires_at: chrono::Utc::now(),
            title_variables,
            body_variables,
        };

        let body = build_notification_body(&notification, "").unwrap();
        assert_eq!(body["category"], json!("ORDER"));
        assert!(body.get("channelId").is_none());
        assert!(body.get("title").is_none());
        assert!(body.get("content").is_none());
        assert_eq!(
            body["private_msg_template_id"],
            json!("ac20d345c501088a844ce859e253e261718154058221aa87c0ba418a547c9a6c")
        );
        assert_eq!(body["private_title_parameters"], json!({"city":"北京"}));
        assert_eq!(body["private_content_parameters"], json!({"userName":"汤姆"}));
        assert_eq!(body["notifyLevel"], json!(2));
        assert!(body["private_title_parameters"].is_object());
        assert!(body["private_content_parameters"].is_object());
    }

    #[test]
    fn open_page_uses_click_action_type_4_with_fqcn() {
        let notification = RenderedNotification {
            title: "t".into(),
            body: "b".into(),
            payload: serde_json::Value::Null,
            click_action: ClickAction {
                r#type: ClickActionType::OpenPage,
                activity: Some("com.jiangker.push.sample.DemoTargetActivity".into()),
                url: None,
                params: Default::default(),
            },
            package_name: "com.jiangker.push.sample".into(),
            channels: TemplateChannels {
                oppo: Some(OppoChannelConfig {
                    category: Some("IM".into()),
                    channel_id: None,
                    private_template_id: None,
                }),
                ..Default::default()
            },
            delivery_mode: DeliveryMode::Notification,
            notify_id: None,
            vendor_fallback: None,
            expires_at: chrono::Utc::now(),
            title_variables: HashMap::new(),
            body_variables: HashMap::new(),
        };

        let body = build_notification_body(&notification, "").unwrap();
        assert_eq!(body["clickActionType"], json!(4));
        assert_eq!(
            body["clickActionActivity"],
            json!("com.jiangker.push.sample.DemoTargetActivity")
        );
        assert!(body.get("clickActionUrl").is_none());
    }

    #[test]
    fn omits_title_parameters_when_title_has_no_placeholders() {
        let mut body_variables = HashMap::new();
        body_variables.insert("userName".into(), "汤姆".into());

        let notification = RenderedNotification {
            title: "订单提醒".into(),
            body: "欢迎汤姆".into(),
            payload: serde_json::Value::Null,
            click_action: ClickAction::default(),
            package_name: "com.test".into(),
            channels: TemplateChannels {
                oppo: Some(OppoChannelConfig {
                    category: Some("ORDER".into()),
                    // 旧错误数据：channel_id 被存成了 category
                    channel_id: Some("ORDER".into()),
                    private_template_id: Some("tpl-fixed-title".into()),
                }),
                ..Default::default()
            },
            delivery_mode: DeliveryMode::Notification,
            notify_id: None,
            vendor_fallback: None,
            expires_at: chrono::Utc::now(),
            title_variables: HashMap::new(),
            body_variables,
        };

        let body = build_notification_body(&notification, "").unwrap();
        assert_eq!(body["category"], json!("ORDER"));
        assert!(body.get("channelId").is_none());
        assert!(body.get("title").is_none());
        assert!(body.get("content").is_none());
        assert!(body.get("private_title_parameters").is_none());
        assert_eq!(body["private_content_parameters"], json!({"userName":"汤姆"}));
        assert_eq!(body["private_msg_template_id"], json!("tpl-fixed-title"));
    }

    #[test]
    fn public_message_still_sends_rendered_title_and_content() {
        let notification = RenderedNotification {
            title: "活动通知".into(),
            body: "限时优惠".into(),
            payload: serde_json::Value::Null,
            click_action: ClickAction::default(),
            package_name: "com.test".into(),
            channels: TemplateChannels {
                oppo: Some(OppoChannelConfig {
                    category: Some("MARKETING".into()),
                    channel_id: None,
                    private_template_id: None,
                }),
                ..Default::default()
            },
            delivery_mode: DeliveryMode::Notification,
            notify_id: None,
            vendor_fallback: None,
            expires_at: chrono::Utc::now(),
            title_variables: HashMap::new(),
            body_variables: HashMap::new(),
        };

        let body = build_notification_body(&notification, "").unwrap();
        assert_eq!(body["title"], json!("活动通知"));
        assert_eq!(body["content"], json!("限时优惠"));
        assert!(body.get("private_msg_template_id").is_none());
    }

    #[test]
    fn notification_body_includes_notify_id_when_set() {
        let notification = RenderedNotification {
            title: "t".into(),
            body: "b".into(),
            payload: serde_json::Value::Null,
            click_action: ClickAction::default(),
            package_name: "com.test".into(),
            channels: TemplateChannels::default(),
            delivery_mode: DeliveryMode::Notification,
            notify_id: Some(42),
            vendor_fallback: None,
            expires_at: chrono::Utc::now(),
            title_variables: HashMap::new(),
            body_variables: HashMap::new(),
        };
        let body = build_notification_body(&notification, "").unwrap();
        assert_eq!(body["notifyId"], json!(42));
    }
}
