use std::sync::Arc;
use std::time::Duration;

use crate::db::Database;
use crate::push::fallback::send_vendor_fallback;
use crate::push::PushHubManager;
use crate::AppResult;

const WORKER_INTERVAL_SECS: u64 = 15;

pub fn spawn(db: Arc<Database>, hub_manager: Arc<PushHubManager>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(WORKER_INTERVAL_SECS));
        interval.tick().await;
        loop {
            interval.tick().await;
            if let Err(err) = process_stale_fallbacks(db.as_ref(), hub_manager.as_ref()).await {
                tracing::warn!(error = %err, "stale vendor fallback worker failed");
            }
        }
    });
}

async fn process_stale_fallbacks(db: &Database, hub_manager: &PushHubManager) -> AppResult<()> {
    let apps = db.apps().list().await?;
    for app in apps {
        if app.package_name.trim().is_empty() {
            continue;
        }
        let secs = app.online_push_fallback_secs.max(1);
        let jobs = db.outbox().list_stale_fallbacks(secs).await?;
        if jobs.is_empty() {
            continue;
        }

        let hub = hub_manager.hub_for_app(&app)?;
        let package_name = app.package_name.trim();
        for job in jobs {
            if job.package_name.trim() != package_name {
                continue;
            }

            let job_id = job.id.clone();
            match send_vendor_fallback(hub.as_ref(), &job).await {
                Ok(provider_result) => {
                    let _ = db.outbox().mark_fallback_sent(&[job_id.clone()]).await;
                    let _ = db.outbox().mark_delivered(&[job_id.clone()]).await;
                    let _ = crate::push::trace::record_vendor_fallback(
                        db,
                        &job_id,
                        &job.fallback_platform,
                        true,
                        Some("在线消息超时未 ACK，已降级厂商离线推送"),
                        provider_result.message_id.as_deref(),
                    )
                    .await;
                    tracing::info!(
                        outbox_id = %job_id,
                        platform = %job.fallback_platform,
                        vendor_message_id = ?provider_result.message_id,
                        "stale online message downgraded to vendor push"
                    );
                }
                Err(err) => {
                    let _ = crate::push::trace::record_vendor_fallback(
                        db,
                        &job_id,
                        &job.fallback_platform,
                        false,
                        Some(&err.to_string()),
                        None,
                    )
                    .await;
                    tracing::warn!(
                        outbox_id = %job_id,
                        platform = %job.fallback_platform,
                        error = %err,
                        "stale vendor fallback failed"
                    );
                }
            }
        }
    }

    Ok(())
}
