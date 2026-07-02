use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::models::RenderedNotification;
use crate::AppResult;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderSendResult {
    pub success_count: usize,
    pub failure_count: usize,
    pub message_id: Option<String>,
    #[serde(default)]
    pub outbox_ids: Vec<String>,
    #[serde(default)]
    pub ws_delivered: usize,
}

#[async_trait]
pub trait PushProvider: Send + Sync {
    fn platform(&self) -> &'static str;
    async fn send(
        &self,
        push_tokens: &[String],
        notification: &RenderedNotification,
    ) -> AppResult<ProviderSendResult>;
}

pub mod api_auth;
pub mod credentials;
pub mod fallback;
pub mod fallback_worker;
pub mod hub;
pub mod hub_manager;
pub mod init_snippet;
pub mod online;
pub mod online_hub;
pub mod service;
pub mod template_render;
pub mod trace;
pub mod vendors;

pub use hub::PushHub;
pub use hub_manager::PushHubManager;
pub use online::OnlinePushProvider;
pub use online_hub::OnlinePushHub;
pub use service::PushService;
pub use vendors::{
    HonorPushProvider, HuaweiPushProvider, MeizuPushProvider, OppoPushProvider, VivoPushProvider,
    XiaomiPushProvider,
};
