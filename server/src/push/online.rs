use std::sync::Arc;

use async_trait::async_trait;

use crate::db::OutboxRepository;
use crate::models::{FALLBACK_PLATFORM, RenderedNotification};
use crate::push::online_hub::OnlinePushHub;
use crate::push::{ProviderSendResult, PushProvider};
use crate::AppResult;

/// 在线推送：写入 outbox 并通过 WebSocket 长连接实时下发；HTTP 轮询作兜底。
pub struct OnlinePushProvider {
    outbox: Arc<dyn OutboxRepository>,
    hub: Arc<OnlinePushHub>,
}

impl OnlinePushProvider {
    pub fn new(outbox: Arc<dyn OutboxRepository>, hub: Arc<OnlinePushHub>) -> Self {
        Self { outbox, hub }
    }
}

fn ws_payload(message: &crate::models::OutboxMessage) -> String {
    message.to_online_ws_payload()
}

#[async_trait]
impl PushProvider for OnlinePushProvider {
    fn platform(&self) -> &'static str {
        FALLBACK_PLATFORM
    }

    async fn send(
        &self,
        push_tokens: &[String],
        notification: &RenderedNotification,
    ) -> AppResult<ProviderSendResult> {
        if push_tokens.is_empty() {
            return Ok(ProviderSendResult {
                success_count: 0,
                failure_count: 0,
                message_id: None,
                outbox_ids: vec![],
                ws_delivered: 0,
            });
        }

        let result = self.outbox.enqueue(push_tokens, notification).await?;
        let mut ws_delivered = 0usize;

        for enqueued in &result.messages {
            let payload = ws_payload(&enqueued.message);
            ws_delivered += self.hub.publish(&enqueued.push_token, payload).await;
        }

        tracing::info!(
            platform = FALLBACK_PLATFORM,
            count = push_tokens.len(),
            batch_id = %result.batch_id,
            ws_delivered,
            "queued notification to online outbox"
        );

        Ok(ProviderSendResult {
            success_count: push_tokens.len(),
            failure_count: 0,
            message_id: Some(result.batch_id.clone()),
            outbox_ids: result.messages.iter().map(|m| m.message.id.clone()).collect(),
            ws_delivered,
        })
    }
}
