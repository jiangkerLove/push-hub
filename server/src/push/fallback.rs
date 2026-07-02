use crate::db::Database;
use crate::models::{DeliveryMode, OutboxFallbackJob, RenderedNotification};
use crate::push::{ProviderSendResult, PushHub};
use crate::AppResult;

#[derive(Debug, Clone, Default)]
pub struct FallbackBatchResult {
    pub success_count: usize,
    pub failure_count: usize,
    pub last_error: Option<String>,
    pub last_platform: Option<String>,
    pub last_vendor_message_id: Option<String>,
}

/// WebSocket 未送达时，立即对 outbox 消息执行厂商降级。
pub async fn fallback_outbox_ids(
    db: &Database,
    hub: &PushHub,
    outbox_ids: &[String],
) -> AppResult<FallbackBatchResult> {
    let jobs = db.outbox().find_fallback_jobs_by_ids(outbox_ids).await?;
    if jobs.is_empty() {
        return Ok(FallbackBatchResult::default());
    }

    let mut result = FallbackBatchResult::default();
    for job in jobs {
        match send_vendor_fallback(hub, &job).await {
            Ok(provider_result) => {
                let outbox_id = job.id.clone();
                let platform = job.fallback_platform.clone();
                let vendor_message_id = provider_result.message_id.clone();
                db.outbox().mark_fallback_sent(&[job.id.clone()]).await?;
                let _ = db.outbox().mark_delivered(&[job.id]).await;
                crate::push::trace::record_vendor_fallback(
                    db,
                    &outbox_id,
                    &platform,
                    true,
                    Some("WebSocket 未送达，已立即降级厂商离线推送"),
                    vendor_message_id.as_deref(),
                )
                .await?;
                result.success_count += 1;
                result.last_platform = Some(platform);
                result.last_vendor_message_id = vendor_message_id;
                tracing::info!(
                    outbox_id = %outbox_id,
                    platform = %result.last_platform.as_deref().unwrap_or(""),
                    vendor_message_id = ?result.last_vendor_message_id,
                    "websocket undelivered, immediate vendor fallback"
                );
            }
            Err(err) => {
                let error = err.to_string();
                crate::push::trace::record_vendor_fallback(
                    db,
                    &job.id,
                    &job.fallback_platform,
                    false,
                    Some(&error),
                    None,
                )
                .await?;
                result.failure_count += 1;
                result.last_error = Some(error.clone());
                result.last_platform = Some(job.fallback_platform.clone());
                tracing::warn!(
                    outbox_id = %job.id,
                    platform = %job.fallback_platform,
                    error = %error,
                    "immediate vendor fallback failed"
                );
            }
        }
    }

    Ok(result)
}

pub async fn send_vendor_fallback(
    hub: &PushHub,
    job: &OutboxFallbackJob,
) -> AppResult<ProviderSendResult> {
    let provider = hub
        .get(&job.fallback_platform)
        .ok_or_else(|| {
            crate::AppError::Push(format!(
                "fallback platform '{}' is not registered",
                job.fallback_platform
            ))
        })?;

    let rendered = RenderedNotification {
        title: job.title.clone(),
        body: job.body.clone(),
        payload: job.payload.clone(),
        click_action: job.click_action.clone(),
        package_name: job.package_name.clone(),
        channels: job.channels.clone(),
        delivery_mode: DeliveryMode::Notification,
        notify_id: job.notify_id,
        vendor_fallback: None,
        expires_at: chrono::Utc::now(),
        title_variables: job.title_variables.clone(),
        body_variables: job.body_variables.clone(),
    };

    provider
        .send(std::slice::from_ref(&job.fallback_token), &rendered)
        .await
}
