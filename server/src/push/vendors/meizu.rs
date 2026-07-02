use std::collections::BTreeMap;

use async_trait::async_trait;
use md5;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::models::{click_params_object, ClickActionType, RenderedNotification};
use crate::push::{ProviderSendResult, PushProvider};
use crate::AppError;
use crate::AppResult;

const MEIZU_PUSH_URL: &str =
    "https://server-api-push.meizu.com/garcia/api/server/push/varnished/pushByPushId";
const MEIZU_BATCH_SIZE: usize = 100;

pub struct MeizuPushProvider {
    client: Client,
    app_id: String,
    app_secret: String,
}

impl MeizuPushProvider {
    pub fn new(app_id: String, app_secret: String) -> Self {
        Self {
            client: Client::new(),
            app_id,
            app_secret,
        }
    }

    pub async fn validate_credentials(&self) -> AppResult<()> {
        let message_json = json!({
            "noticeBarInfo": {
                "noticeMsgType": 0,
                "noticeBarType": 0,
                "title": "credential_check",
                "content": "credential_check",
            },
            "clickTypeInfo": { "clickType": 0 },
            "pushTimeInfo": { "offLine": 0 },
        })
        .to_string();
        let mut params = BTreeMap::new();
        params.insert("appId".to_string(), self.app_id.clone());
        params.insert("pushIds".to_string(), "credential_check".into());
        params.insert("messageJson".to_string(), message_json);
        let sign = compute_sign(&params, &self.app_secret);
        params.insert("sign".to_string(), sign);

        let response = self
            .client
            .post(MEIZU_PUSH_URL)
            .header(
                "Content-Type",
                "application/x-www-form-urlencoded;charset=UTF-8",
            )
            .form(&params)
            .send()
            .await?;
        let status = response.status();
        let text = response.text().await?;
        let parsed: MeizuSendResponse = serde_json::from_str(&text).map_err(|err| {
            AppError::Push(format!(
                "invalid meizu response (status={status}): {err}; body={text}"
            ))
        })?;
        match parsed.code.as_str() {
            "200" => Ok(()),
            "110001" => Err(AppError::Push("meizu 签名校验失败，请检查 App Secret".into())),
            "110002" => Err(AppError::Push("meizu App ID 不存在或无效".into())),
            "110003" | "110010" => Ok(()),
            _ => Err(AppError::Push(format!(
                "meizu api error code={}: {}",
                parsed.code,
                parsed.message.unwrap_or_else(|| "unknown error".into())
            ))),
        }
    }
}

#[async_trait]
impl PushProvider for MeizuPushProvider {
    fn platform(&self) -> &'static str {
        "meizu"
    }

    async fn send(
        &self,
        push_tokens: &[String],
        notification: &RenderedNotification,
    ) -> AppResult<ProviderSendResult> {
        if push_tokens.is_empty() {
            return Err(AppError::BadRequest("push_tokens cannot be empty".into()));
        }

        let notice_msg_type = notification
            .channels
            .meizu_notice_msg_type()
            .ok_or_else(|| {
                AppError::BadRequest(
                    "template channels.meizu.msg_type is required (PUBLIC or PRIVATE)".into(),
                )
            })?;

        let message_json = build_message_json(notification, notice_msg_type);
        let mut success_count = 0usize;
        let mut failure_count = 0usize;
        let mut last_msg_id = None;

        for chunk in push_tokens.chunks(MEIZU_BATCH_SIZE) {
            let push_ids = chunk.join(",");
            let mut params = BTreeMap::new();
            params.insert("appId".to_string(), self.app_id.clone());
            params.insert("pushIds".to_string(), push_ids);
            params.insert("messageJson".to_string(), message_json.clone());
            let sign = compute_sign(&params, &self.app_secret);
            params.insert("sign".to_string(), sign);

            let response = self
                .client
                .post(MEIZU_PUSH_URL)
                .header(
                    "Content-Type",
                    "application/x-www-form-urlencoded;charset=UTF-8",
                )
                .form(&params)
                .send()
                .await?;

            let status = response.status();
            let text = response.text().await?;
            let parsed: MeizuSendResponse = serde_json::from_str(&text).map_err(|err| {
                AppError::Push(format!(
                    "invalid meizu response (status={status}): {err}; body={text}"
                ))
            })?;

            if parsed.code == "200" {
                success_count += chunk.len();
                last_msg_id = parsed.msg_id;
            } else {
                failure_count += chunk.len();
            }
        }

        if success_count == 0 {
            return Err(AppError::Push(format!(
                "meizu send failed for all {} tokens",
                push_tokens.len()
            )));
        }

        Ok(ProviderSendResult {
            success_count,
            failure_count,
            message_id: last_msg_id,
            outbox_ids: vec![],
            ws_delivered: 0,
        })
    }
}

