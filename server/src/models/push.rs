use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::models::{ClickAction, DeliveryMode, TemplateChannels};

#[derive(Debug, Clone, Deserialize)]
pub struct SendPushRequest {
    /// Push Hub 应用 ID；不传时使用默认应用
    #[serde(default)]
    pub app_id: Option<String>,
    /// 推送模板 ID；私信可不传模板直接推送
    #[serde(default)]
    pub template_id: Option<String>,
    /// 直接推送标题；无模板时必填，有模板时可覆盖渲染结果
    #[serde(default)]
    pub title: Option<String>,
    /// 直接推送内容；无模板时必填，有模板时可覆盖渲染结果
    #[serde(default)]
    pub body: Option<String>,
    /// 拼接模板标题变量，替换 title 中的 {{key}}
    #[serde(default)]
    pub title_variables: HashMap<String, String>,
    /// 拼接模板正文变量，替换 body 中的 {{key}}
    #[serde(default)]
    pub body_variables: HashMap<String, String>,
    /// 无模板时可选，未配置则使用应用默认推送通道
    #[serde(default)]
    pub channels: TemplateChannels,
    /// 业务 payload，任意 JSON
    #[serde(default = "default_payload")]
    pub payload: Value,
    /// 本次推送的点击行为（与具体消息内容绑定，不由模板限定）
    #[serde(default)]
    pub click_action: Option<ClickAction>,
    /// 投递模式：`notification`（默认）或 `data`（透传）
    #[serde(default)]
    pub delivery_mode: DeliveryMode,
    /// 通知栏 notify_id / notifyId（0~2147483647）；仅发送请求携带，相同 ID 的新消息覆盖旧通知。
    #[serde(default)]
    pub notify_id: Option<i32>,
    /// 覆盖模板默认缓存天数对应的截止时间（ISO 8601）
    #[serde(default)]
    pub cache_until: Option<DateTime<Utc>>,
    pub targets: PushTargets,
}

impl SendPushRequest {
    pub fn uses_template(&self) -> bool {
        self.template_id
            .as_ref()
            .is_some_and(|id| !id.trim().is_empty())
    }
}

fn default_payload() -> Value {
    json!({})
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PushTargets {
    #[serde(default)]
    pub device_ids: Vec<String>,
    #[serde(default)]
    pub push_tokens: Vec<String>,
    /// 使用 push_tokens 直接发送时必填
    pub platform: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SendPushResponse {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
    pub platforms: Vec<PlatformSendResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlatformSendResult {
    pub platform: String,
    pub success: usize,
    pub failed: usize,
    pub message_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OutboxMessage {
    pub id: String,
    pub title: String,
    pub body: String,
    pub payload: Value,
    pub delivery_mode: DeliveryMode,
    pub click_action: ClickAction,
    /// 发送请求中的 notify_id；在线通知栏相同 ID 会覆盖旧通知。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notify_id: Option<i32>,
    pub created_at: String,
}

impl OutboxMessage {
    pub fn to_online_ws_payload(&self) -> String {
        #[derive(Serialize)]
        struct Payload<'a> {
            #[serde(rename = "type")]
            msg_type: &'static str,
            #[serde(flatten)]
            message: &'a OutboxMessage,
        }
        serde_json::to_string(&Payload {
            msg_type: "message",
            message: self,
        })
        .expect("OutboxMessage serializes")
    }
}

#[derive(Debug, Clone)]
pub struct EnqueuedMessage {
    pub push_token: String,
    pub message: OutboxMessage,
}

#[derive(Debug, Clone)]
pub struct EnqueueResult {
    pub batch_id: String,
    pub messages: Vec<EnqueuedMessage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OnlineMessageAck {
    pub id: String,
    #[serde(default = "default_displayed")]
    pub displayed: bool,
    #[serde(default)]
    pub reason: Option<String>,
}

fn default_displayed() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct AckOnlineMessagesRequest {
    pub push_token: String,
    #[serde(default)]
    pub ids: Vec<String>,
    #[serde(default)]
    pub acks: Vec<OnlineMessageAck>,
}

impl AckOnlineMessagesRequest {
    pub fn normalized_acks(&self) -> Vec<OnlineMessageAck> {
        if !self.acks.is_empty() {
            return self.acks.clone();
        }
        self.ids
            .iter()
            .map(|id| OnlineMessageAck {
                id: id.clone(),
                displayed: true,
                reason: None,
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct OutboxFallbackJob {
    pub id: String,
    pub package_name: String,
    pub title: String,
    pub body: String,
    pub payload: Value,
    pub delivery_mode: DeliveryMode,
    pub fallback_platform: String,
    pub fallback_token: String,
    pub channels: TemplateChannels,
    pub click_action: ClickAction,
    pub title_variables: HashMap<String, String>,
    pub body_variables: HashMap<String, String>,
    pub notify_id: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct PushFallbackTarget {
    pub platform: String,
    pub push_token: String,
}

#[derive(Debug, Clone)]
pub struct RenderedNotification {
    pub title: String,
    pub body: String,
    pub payload: Value,
    pub click_action: ClickAction,
    pub package_name: String,
    pub channels: TemplateChannels,
    pub delivery_mode: DeliveryMode,
    /// 发送请求中的 notify_id；未设置则不传给厂商。
    pub notify_id: Option<i32>,
    /// 在线推送未送达时，降级到厂商离线推送
    pub vendor_fallback: Option<PushFallbackTarget>,
    /// 在线 outbox 消息缓存截止时间
    pub expires_at: DateTime<Utc>,
    /// 拼接模板标题变量（厂商私信模板填充用，如 OPPO）
    pub title_variables: HashMap<String, String>,
    /// 拼接模板正文变量
    pub body_variables: HashMap<String, String>,
}
