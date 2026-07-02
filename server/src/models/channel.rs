use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 应用级推送通道配置，供模板选择使用。
/// - 小米：`code` 为 NotificationChannel ID
/// - 华为 / 荣耀：`code` 为消息分类，如 `WORK`
/// - OPPO：`code` 为通知通道 ID
/// - vivo：`code` 为系统消息二级分类，如 `IM`、`ORDER`
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PushChannel {
    pub id: String,
    pub app_id: String,
    pub platform: String,
    pub name: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePushChannelRequest {
    pub platform: String,
    pub name: String,
    pub code: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub is_default: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePushChannelRequest {
    pub platform: String,
    pub name: String,
    pub code: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub is_default: bool,
}

pub fn normalize_platform(platform: &str) -> Option<&'static str> {
    match platform.trim().to_lowercase().as_str() {
        "xiaomi" => Some("xiaomi"),
        "huawei" => Some("huawei"),
        "oppo" => Some("oppo"),
        "vivo" => Some("vivo"),
        "honor" => Some("honor"),
        "meizu" => Some("meizu"),
        _ => None,
    }
}
