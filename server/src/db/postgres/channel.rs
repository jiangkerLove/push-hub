use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::db::{ChannelRepository, NewPushChannel, UpdatePushChannel};
use crate::models::PushChannel;
use crate::AppResult;

pub async fn create_repository(pool: PgPool) -> AppResult<Arc<dyn ChannelRepository>> {
    Ok(Arc::new(PgChannelRepository { pool }))
}

struct PgChannelRepository {
    pool: PgPool,
}

const CHANNEL_COLUMNS: &str =
    "id, app_id, platform, name, code, description, is_default, created_at, updated_at";

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

#[async_trait]
impl ChannelRepository for PgChannelRepository {
    async fn create(&self, channel: NewPushChannel) -> AppResult<PushChannel> {
        if channel.is_default {
            self.clear_default(&channel.app_id, &channel.platform).await?;
        }

        sqlx::query(
            r#"
            INSERT INTO push_channels (id, app_id, platform, name, code, description, is_default)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(&channel.id)
        .bind(&channel.app_id)
        .bind(&channel.platform)
        .bind(&channel.name)
        .bind(&channel.code)
        .bind(normalize_optional(channel.description))
        .bind(channel.is_default)
        .execute(&self.pool)
        .await?;

        self.find_by_id(&channel.id)
            .await?
            .ok_or_else(|| crate::AppError::Database(sqlx::Error::RowNotFound))
    }

    async fn update(&self, id: &str, channel: UpdatePushChannel) -> AppResult<PushChannel> {
        let existing = self
            .find_by_id(id)
            .await?
            .ok_or_else(|| crate::AppError::NotFound(format!("channel not found: {id}")))?;

        if channel.is_default {
            self.clear_default(&existing.app_id, &channel.platform)
                .await?;
        }

        let result = sqlx::query(
            r#"
            UPDATE push_channels SET
                platform = $1,
                name = $2,
                code = $3,
                description = $4,
                is_default = $5,
                updated_at = NOW()
            WHERE id = $6
            "#,
        )
        .bind(&channel.platform)
        .bind(&channel.name)
        .bind(&channel.code)
        .bind(normalize_optional(channel.description))
        .bind(channel.is_default)
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(crate::AppError::NotFound(format!("channel not found: {id}")));
        }

        self.find_by_id(id)
            .await?
            .ok_or_else(|| crate::AppError::Database(sqlx::Error::RowNotFound))
    }

    async fn find_by_id(&self, id: &str) -> AppResult<Option<PushChannel>> {
        let query = format!("SELECT {CHANNEL_COLUMNS} FROM push_channels WHERE id = $1");
        Ok(sqlx::query_as::<_, PushChannel>(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?)
    }

    async fn list_by_app_id(&self, app_id: &str) -> AppResult<Vec<PushChannel>> {
        let query = format!(
            "SELECT {CHANNEL_COLUMNS} FROM push_channels WHERE app_id = $1 ORDER BY platform, is_default DESC, name"
        );
        Ok(sqlx::query_as::<_, PushChannel>(&query)
            .bind(app_id)
            .fetch_all(&self.pool)
            .await?)
    }

    async fn delete(&self, id: &str) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM push_channels WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(crate::AppError::NotFound(format!("channel not found: {id}")));
        }
        Ok(())
    }

    async fn clear_default(&self, app_id: &str, platform: &str) -> AppResult<()> {
        sqlx::query(
            "UPDATE push_channels SET is_default = FALSE, updated_at = NOW() WHERE app_id = $1 AND platform = $2",
        )
        .bind(app_id)
        .bind(platform)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
