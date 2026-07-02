use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod admin;
pub mod click_action;
pub mod delivery_mode;
pub mod device;
pub mod device_id;
pub mod platform;
pub mod push;
pub mod push_trace;
pub mod channel;
pub mod template;
pub mod user;

pub use admin::*;
pub use click_action::*;
pub use delivery_mode::*;
pub use device::RegisterDeviceRequest;
pub use device_id::*;
pub use platform::*;
pub use push::*;
pub use push_trace::*;
pub use channel::*;
pub use template::*;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Device {
    pub id: String,
    pub app_id: String,
    pub package_name: String,
    pub platform: String,
    pub push_token: String,
    pub online_token: Option<String>,
    pub last_online_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Device {
    /// 设备是否在 TTL 内有过在线心跳（轮询拉取消息即视为在线）
    pub fn is_online(&self, ttl_secs: i64) -> bool {
        let Some(at) = self.last_online_at else {
            return false;
        };
        Utc::now().signed_duration_since(at).num_seconds() < ttl_secs
    }
}
