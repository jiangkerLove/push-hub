use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::types::Json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::OutboxRepository;
use crate::models::{
    ClickAction, DeliveryMode, EnqueuedMessage, EnqueueResult, OutboxFallbackJob, OutboxMessage,
    RenderedNotification,
};
use crate::AppResult;

pub async fn create_repository(pool: PgPool) -> AppResult<Arc<dyn OutboxRepository>> {
    // 兼容存量库：厂商降级需要保留模板变量（OPPO 私信参数）。
    sqlx::query(
        "ALTER TABLE push_outbox ADD COLUMN IF NOT EXISTS template_vars_json JSONB",
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE push_outbox ADD COLUMN IF NOT EXISTS notify_id INTEGER")
        .execute(&pool)
        .await?;
    Ok(Arc::new(PgOutboxRepository { pool }))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
struct TemplateVarsJson {
    #[serde(default)]
    title: std::collections::HashMap<String, String>,
    #[serde(default)]
    body: std::collections::HashMap<String, String>,
}

struct PgOutboxRepository {
    pool: PgPool,
}

fn format_ts(ts: DateTime<Utc>) -> String {
    ts.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[async_trait]
impl OutboxRepository for PgOutboxRepository {
    async fn enqueue(
        &self,
        push_tokens: &[String],
        notification: &RenderedNotification,
    ) -> AppResult<EnqueueResult> {
        let batch_id = Uuid::new_v4().to_string();
        let delivery_mode = notification.delivery_mode.as_str();
        let (fallback_platform, fallback_token) = notification
            .vendor_fallback
            .as_ref()
            .map(|f| (Some(f.platform.as_str()), Some(f.push_token.as_str())))
            .unwrap_or((None, None));

        let created_at = Utc::now();
        let mut messages = Vec::with_capacity(push_tokens.len());

        for token in push_tokens {
            let row_id = Uuid::new_v4().to_string();
            sqlx::query(
                r#"
                INSERT INTO push_outbox (
                    id, batch_id, push_token, package_name, title, body, payload,
                    delivery_mode, channels_json, click_action_json, template_vars_json,
                    notify_id, fallback_platform, fallback_token, expires_at, created_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
                "#,
            )
            .bind(&row_id)
            .bind(&batch_id)
            .bind(token)
            .bind(&notification.package_name)
            .bind(&notification.title)
            .bind(&notification.body)
            .bind(Json(&notification.payload))
            .bind(delivery_mode)
            .bind(Json(&notification.channels))
            .bind(Json(&notification.click_action))
            .bind(Json(&TemplateVarsJson {
                title: notification.title_variables.clone(),
                body: notification.body_variables.clone(),
            }))
            .bind(notification.notify_id)
            .bind(fallback_platform)
            .bind(fallback_token)
            .bind(notification.expires_at)
            .bind(created_at)
            .execute(&self.pool)
            .await?;

            messages.push(EnqueuedMessage {
                push_token: token.clone(),
                message: OutboxMessage {
                    id: row_id,
                    title: notification.title.clone(),
                    body: notification.body.clone(),
                    payload: notification.payload.clone(),
                    delivery_mode: notification.delivery_mode,
                    click_action: notification.click_action.clone(),
                    notify_id: notification.notify_id,
                    created_at: format_ts(created_at),
                },
            });
        }

        Ok(EnqueueResult { batch_id, messages })
    }

    async fn fetch_pending(
        &self,
        push_token: &str,
        limit: i64,
    ) -> AppResult<Vec<OutboxMessage>> {
        let limit = limit.clamp(1, 100);
        let rows = sqlx::query_as::<_, OutboxRow>(
            r#"
            SELECT id, title, body, payload, delivery_mode, click_action_json, notify_id, created_at
            FROM push_outbox
            WHERE push_token = $1
              AND delivered_at IS NULL
              AND (expires_at IS NULL OR expires_at > NOW())
            ORDER BY created_at ASC
            LIMIT $2
            "#,
        )
        .bind(push_token.trim())
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(OutboxRow::into_message).collect()
    }

    async fn ack(&self, push_token: &str, ids: &[String]) -> AppResult<usize> {
        if ids.is_empty() {
            return Ok(0);
        }

        let placeholders = super::placeholders(2, ids.len());
        let query = format!(
            r#"
            UPDATE push_outbox
            SET delivered_at = NOW()
            WHERE push_token = $1 AND delivered_at IS NULL AND id IN ({placeholders})
            "#
        );

        let mut sql = sqlx::query(&query).bind(push_token.trim());
        for id in ids {
            sql = sql.bind(id);
        }

        let result = sql.execute(&self.pool).await?;
        Ok(result.rows_affected() as usize)
    }

    async fn list_stale_fallbacks(&self, older_than_secs: i64) -> AppResult<Vec<OutboxFallbackJob>> {
        let secs = older_than_secs.max(1);
        let cutoff = Utc::now() - chrono::Duration::seconds(secs);
        let rows = sqlx::query_as::<_, OutboxFallbackRow>(
            r#"
            SELECT id, package_name, title, body, payload, delivery_mode,
                   fallback_platform, fallback_token, channels_json, click_action_json,
                   template_vars_json, notify_id
            FROM push_outbox
            WHERE delivered_at IS NULL
              AND fallback_sent_at IS NULL
              AND fallback_platform IS NOT NULL
              AND fallback_token IS NOT NULL
              AND created_at <= $1
            ORDER BY created_at ASC
            LIMIT 100
            "#,
        )
        .bind(cutoff)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(OutboxFallbackRow::into_job).collect()
    }

    async fn mark_fallback_sent(&self, ids: &[String]) -> AppResult<usize> {
        if ids.is_empty() {
            return Ok(0);
        }
        let placeholders = super::placeholders(1, ids.len());
        let query = format!(
            "UPDATE push_outbox SET fallback_sent_at = NOW() WHERE id IN ({placeholders})"
        );
        let mut sql = sqlx::query(&query);
        for id in ids {
            sql = sql.bind(id);
        }
        Ok(sql.execute(&self.pool).await?.rows_affected() as usize)
    }

    async fn find_fallback_jobs_by_ids(&self, ids: &[String]) -> AppResult<Vec<OutboxFallbackJob>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders = super::placeholders(1, ids.len());
        let query = format!(
            r#"
            SELECT id, package_name, title, body, payload, delivery_mode,
                   fallback_platform, fallback_token, channels_json, click_action_json,
                   template_vars_json, notify_id
            FROM push_outbox
            WHERE id IN ({placeholders})
              AND fallback_platform IS NOT NULL
              AND fallback_token IS NOT NULL
              AND fallback_sent_at IS NULL
            "#
        );
        let mut sql = sqlx::query_as::<_, OutboxFallbackRow>(&query);
        for id in ids {
            sql = sql.bind(id);
        }
        let rows = sql.fetch_all(&self.pool).await?;
        rows.into_iter().map(OutboxFallbackRow::into_job).collect()
    }

    async fn clear_fallback_targets(&self, ids: &[String]) -> AppResult<()> {
        if ids.is_empty() {
            return Ok(());
        }
        let placeholders = super::placeholders(1, ids.len());
        let query = format!(
            "UPDATE push_outbox SET fallback_platform = NULL, fallback_token = NULL WHERE id IN ({placeholders})"
        );
        let mut sql = sqlx::query(&query);
        for id in ids {
            sql = sql.bind(id);
        }
        sql.execute(&self.pool).await?;
        Ok(())
    }

    async fn mark_delivered(&self, ids: &[String]) -> AppResult<usize> {
        if ids.is_empty() {
            return Ok(0);
        }
        let placeholders = super::placeholders(1, ids.len());
        let query = format!(
            "UPDATE push_outbox SET delivered_at = NOW() WHERE id IN ({placeholders}) AND delivered_at IS NULL"
        );
        let mut sql = sqlx::query(&query);
        for id in ids {
            sql = sql.bind(id);
        }
        Ok(sql.execute(&self.pool).await?.rows_affected() as usize)
    }
}

#[derive(Debug, sqlx::FromRow)]
struct OutboxRow {
    id: String,
    title: String,
    body: String,
    payload: Json<Value>,
    delivery_mode: String,
    click_action_json: Option<Json<ClickAction>>,
    notify_id: Option<i32>,
    created_at: DateTime<Utc>,
}

impl OutboxRow {
    fn into_message(self) -> AppResult<OutboxMessage> {
        let delivery_mode =
            DeliveryMode::parse(&self.delivery_mode).unwrap_or(DeliveryMode::Notification);
        let click_action = self
            .click_action_json
            .map(|j| j.0)
            .unwrap_or_default();
        Ok(OutboxMessage {
            id: self.id,
            title: self.title,
            body: self.body,
            payload: self.payload.0,
            delivery_mode,
            click_action,
            notify_id: self.notify_id,
            created_at: format_ts(self.created_at),
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
struct OutboxFallbackRow {
    id: String,
    package_name: String,
    title: String,
    body: String,
    payload: Json<Value>,
    delivery_mode: String,
    fallback_platform: String,
    fallback_token: String,
    channels_json: Option<Json<crate::models::TemplateChannels>>,
    click_action_json: Option<Json<ClickAction>>,
    template_vars_json: Option<Json<TemplateVarsJson>>,
    notify_id: Option<i32>,
}

impl OutboxFallbackRow {
    fn into_job(self) -> AppResult<OutboxFallbackJob> {
        let delivery_mode =
            DeliveryMode::parse(&self.delivery_mode).unwrap_or(DeliveryMode::Notification);
        let channels = self.channels_json.map(|j| j.0).unwrap_or_default();
        let click_action = self
            .click_action_json
            .map(|j| j.0)
            .unwrap_or_default();
        let vars = self.template_vars_json.map(|j| j.0).unwrap_or_default();
        Ok(OutboxFallbackJob {
            id: self.id,
            package_name: self.package_name,
            title: self.title,
            body: self.body,
            payload: self.payload.0,
            delivery_mode,
            fallback_platform: self.fallback_platform,
            fallback_token: self.fallback_token,
            channels,
            click_action,
            title_variables: vars.title,
            body_variables: vars.body,
            notify_id: self.notify_id,
        })
    }
}
