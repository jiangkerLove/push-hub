use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::db::{AppRepository, NewApp, UpdateApp};
use crate::models::PushApp;
use crate::AppResult;

pub async fn create_repository(pool: PgPool) -> AppResult<Arc<dyn AppRepository>> {
    sqlx::query("ALTER TABLE apps ADD COLUMN IF NOT EXISTS push_api_key TEXT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "UPDATE apps SET push_api_key = 'phk_' || replace(gen_random_uuid()::text, '-', '')
         WHERE push_api_key IS NULL OR push_api_key = ''",
    )
    .execute(&pool)
    .await?;

    Ok(Arc::new(PgAppRepository { pool }))
}

struct PgAppRepository {
    pool: PgPool,
}

const APP_COLUMNS: &str = "id, name, package_name, ios_bundle_id, harmony_bundle_name, description, server_base_url, push_api_key, xiaomi_app_id, xiaomi_app_key, xiaomi_channel_id, xiaomi_app_secret, huawei_app_id, huawei_oauth_client_id, huawei_app_secret, oppo_app_key, oppo_app_secret, oppo_master_secret, vivo_app_id, vivo_app_key, vivo_app_secret, honor_app_id, honor_oauth_client_id, honor_app_secret, meizu_app_id, meizu_app_key, meizu_app_secret, online_push_fallback_secs::BIGINT AS online_push_fallback_secs, online_message_cache_secs::BIGINT AS online_message_cache_secs, is_default, created_at, updated_at";

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

#[async_trait]
impl AppRepository for PgAppRepository {
    async fn create(&self, app: NewApp) -> AppResult<PushApp> {
        let huawei_oauth_client_id = app
            .huawei_oauth_client_id
            .or_else(|| app.huawei_app_id.clone());

        sqlx::query(
            r#"
            INSERT INTO apps (
                id, name, package_name, ios_bundle_id, harmony_bundle_name, description, server_base_url, push_api_key,
                xiaomi_app_id, xiaomi_app_key, xiaomi_channel_id, xiaomi_app_secret,
                huawei_app_id, huawei_oauth_client_id, huawei_app_secret,
                oppo_app_key, oppo_app_secret, oppo_master_secret,
                vivo_app_id, vivo_app_key, vivo_app_secret,
                honor_app_id, honor_oauth_client_id, honor_app_secret,
                meizu_app_id, meizu_app_key, meizu_app_secret,
                online_push_fallback_secs, online_message_cache_secs, is_default
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18,
                $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30
            )
            "#,
        )
        .bind(&app.id)
        .bind(&app.name)
        .bind(&app.package_name)
        .bind(normalize_optional(app.ios_bundle_id))
        .bind(normalize_optional(app.harmony_bundle_name))
        .bind(&app.description)
        .bind(normalize_optional(app.server_base_url))
        .bind(&app.push_api_key)
        .bind(normalize_optional(app.xiaomi_app_id))
        .bind(normalize_optional(app.xiaomi_app_key))
        .bind(normalize_optional(app.xiaomi_channel_id))
        .bind(normalize_optional(app.xiaomi_app_secret))
        .bind(normalize_optional(app.huawei_app_id))
        .bind(normalize_optional(huawei_oauth_client_id))
        .bind(normalize_optional(app.huawei_app_secret))
        .bind(normalize_optional(app.oppo_app_key))
        .bind(normalize_optional(app.oppo_app_secret))
        .bind(normalize_optional(app.oppo_master_secret))
        .bind(normalize_optional(app.vivo_app_id))
        .bind(normalize_optional(app.vivo_app_key))
        .bind(normalize_optional(app.vivo_app_secret))
        .bind(normalize_optional(app.honor_app_id))
        .bind(normalize_optional(app.honor_oauth_client_id))
        .bind(normalize_optional(app.honor_app_secret))
        .bind(normalize_optional(app.meizu_app_id))
        .bind(normalize_optional(app.meizu_app_key))
        .bind(normalize_optional(app.meizu_app_secret))
        .bind(app.online_push_fallback_secs)
        .bind(app.online_message_cache_secs)
        .bind(app.is_default)
        .execute(&self.pool)
        .await?;

        self.find_by_id(&app.id)
            .await?
            .ok_or_else(|| crate::AppError::Database(sqlx::Error::RowNotFound))
    }

