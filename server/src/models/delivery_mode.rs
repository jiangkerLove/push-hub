use serde::{Deserialize, Serialize};

/// 推送投递模式：通知栏展示或透传至 App。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryMode {
    /// 通知栏消息（默认）
    #[default]
    Notification,
    /// 透传消息，不展示系统通知栏，由 App 自行处理
    Data,
}

impl DeliveryMode {
    pub fn is_pass_through(self) -> bool {
        matches!(self, DeliveryMode::Data)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            DeliveryMode::Notification => "notification",
            DeliveryMode::Data => "data",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "notification" => Some(DeliveryMode::Notification),
            "data" => Some(DeliveryMode::Data),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_delivery_mode() {
        assert_eq!(DeliveryMode::parse("data"), Some(DeliveryMode::Data));
        assert_eq!(
            DeliveryMode::parse("notification"),
            Some(DeliveryMode::Notification)
        );
        assert_eq!(DeliveryMode::parse("unknown"), None);
    }
}
