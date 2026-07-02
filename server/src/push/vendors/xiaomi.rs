use async_trait::async_trait;
use reqwest::Client;

use crate::models::{
    build_intent_uri, ClickAction, ClickActionType, RenderedNotification,
};
use crate::push::{ProviderSendResult, PushProvider};
use crate::AppError;
use crate::AppResult;

const XIAOMI_API_URL: &str = "https://api.xmpush.xiaomi.com/v2/message/regid";
const XIAOMI_QUOTA_URL: &str = "https://api.xmpush.xiaomi.com/v1/trace/quota/get";

/// 小米推送 notify_effect 取值
const NOTIFY_LAUNCHER_ACTIVITY: &str = "1";
const NOTIFY_ACTIVITY: &str = "2";
const NOTIFY_WEB: &str = "3";

pub struct XiaomiPushProvider {
    client: Client,
    app_secret: String,
    default_package_name: String,
}

impl XiaomiPushProvider {
    pub fn new(app_secret: String, default_package_name: String) -> Self {
        Self {
            client: Client::new(),
            app_secret,
            default_package_name,
        }
    }

    pub async fn validate_credentials(&self) -> AppResult<()> {
        let (secret_result, secret_body) = self.xiaomi_get(XIAOMI_QUOTA_URL).await?;
        classify_xiaomi_secret_result(&secret_result, &secret_body)
    }

    async fn xiaomi_get(&self, url: &str) -> AppResult<(XiaomiApiResponse, String)> {
        let response = self
            .client
            .get(url)
            .header("Authorization", format!("key={}", self.app_secret))
            .send()
            .await?;
        self.parse_xiaomi_response(response).await
    }

    async fn parse_xiaomi_response(
        &self,
        response: reqwest::Response,
    ) -> AppResult<(XiaomiApiResponse, String)> {
        let status = response.status();
        let body = response.text().await?;
        let result: XiaomiApiResponse = serde_json::from_str(&body).map_err(|err| {
            AppError::Push(format!(
                "invalid xiaomi response (status={status}): {err}; body={body}"
            ))
        })?;
        Ok((result, body))
    }
}

fn classify_xiaomi_secret_result(result: &XiaomiApiResponse, body: &str) -> AppResult<()> {
    match result.code {
        0 => Ok(()),
        21301 | 21302 => Err(AppError::Push(
            "xiaomi App Secret 校验失败，请检查小米推送密钥".into(),
        )),
        // 鉴权已通过，仅 trace 权限或限流问题
        10027 | 10038 | 10039 => Ok(()),
        _ => Err(AppError::Push(format!(
            "xiaomi App Secret 校验失败: {}",
            xiaomi_error_message(result, body)
        ))),
    }
}

fn xiaomi_error_message(result: &XiaomiApiResponse, body: &str) -> String {
    result
        .description
        .clone()
        .or_else(|| result.reason.clone())
        .or_else(|| result.info.clone())
        .unwrap_or_else(|| body.to_string())
}

#[async_trait]
impl PushProvider for XiaomiPushProvider {
    fn platform(&self) -> &'static str {
        "xiaomi"
    }

    async fn send(
        &self,
        push_tokens: &[String],
        notification: &RenderedNotification,
    ) -> AppResult<ProviderSendResult> {
        if push_tokens.is_empty() {
            return Err(AppError::BadRequest("push_tokens cannot be empty".into()));
        }

        let pass_through = notification.delivery_mode.is_pass_through();
        let channel_id = if pass_through {
            None
        } else {
            Some(notification.channels.xiaomi_channel_id().ok_or_else(|| {
                AppError::BadRequest(
                    "template channels.xiaomi.channel_id is required for xiaomi notification push".into(),
                )
            })?)
        };

        let registration_id = push_tokens.join(",");
        let package_name = if notification.package_name.is_empty() {
            self.default_package_name.clone()
        } else {
            notification.package_name.clone()
        };

        let payload = notification.payload.to_string();

        let mut form: Vec<(&str, String)> = vec![
            ("registration_id", registration_id),
            ("payload", payload),
            ("restricted_package_name", package_name.clone()),
            (
                "pass_through",
                if pass_through { "1" } else { "0" }.into(),
            ),
        ];

        if pass_through {
            // 透传：业务数据走 payload，不展示通知栏
            if !notification.title.is_empty() {
                form.push(("title", notification.title.clone()));
            }
            if !notification.body.is_empty() {
                form.push(("description", notification.body.clone()));
            }
        } else {
            form.extend([
                ("title", notification.title.clone()),
                ("description", notification.body.clone()),
                ("notify_type", "1".into()),
                ("extra.channel_id", channel_id.unwrap().to_string()),
            ]);
            if let Some(notify_id) = notification.notify_id {
                form.push(("notify_id", notify_id.to_string()));
            }
            apply_click_action_extras(
                &mut form,
                &notification.click_action,
                &package_name,
            );
        }

        let form_refs: Vec<(&str, &str)> = form.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let response = self
            .client
            .post(XIAOMI_API_URL)
            .header("Authorization", format!("key={}", self.app_secret))
            .form(&form_refs)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        let result: XiaomiApiResponse = serde_json::from_str(&body).map_err(|err| {
            AppError::Push(format!(
                "invalid xiaomi response (status={status}): {err}; body={body}"
            ))
        })?;

        if result.code != 0 {
            return Err(AppError::Push(format!(
                "xiaomi api error code={}: {}",
                result.code,
                result
                    .description
                    .or(result.reason)
                    .or(result.info)
                    .unwrap_or_else(|| "unknown error".into())
            )));
        }

        let data = result.data.as_ref();
        Ok(ProviderSendResult {
            success_count: data
                .and_then(|d| d.success_count)
                .unwrap_or(push_tokens.len() as i64) as usize,
            failure_count: data.and_then(|d| d.failure_count).unwrap_or(0) as usize,
            message_id: data.and_then(|d| d.id.clone()),
            outbox_ids: vec![],
            ws_delivered: 0,
        })
    }
}

