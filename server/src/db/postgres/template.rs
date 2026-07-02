use std::sync::Arc;

use async_trait::async_trait;
use sqlx::types::Json;
use sqlx::PgPool;

use crate::db::{NewTemplate, TemplateRepository, UpdateTemplate};
use crate::models::PushTemplate;
use crate::AppResult;

pub async fn create_repository(pool: PgPool) -> AppResult<Arc<dyn TemplateRepository>> {
    Ok(Arc::new(PgTemplateRepository { pool }))
}

struct PgTemplateRepository {
    pool: PgPool,
}

const TEMPLATE_COLUMNS: &str =
    "id, app_id, name, kind, content_mode, title, body, channels, click_action, message_cache_days::BIGINT AS message_cache_days, created_at, updated_at";

#[async_trait]
impl TemplateRepository for PgTemplateRepository {
    async fn create(&self, template: NewTemplate) -> AppResult<PushTemplate> {
        sqlx::query(
            r#"
            INSERT INTO push_templates (id, app_id, name, kind, content_mode, title, body, channels, click_action, message_cache_days)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(&template.id)
        .bind(&template.app_id)
        .bind(&template.name)
        .bind(&template.kind)
        .bind(&template.content_mode)
        .bind(&template.title)
        .bind(&template.body)
        .bind(Json(&template.channels))
        .bind(Json(&template.click_action))
        .bind(template.message_cache_days.max(1))
        .execute(&self.pool)
        .await?;

        self.find_by_id(&template.id)
            .await?
            .ok_or_else(|| crate::AppError::Database(sqlx::Error::RowNotFound))
    }

    async fn update(&self, id: &str, template: UpdateTemplate) -> AppResult<PushTemplate> {
        let result = sqlx::query(
            r#"
            UPDATE push_templates SET
                name = $1,
                kind = $2,
                content_mode = $3,
                title = $4,
                body = $5,
                channels = $6,
                click_action = $7,
                message_cache_days = $8,
                updated_at = NOW()
            WHERE id = $9
            "#,
        )
        .bind(&template.name)
        .bind(&template.kind)
        .bind(&template.content_mode)
        .bind(&template.title)
        .bind(&template.body)
        .bind(Json(&template.channels))
        .bind(Json(&template.click_action))
        .bind(template.message_cache_days.max(1))
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(crate::AppError::NotFound(format!("template not found: {id}")));
        }

        self.find_by_id(id)
            .await?
            .ok_or_else(|| crate::AppError::Database(sqlx::Error::RowNotFound))
    }

    async fn find_by_id(&self, id: &str) -> AppResult<Option<PushTemplate>> {
        let query = format!("SELECT {TEMPLATE_COLUMNS} FROM push_templates WHERE id = $1");
        Ok(sqlx::query_as::<_, PushTemplate>(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?)
    }

    async fn list(&self) -> AppResult<Vec<PushTemplate>> {
        let query =
            format!("SELECT {TEMPLATE_COLUMNS} FROM push_templates ORDER BY updated_at DESC");
        Ok(sqlx::query_as::<_, PushTemplate>(&query)
            .fetch_all(&self.pool)
            .await?)
    }

    async fn list_by_app_id(&self, app_id: &str) -> AppResult<Vec<PushTemplate>> {
        let query = format!(
            "SELECT {TEMPLATE_COLUMNS} FROM push_templates WHERE app_id = $1 ORDER BY updated_at DESC"
        );
        Ok(sqlx::query_as::<_, PushTemplate>(&query)
            .bind(app_id)
            .fetch_all(&self.pool)
            .await?)
    }

    async fn delete(&self, id: &str) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM push_templates WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(crate::AppError::NotFound(format!("template not found: {id}")));
        }
        Ok(())
    }
}
