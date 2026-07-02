use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PushJob {
    pub id: String,
    pub app_id: String,
    pub template_id: String,
    pub template_name: String,
    pub title: String,
    pub body: String,
    pub total_targets: i64,
    pub success_count: i64,
    pub failed_count: i64,
    pub batch_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PushJobTarget {
    pub id: String,
    pub job_id: String,
    pub device_id: Option<String>,
    pub platform: String,
    pub push_token: String,
    pub route_decision: String,
    pub final_status: String,
    pub final_channel: Option<String>,
    pub outbox_id: Option<String>,
    pub vendor_message_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PushJobEvent {
    pub id: String,
    pub job_id: String,
    pub target_id: Option<String>,
    pub stage: String,
    pub status: String,
    pub platform: Option<String>,
    pub detail: String,
    pub metadata: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PushJobMessageTrace {
    pub target: PushJobTarget,
    pub events: Vec<PushJobEvent>,
    pub outbox: Option<PushOutboxTrace>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PushJobDetail {
    pub job: PushJob,
    pub job_events: Vec<PushJobEvent>,
    pub messages: Vec<PushJobMessageTrace>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct PushOutboxTrace {
    pub id: String,
    pub push_token: String,
    pub delivered_at: Option<String>,
    pub fallback_sent_at: Option<String>,
    pub fallback_platform: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PushStatsOverview {
    pub days: i64,
    pub total_jobs: i64,
    pub total_targets: i64,
    pub success_targets: i64,
    pub failed_targets: i64,
    pub success_rate: f64,
    pub push_by_platform: Vec<PushPlatformStat>,
    pub daily: Vec<DailyStat>,
    pub devices: DeviceStatsOverview,
    pub template_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PushPlatformStat {
    pub platform: String,
    pub success: i64,
    pub failed: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeviceStatsOverview {
    pub total: i64,
    pub recent_online: i64,
    pub new_in_period: i64,
    pub by_platform: Vec<PlatformStat>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlatformStat {
    pub platform: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DailyStat {
    pub date: String,
    pub jobs: i64,
    pub success: i64,
    pub failed: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppInitSnippet {
    pub server_base_url: String,
    pub package_name: String,
    pub push_api_key: String,
    pub kotlin: String,
    pub push_properties: String,
    pub manifest_placeholders: Value,
    pub manifest_placeholders_kotlin: String,
}

pub mod stages {
    pub const RECEIVED: &str = "received";
    pub const TEMPLATE_RENDERED: &str = "template_rendered";
    pub const ROUTE_SELECTED: &str = "route_selected";
    pub const ONLINE_ENQUEUE: &str = "online_enqueue";
    pub const ONLINE_WS: &str = "online_ws";
    pub const VENDOR_SEND: &str = "vendor_send";
    pub const VENDOR_FALLBACK: &str = "vendor_fallback";
    pub const ONLINE_ACK: &str = "online_ack";
    pub const CLIENT_DISPLAY: &str = "client_display";
}

pub mod statuses {
    pub const OK: &str = "ok";
    pub const FAILED: &str = "failed";
    pub const SKIPPED: &str = "skipped";
    pub const PENDING: &str = "pending";
}