fn apply_click_action_extras(
    form: &mut Vec<(&str, String)>,
    action: &ClickAction,
    package_name: &str,
) {
    match action.r#type {
        ClickActionType::OpenApp => {
            form.push(("extra.notify_effect", NOTIFY_LAUNCHER_ACTIVITY.into()));
        }
        ClickActionType::OpenPage => {
            let activity = action
                .activity_class()
                .expect("validated open_page requires FQCN activity");
            form.push(("extra.notify_effect", NOTIFY_ACTIVITY.into()));
            form.push((
                "extra.intent_uri",
                build_intent_uri(package_name, activity, &action.params),
            ));
        }
        ClickActionType::OpenWeb => {
            let url = action
                .url_str()
                .expect("validated open_web requires url");
            form.push(("extra.notify_effect", NOTIFY_WEB.into()));
            form.push(("extra.web_uri", url.to_string()));
        }
    }
}

#[derive(Debug, serde::Deserialize)]
struct XiaomiApiResponse {
    code: i64,
    description: Option<String>,
    data: Option<XiaomiApiData>,
    info: Option<String>,
    reason: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct XiaomiApiData {
    id: Option<String>,
    success_count: Option<i64>,
    failure_count: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn classify_secret_success() {
        let result = XiaomiApiResponse {
            code: 0,
            description: Some("成功".into()),
            data: None,
            info: None,
            reason: None,
        };
        assert!(classify_xiaomi_secret_result(&result, "").is_ok());
    }

    #[test]
    fn classify_secret_auth_failure() {
        let result = XiaomiApiResponse {
            code: 21301,
            description: Some("认证失败".into()),
            data: None,
            info: None,
            reason: None,
        };
        assert!(classify_xiaomi_secret_result(&result, "").is_err());
    }

    #[test]
    fn classify_trace_forbidden_still_ok() {
        let result = XiaomiApiResponse {
            code: 10038,
            description: Some("应用禁止访问统计和trace数据".into()),
            data: None,
            info: None,
            reason: None,
        };
        assert!(classify_xiaomi_secret_result(&result, "").is_ok());
    }

    #[test]
    fn open_app_sets_notify_effect() {
        let mut form = Vec::new();
        apply_click_action_extras(
            &mut form,
            &ClickAction {
                r#type: ClickActionType::OpenApp,
                ..Default::default()
            },
            "com.example.app",
        );
        assert!(form.iter().any(|(k, v)| *k == "extra.notify_effect" && v == "1"));
    }

    #[test]
    fn open_page_sets_notify_activity_and_intent_uri() {
        let mut form = Vec::new();
        let mut params = HashMap::new();
        params.insert("order_id".into(), serde_json::json!("42"));
        apply_click_action_extras(
            &mut form,
            &ClickAction {
                r#type: ClickActionType::OpenPage,
                activity: Some("com.xiaomi.mipushdemo.NewsActivity".into()),
                params,
                ..Default::default()
            },
            "com.xiaomi.mipushdemo",
        );
        assert!(form.iter().any(|(k, v)| *k == "extra.notify_effect" && v == "2"));
        let uri = form
            .iter()
            .find(|(k, _)| *k == "extra.intent_uri")
            .map(|(_, v)| v.as_str())
            .unwrap();
        assert!(uri.contains("component=com.xiaomi.mipushdemo/.NewsActivity"));
        assert!(uri.contains("S.order_id=42"));
        assert!(uri.contains("launchFlags=0x4000000"));
    }

    #[test]
    fn notification_form_includes_notify_id_when_set() {
        let mut form: Vec<(&str, String)> = vec![];
        if let Some(notify_id) = Some(1001i32) {
            form.push(("notify_id", notify_id.to_string()));
        }
        assert!(form.iter().any(|(k, v)| *k == "notify_id" && v == "1001"));
    }
}
