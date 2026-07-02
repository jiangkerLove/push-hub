use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;

use crate::db::{
    NewPushJob, NewPushJobEvent, NewPushJobTarget, PushTraceRepository, UpdatePushJobTarget,
};
use crate::models::push_trace::{
    DailyStat, DeviceStatsOverview, PushJob, PushJobDetail, PushJobEvent, PushJobMessageTrace,
    PushJobTarget, PushOutboxTrace, PushPlatformStat, PushStatsOverview,
};
use crate::AppResult;

pub async fn create_repository(pool: PgPool) -> AppResult<Arc<dyn PushTraceRepository>> {
    Ok(Arc::new(PgPushTraceRepository { pool }))
}

struct PgPushTraceRepository {
    pool: PgPool,
}

const JOB_COLUMNS: &str = "id, app_id, template_id, template_name, title, body, total_targets, success_count, failed_count, batch_id, created_at";
const TARGET_COLUMNS: &str = "id, job_id, device_id, platform, push_token, route_decision, final_status, final_channel, outbox_id, vendor_message_id, created_at";

#[async_trait]
impl PushTraceRepository for PgPushTraceRepository {
    async fn create_job(&self, job: NewPushJob) -> AppResult<PushJob> {
        sqlx::query(
            r#"
            INSERT INTO push_jobs (id, app_id, template_id, template_name, title, body, total_targets)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(&job.id)
        .bind(&job.app_id)
        .bind(&job.template_id)
        .bind(&job.template_name)
        .bind(&job.title)
        .bind(&job.body)
        .bind(job.total_targets)
        .execute(&self.pool)
        .await?;

        self.get_job(&job.id).await
    }

    async fn finish_job(&self, job_id: &str, success: i64, failed: i64) -> AppResult<()> {
        sqlx::query("UPDATE push_jobs SET success_count = $1, failed_count = $2 WHERE id = $3")
            .bind(success)
            .bind(failed)
            .bind(job_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn set_job_batch_id(&self, job_id: &str, batch_id: &str) -> AppResult<()> {
        sqlx::query("UPDATE push_jobs SET batch_id = $1 WHERE id = $2")
            .bind(batch_id)
            .bind(job_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn create_target(&self, target: NewPushJobTarget) -> AppResult<PushJobTarget> {
        sqlx::query(
            r#"
            INSERT INTO push_job_targets (id, job_id, device_id, platform, push_token)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(&target.id)
        .bind(&target.job_id)
        .bind(&target.device_id)
        .bind(&target.platform)
        .bind(&target.push_token)
        .execute(&self.pool)
        .await?;

        self.get_target(&target.id).await
    }

    async fn update_target(&self, target_id: &str, update: UpdatePushJobTarget) -> AppResult<()> {
        sqlx::query("UPDATE push_job_targets SET route_decision = $1 WHERE id = $2")
            .bind(update.route_decision)
            .bind(target_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn set_target_outbox(&self, target_id: &str, outbox_id: &str) -> AppResult<()> {
        sqlx::query("UPDATE push_job_targets SET outbox_id = $1 WHERE id = $2")
            .bind(outbox_id)
            .bind(target_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn set_target_vendor_message(
        &self,
        target_id: &str,
        vendor_message_id: &str,
    ) -> AppResult<()> {
        sqlx::query("UPDATE push_job_targets SET vendor_message_id = $1 WHERE id = $2")
            .bind(vendor_message_id)
            .bind(target_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn finish_target(
        &self,
        target_id: &str,
        status: &str,
        channel: Option<&str>,
    ) -> AppResult<()> {
        sqlx::query(
            "UPDATE push_job_targets SET final_status = $1, final_channel = $2 WHERE id = $3",
        )
        .bind(status)
        .bind(channel)
        .bind(target_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn add_event(&self, event: NewPushJobEvent) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO push_job_events (id, job_id, target_id, stage, status, platform, detail, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(&event.id)
        .bind(&event.job_id)
        .bind(&event.target_id)
        .bind(&event.stage)
        .bind(&event.status)
        .bind(&event.platform)
        .bind(&event.detail)
        .bind(&event.metadata)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn link_outbox(
        &self,
        outbox_id: &str,
        job_id: &str,
        target_id: &str,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO push_outbox_trace (outbox_id, job_id, target_id) VALUES ($1, $2, $3)
            ON CONFLICT (outbox_id) DO UPDATE SET
                job_id = EXCLUDED.job_id,
                target_id = EXCLUDED.target_id
            "#,
        )
        .bind(outbox_id)
        .bind(job_id)
        .bind(target_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_job_id_by_outbox(&self, outbox_id: &str) -> AppResult<Option<String>> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT job_id FROM push_outbox_trace WHERE outbox_id = $1")
                .bind(outbox_id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(row.map(|r| r.0))
    }

    async fn find_target_id_by_outbox(&self, outbox_id: &str) -> AppResult<Option<String>> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT target_id FROM push_outbox_trace WHERE outbox_id = $1")
                .bind(outbox_id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(row.map(|r| r.0))
    }

    async fn list_jobs(&self, app_id: &str, limit: i64, offset: i64) -> AppResult<Vec<PushJob>> {
        let query = format!(
            "SELECT {JOB_COLUMNS} FROM push_jobs WHERE app_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        );
        Ok(sqlx::query_as::<_, PushJob>(&query)
            .bind(app_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?)
    }

    async fn get_job_detail(&self, job_id: &str) -> AppResult<Option<PushJobDetail>> {
        let job = match self.get_job(job_id).await {
            Ok(job) => job,
            Err(_) => return Ok(None),
        };

        let targets = sqlx::query_as::<_, PushJobTarget>(&format!(
            "SELECT {TARGET_COLUMNS} FROM push_job_targets WHERE job_id = $1 ORDER BY created_at"
        ))
        .bind(job_id)
        .fetch_all(&self.pool)
        .await?;

        let events = sqlx::query_as::<_, PushJobEvent>(
            "SELECT id, job_id, target_id, stage, status, platform, detail, metadata, created_at FROM push_job_events WHERE job_id = $1 ORDER BY created_at",
        )
        .bind(job_id)
        .fetch_all(&self.pool)
        .await?;

        let outbox_by_id = if let Some(batch_id) = job.batch_id.as_ref() {
            sqlx::query_as::<_, PushOutboxTraceRow>(
                r#"
                SELECT
                    id,
                    push_token,
                    to_char(delivered_at AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI:SS') AS delivered_at,
                    to_char(fallback_sent_at AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI:SS') AS fallback_sent_at,
                    fallback_platform,
                    to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI:SS') AS created_at
                FROM push_outbox WHERE batch_id = $1 ORDER BY created_at
                "#,
            )
            .bind(batch_id)
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|row| (row.id.clone(), row.into_trace()))
            .collect::<HashMap<_, _>>()
        } else {
            HashMap::new()
        };

        let job_events = events
            .iter()
            .filter(|event| event.target_id.is_none())
            .cloned()
            .collect();

        let messages = targets
            .into_iter()
            .map(|target| {
                let message_events = events
                    .iter()
                    .filter(|event| event.target_id.as_deref() == Some(target.id.as_str()))
                    .cloned()
                    .collect();
                let outbox = target
                    .outbox_id
                    .as_ref()
                    .and_then(|id| outbox_by_id.get(id).cloned());
                PushJobMessageTrace {
                    target,
                    events: message_events,
                    outbox,
                }
            })
            .collect();

        Ok(Some(PushJobDetail {
            job,
            job_events,
            messages,
        }))
    }

    async fn stats(&self, app_id: &str, days: i64) -> AppResult<PushStatsOverview> {
        let days = days.clamp(1, 90);
        let since = Utc::now() - chrono::Duration::days(days);

        let summary: (i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT COUNT(*)::BIGINT, COALESCE(SUM(success_count), 0)::BIGINT, COALESCE(SUM(failed_count), 0)::BIGINT
            FROM push_jobs WHERE app_id = $1 AND created_at >= $2
            "#,
        )
        .bind(app_id)
        .bind(since)
        .fetch_one(&self.pool)
        .await?;

        let by_platform = sqlx::query_as::<_, (String, i64, i64)>(
            r#"
            SELECT
                COALESCE(t.final_channel, t.platform) AS platform,
                SUM(CASE WHEN t.final_status = 'ok' THEN 1 ELSE 0 END)::BIGINT AS success,
                SUM(CASE WHEN t.final_status != 'ok' THEN 1 ELSE 0 END)::BIGINT AS failed
            FROM push_job_targets t
            JOIN push_jobs j ON j.id = t.job_id
            WHERE j.app_id = $1 AND j.created_at >= $2
            GROUP BY COALESCE(t.final_channel, t.platform)
            ORDER BY COUNT(*) DESC
            "#,
        )
        .bind(app_id)
        .bind(since)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|(platform, success, failed)| PushPlatformStat {
            platform,
            success,
            failed,
        })
        .collect();

        let daily = sqlx::query_as::<_, (String, i64, i64, i64)>(
            r#"
            SELECT
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD') AS day,
                COUNT(*)::BIGINT,
                COALESCE(SUM(success_count), 0)::BIGINT,
                COALESCE(SUM(failed_count), 0)::BIGINT
            FROM push_jobs WHERE app_id = $1 AND created_at >= $2
            GROUP BY to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD')
            ORDER BY day DESC
            "#,
        )
        .bind(app_id)
        .bind(since)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|(date, jobs, success, failed)| DailyStat {
            date,
            jobs,
            success,
            failed,
        })
        .collect();

        let total_targets = summary.1 + summary.2;
        let success_rate = if total_targets > 0 {
            summary.1 as f64 / total_targets as f64
        } else {
            0.0
        };

        Ok(PushStatsOverview {
            days,
            total_jobs: summary.0,
            total_targets,
            success_targets: summary.1,
            failed_targets: summary.2,
            success_rate,
            push_by_platform: by_platform,
            daily,
            devices: DeviceStatsOverview {
                total: 0,
                recent_online: 0,
                new_in_period: 0,
                by_platform: Vec::new(),
            },
            template_count: 0,
        })
    }
}

impl PgPushTraceRepository {
    async fn get_job(&self, job_id: &str) -> AppResult<PushJob> {
        let query = format!("SELECT {JOB_COLUMNS} FROM push_jobs WHERE id = $1");
        Ok(sqlx::query_as::<_, PushJob>(&query)
            .bind(job_id)
            .fetch_one(&self.pool)
            .await?)
    }

    async fn get_target(&self, target_id: &str) -> AppResult<PushJobTarget> {
        let query = format!("SELECT {TARGET_COLUMNS} FROM push_job_targets WHERE id = $1");
        Ok(sqlx::query_as::<_, PushJobTarget>(&query)
            .bind(target_id)
            .fetch_one(&self.pool)
            .await?)
    }
}

#[derive(Debug, sqlx::FromRow)]
struct PushOutboxTraceRow {
    id: String,
    push_token: String,
    delivered_at: Option<String>,
    fallback_sent_at: Option<String>,
    fallback_platform: Option<String>,
    created_at: String,
}

impl PushOutboxTraceRow {
    fn into_trace(self) -> PushOutboxTrace {
        PushOutboxTrace {
            id: self.id,
            push_token: self.push_token,
            delivered_at: self.delivered_at,
            fallback_sent_at: self.fallback_sent_at,
            fallback_platform: self.fallback_platform,
            created_at: self.created_at,
        }
    }
}
