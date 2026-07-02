use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::db::AdminUserRepository;
use crate::models::user::AdminUser;
use crate::AppResult;

pub async fn create_repository(pool: PgPool) -> AppResult<Arc<dyn AdminUserRepository>> {
    sqlx::query("ALTER TABLE admin_users ADD COLUMN IF NOT EXISTS is_owner BOOLEAN NOT NULL DEFAULT FALSE")
        .execute(&pool)
        .await?;
    sqlx::query("ALTER TABLE admin_users ADD COLUMN IF NOT EXISTS display_time_zone TEXT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "ALTER TABLE admin_users ADD COLUMN IF NOT EXISTS password_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "UPDATE admin_users
         SET is_owner = TRUE
         WHERE id = (
             SELECT id FROM admin_users ORDER BY created_at ASC LIMIT 1
         )
         AND NOT EXISTS (SELECT 1 FROM admin_users WHERE is_owner = TRUE)",
    )
    .execute(&pool)
    .await?;

    Ok(Arc::new(PgAdminUserRepository { pool }))
}

struct PgAdminUserRepository {
    pool: PgPool,
}

#[async_trait]
impl AdminUserRepository for PgAdminUserRepository {
    async fn create(
        &self,
        id: &str,
        username: &str,
        password_hash: &str,
        is_owner: bool,
    ) -> AppResult<AdminUser> {
        sqlx::query(
            "INSERT INTO admin_users (id, username, password_hash, is_owner) VALUES ($1, $2, $3, $4)",
        )
        .bind(id)
        .bind(username)
        .bind(password_hash)
        .bind(is_owner)
        .execute(&self.pool)
        .await?;

        self.find_by_username(username)
            .await?
            .ok_or_else(|| crate::AppError::Database(sqlx::Error::RowNotFound))
    }

    async fn find_by_username(&self, username: &str) -> AppResult<Option<AdminUser>> {
        Ok(sqlx::query_as::<_, AdminUser>(
            "SELECT id, username, password_hash, is_owner, display_time_zone, password_updated_at, created_at
             FROM admin_users WHERE username = $1",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?)
    }

    async fn find_by_id(&self, id: &str) -> AppResult<Option<AdminUser>> {
        Ok(sqlx::query_as::<_, AdminUser>(
            "SELECT id, username, password_hash, is_owner, display_time_zone, password_updated_at, created_at
             FROM admin_users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?)
    }

    async fn list(&self) -> AppResult<Vec<AdminUser>> {
        Ok(sqlx::query_as::<_, AdminUser>(
            "SELECT id, username, password_hash, is_owner, display_time_zone, password_updated_at, created_at
             FROM admin_users
             ORDER BY is_owner DESC, created_at ASC",
        )
        .fetch_all(&self.pool)
        .await?)
    }

    async fn find_owner(&self) -> AppResult<Option<AdminUser>> {
        Ok(sqlx::query_as::<_, AdminUser>(
            "SELECT id, username, password_hash, is_owner, display_time_zone, password_updated_at, created_at
             FROM admin_users WHERE is_owner = TRUE
             ORDER BY created_at ASC
             LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    async fn update_owner_display_time_zone(&self, time_zone: &str) -> AppResult<()> {
        let updated = sqlx::query(
            "UPDATE admin_users SET display_time_zone = $1 WHERE is_owner = TRUE",
        )
        .bind(time_zone)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if updated == 0 {
            return Err(crate::AppError::BadRequest("未找到主账号".into()));
        }
        Ok(())
    }

    async fn update_password(&self, id: &str, password_hash: &str) -> AppResult<()> {
        let updated = sqlx::query(
            "UPDATE admin_users SET password_hash = $1, password_updated_at = NOW() WHERE id = $2",
        )
        .bind(password_hash)
        .bind(id)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if updated == 0 {
            return Err(crate::AppError::NotFound("用户不存在".into()));
        }
        Ok(())
    }

    async fn update_username(&self, id: &str, username: &str) -> AppResult<()> {
        let updated = sqlx::query("UPDATE admin_users SET username = $1 WHERE id = $2")
            .bind(username)
            .bind(id)
            .execute(&self.pool)
            .await?;

        if updated.rows_affected() == 0 {
            return Err(crate::AppError::NotFound("用户不存在".into()));
        }
        Ok(())
    }

    async fn delete(&self, id: &str) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM admin_users WHERE id = $1 AND is_owner = FALSE")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            let user = self.find_by_id(id).await?;
            if user.is_some() {
                return Err(crate::AppError::BadRequest("主账号不可删除".into()));
            }
            return Err(crate::AppError::NotFound("用户不存在".into()));
        }
        Ok(())
    }

    async fn count(&self) -> AppResult<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM admin_users")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }
}