    async fn update(&self, id: &str, app: UpdateApp) -> AppResult<PushApp> {
        let huawei_oauth_client_id = app
            .huawei_oauth_client_id
            .or_else(|| app.huawei_app_id.clone());

        let result = sqlx::query(
            r#"
            UPDATE apps SET
                name = $1,
                package_name = $2,
                ios_bundle_id = $3,
                harmony_bundle_name = $4,
                description = $5,
                server_base_url = $6,
                xiaomi_app_id = $7,
                xiaomi_app_key = $8,
                xiaomi_channel_id = $9,
                xiaomi_app_secret = $10,
                huawei_app_id = $11,
                huawei_oauth_client_id = $12,
                huawei_app_secret = $13,
                oppo_app_key = $14,
                oppo_app_secret = $15,
                oppo_master_secret = $16,
                vivo_app_id = $17,
                vivo_app_key = $18,
                vivo_app_secret = $19,
                honor_app_id = $20,
                honor_oauth_client_id = $21,
                honor_app_secret = $22,
                meizu_app_id = $23,
                meizu_app_key = $24,
                meizu_app_secret = $25,
                online_push_fallback_secs = $26,
                online_message_cache_secs = $27,
                updated_at = NOW()
            WHERE id = $28
            "#,
        )
        .bind(&app.name)
        .bind(&app.package_name)
        .bind(normalize_optional(app.ios_bundle_id))
        .bind(normalize_optional(app.harmony_bundle_name))
        .bind(&app.description)
        .bind(normalize_optional(app.server_base_url))
        .bind(normalize_optional(app.xiaomi_app_id))
        .bind(normalize_optional(app.xiaomi_app_key))
        .bind(normalize_optional(app.xiaomi_channel_id))
        .bind(normalize_optional(app.xiaomi_app_secret))
        .bind(normalize_optional(app.huawei_app_id))
        .bind(normalize_optional(huawei_oauth_client_id))
        .bind(normalize_optional(app.huawei_app_secret))
        .bind(normalize_optional(app.oppo_app_key))
        .bind(normalize_optional(app.oppo_app_secret))
        .bind(normalize_optional(app.oppo_master_secret))
        .bind(normalize_optional(app.vivo_app_id))
        .bind(normalize_optional(app.vivo_app_key))
        .bind(normalize_optional(app.vivo_app_secret))
        .bind(normalize_optional(app.honor_app_id))
        .bind(normalize_optional(app.honor_oauth_client_id))
        .bind(normalize_optional(app.honor_app_secret))
        .bind(normalize_optional(app.meizu_app_id))
        .bind(normalize_optional(app.meizu_app_key))
        .bind(normalize_optional(app.meizu_app_secret))
        .bind(app.online_push_fallback_secs)
        .bind(app.online_message_cache_secs)
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(crate::AppError::NotFound(format!("app not found: {id}")));
        }

        self.find_by_id(id)
            .await?
            .ok_or_else(|| crate::AppError::Database(sqlx::Error::RowNotFound))
    }

    async fn find_by_id(&self, id: &str) -> AppResult<Option<PushApp>> {
        let query = format!("SELECT {APP_COLUMNS} FROM apps WHERE id = $1");
        Ok(sqlx::query_as::<_, PushApp>(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?)
    }

    async fn find_by_package_name(&self, package_name: &str) -> AppResult<Option<PushApp>> {
        let query = format!("SELECT {APP_COLUMNS} FROM apps WHERE package_name = $1");
        Ok(sqlx::query_as::<_, PushApp>(&query)
            .bind(package_name)
            .fetch_optional(&self.pool)
            .await?)
    }

    async fn list(&self) -> AppResult<Vec<PushApp>> {
        let query =
            format!("SELECT {APP_COLUMNS} FROM apps ORDER BY is_default DESC, updated_at DESC");
        Ok(sqlx::query_as::<_, PushApp>(&query)
            .fetch_all(&self.pool)
            .await?)
    }

    async fn delete(&self, id: &str) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM apps WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(crate::AppError::NotFound(format!("app not found: {id}")));
        }
        Ok(())
    }

    async fn count(&self) -> AppResult<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM apps")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }

    async fn clear_default(&self) -> AppResult<()> {
        sqlx::query("UPDATE apps SET is_default = FALSE")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn set_default(&self, id: &str) -> AppResult<()> {
        self.clear_default().await?;
        let result =
            sqlx::query("UPDATE apps SET is_default = TRUE, updated_at = NOW() WHERE id = $1")
                .bind(id)
                .execute(&self.pool)
                .await?;
        if result.rows_affected() == 0 {
            return Err(crate::AppError::NotFound(format!("app not found: {id}")));
        }
        Ok(())
    }
}