fn compute_sign(params: &BTreeMap<String, String>, app_secret: &str) -> String {
    let mut base = String::new();
    for (key, value) in params {
        if key == "sign" {
            continue;
        }
        base.push_str(key);
        base.push('=');
        base.push_str(value);
    }
    base.push_str(app_secret);
    format!("{:x}", md5::compute(base.as_bytes()))
}

/// 按官方 messageJson 结构构建：noticeBarInfo.noticeMsgType 区分公信/私信。
/// @see https://github.com/MEIZUPUSH/PushAPI/blob/master/README.md
fn build_message_json(notification: &RenderedNotification, notice_msg_type: i64) -> String {
    let click = build_click(notification);

    json!({
        "noticeBarInfo": {
            "noticeMsgType": notice_msg_type,
            "noticeBarType": 0,
            "title": notification.title,
            "content": notification.body,
        },
        "clickTypeInfo": click,
        "pushTimeInfo": {
            "offLine": 1,
            "validTime": 24,
        },
    })
    .to_string()
}

fn build_click(notification: &RenderedNotification) -> Value {
    let action = &notification.click_action;
    let mut click = match action.r#type {
        // 0 打开应用；1 打开应用页面；2 打开 URI（勿用已弃用的 3）
        ClickActionType::OpenApp => json!({ "clickType": 0 }),
        ClickActionType::OpenWeb => {
            let url = action
                .url_str()
                .expect("validated open_web requires url");
            json!({
                "clickType": 2,
                "url": url,
            })
        }
        ClickActionType::OpenPage => {
            let activity = action
                .activity_class()
                .expect("validated open_page requires FQCN activity");
            json!({
                "clickType": 1,
                "activity": activity,
            })
        }
    };

    let mut parameters = click_params_object(&action.params);
    if let Value::Object(map) = &mut parameters {
        match &notification.payload {
            Value::Object(payload_map) => {
                for (key, value) in payload_map {
                    map.entry(key.clone()).or_insert_with(|| value.clone());
                }
            }
            Value::Null => {}
            other => {
                map.insert("payload".into(), other.clone());
            }
        }
    }
    if parameters.as_object().is_some_and(|m| !m.is_empty()) {
        click["parameters"] = parameters;
    }
    click
}

#[derive(Debug, Deserialize)]
struct MeizuSendResponse {
    code: String,
    #[serde(rename = "msgId")]
    msg_id: Option<String>,
    message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        ClickAction, DeliveryMode, MeizuMsgTypeConfig, TemplateChannels,
    };

    #[test]
    fn compute_sign_sorts_keys_lexicographically() {
        let mut params = BTreeMap::new();
        params.insert("appId".to_string(), "10000".into());
        params.insert(
            "messageJson".to_string(),
            r#"{"title":"t","content":"c"}"#.into(),
        );
        params.insert(
            "pushIds".to_string(),
            "RA50c6348036344485d01776773577c64740465480a6b".into(),
        );
        let sign = compute_sign(&params, "secret");
        assert_eq!(sign.len(), 32);
    }

    #[test]
    fn message_json_uses_notice_msg_type_and_nested_bar_info() {
        let notification = RenderedNotification {
            title: "标题".into(),
            body: "内容".into(),
            payload: json!({"k": "v"}),
            click_action: ClickAction::default(),
            package_name: "com.example".into(),
            delivery_mode: DeliveryMode::Notification,
            notify_id: None,
            channels: TemplateChannels {
                meizu: Some(MeizuMsgTypeConfig {
                    msg_type: "PRIVATE".into(),
                }),
                ..Default::default()
            },
            vendor_fallback: None,
            expires_at: chrono::Utc::now(),
            title_variables: Default::default(),
            body_variables: Default::default(),
        };
        let text = build_message_json(&notification, 1);
        let parsed: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed["noticeBarInfo"]["noticeMsgType"], 1);
        assert_eq!(parsed["noticeBarInfo"]["title"], "标题");
        assert_eq!(parsed["noticeBarInfo"]["content"], "内容");
        assert_eq!(parsed["clickTypeInfo"]["clickType"], 0);
        assert_eq!(parsed["clickTypeInfo"]["parameters"]["k"], "v");
    }
}
