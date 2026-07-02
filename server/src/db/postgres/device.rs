use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::db::{DeviceRepository, NewDevice};
use crate::models::{assign_device_id, Device, DeviceStatsOverview, PlatformStat};
use crate::AppResult;

pub async fn create_repository(pool: PgPool) -> AppResult<Arc<dyn DeviceRepository>> {
    Ok(Arc::new(PgDeviceRepository { pool }))
}

struct PgDeviceRepository {
    pool: PgPool,
}

const DEVICE_COLUMNS: &str =
    "id, app_id, package_name, platform, push_token, online_token, last_online_at, created_at, updated_at";

#[async_trait]
impl DeviceRepository for PgDeviceRepository {
    async fn upsert(&self, device: NewDevice) -> AppResult<Device> {
        let platform = device.platform.trim().to_lowercase();
        let push_token = device.push_token.trim().to_string();
        let online_token = device
            .online_token
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty());

        if let Some(device_id) = device
            .device_id
            .as_deref()
            .map(str::trim)
            .filter(|id| !id.is_empty())
        {
            if let Some(existing) = self.find_by_id(device_id).await? {
                // 同一应用下复用该身份；允许 platform 从 online 升级为厂商通道
                if existing.app_id.is_empty() || existing.app_id == device.app_id {
                    return self
                        .update_existing(
                            &existing.id,
                            &device.app_id,
                            &device.package_name,
                            &platform,
                            &push_token,
                            online_token,
                        )
                        .await;
                }
            }
        }

        let id = assign_device_id(&platform, &push_token);

        sqlx::query(
            r#"
            INSERT INTO devices (id, app_id, package_name, platform, push_token, online_token, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            ON CONFLICT (platform, push_token) DO UPDATE SET
                app_id = EXCLUDED.app_id,
                package_name = EXCLUDED.package_name,
                online_token = COALESCE(EXCLUDED.online_token, devices.online_token),
                updated_at = NOW()
            "#,
        )
        .bind(&id)
        .bind(&device.app_id)
        .bind(&device.package_name)
        .bind(&platform)
        .bind(&push_token)
        .bind(online_token)
        .execute(&self.pool)
        .await?;

        self.find_by_push_token(&platform, &push_token)
            .await?
            .ok_or_else(|| crate::AppError::Database(sqlx::Error::RowNotFound))
    }

    async fn find_by_id(&self, id: &str) -> AppResult<Option<Device>> {
        let query = format!("SELECT {DEVICE_COLUMNS} FROM devices WHERE id = $1");
        Ok(sqlx::query_as::<_, Device>(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?)
    }

    async fn find_by_push_token(
        &self,
        platform: &str,
        push_token: &str,
    ) -> AppResult<Option<Device>> {
        let query =
            format!("SELECT {DEVICE_COLUMNS} FROM devices WHERE platform = $1 AND push_token = $2");
        Ok(sqlx::query_as::<_, Device>(&query)
            .bind(platform)
            .bind(push_token)
            .fetch_optional(&self.pool)
            .await?)
    }

    async fn find_by_ids(&self, ids: &[String]) -> AppResult<Vec<Device>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders = super::placeholders(1, ids.len());
        let query = format!("SELECT {DEVICE_COLUMNS} FROM devices WHERE id IN ({placeholders})");
        let mut q = sqlx::query_as::<_, Device>(&query);
        for id in ids {
            q = q.bind(id);
        }
        Ok(q.fetch_all(&self.pool).await?)
    }

    async fn list(&self, limit: i64, offset: i64) -> AppResult<Vec<Device>> {
        let query = format!(
            "SELECT {DEVICE_COLUMNS} FROM devices ORDER BY updated_at DESC LIMIT $1 OFFSET $2"
        );
        Ok(sqlx::query_as::<_, Device>(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?)
    }

    async fn list_by_app_id(
        &self,
        app_id: &str,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<Device>> {
        let query = format!(
            "SELECT {DEVICE_COLUMNS} FROM devices WHERE app_id = $1 ORDER BY updated_at DESC LIMIT $2 OFFSET $3"
        );
        Ok(sqlx::query_as::<_, Device>(&query)
            .bind(app_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?)
    }

    async fn list_by_package_name(
        &self,
        package_name: &str,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<Device>> {
        let query = format!(
            "SELECT {DEVICE_COLUMNS} FROM devices WHERE package_name = $1 ORDER BY updated_at DESC LIMIT $2 OFFSET $3"
        );
        Ok(sqlx::query_as::<_, Device>(&query)
            .bind(package_name)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?)
    }

    async fn touch_online(&self, online_token: &str) -> AppResult<()> {
        let token = online_token.trim();
        if token.is_empty() {
            return Ok(());
        }

        sqlx::query(
            r#"
            UPDATE devices
            SET last_online_at = NOW(), updated_at = NOW()
            WHERE platform = 'online' AND push_token = $1
            "#,
        )
        .bind(token)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            UPDATE devices
            SET last_online_at = NOW(), updated_at = NOW()
            WHERE online_token = $1
            "#,
        )
        .bind(token)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_online_push_token(&self, token: &str) -> AppResult<Option<Device>> {
        let token = token.trim();
        if token.is_empty() {
            return Ok(None);
        }
        let query = format!(
            "SELECT {DEVICE_COLUMNS} FROM devices WHERE online_token = $1 OR (platform = 'online' AND push_token = $2) LIMIT 1"
        );
        Ok(sqlx::query_as::<_, Device>(&query)
            .bind(token)
            .bind(token)
            .fetch_optional(&self.pool)
            .await?)
    }

    async fn stats_for_app(
        &self,
        app_id: &str,
        since_days: i64,
        online_within_secs: i64,
    ) -> AppResult<DeviceStatsOverview> {
        let since_days = since_days.clamp(1, 90);
        let since = chrono::Utc::now() - chrono::Duration::days(since_days);
        let online_secs = online_within_secs.clamp(1, 86_400);

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM devices WHERE app_id = $1")
            .bind(app_id)
            .fetch_one(&self.pool)
            .await?;

        let recent_online: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM devices
            WHERE app_id = $1
              AND last_online_at IS NOT NULL
              AND last_online_at >= NOW() - make_interval(secs => $2::INT)
            "#,
        )
        .bind(app_id)
        .bind(online_secs as i32)
        .fetch_one(&self.pool)
        .await?;

        let new_in_period: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM devices WHERE app_id = $1 AND created_at >= $2",
        )
        .bind(app_id)
        .bind(since)
        .fetch_one(&self.pool)
        .await?;

        let by_platform = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT platform, COUNT(*)::BIGINT AS cnt
            FROM devices
            WHERE app_id = $1
            GROUP BY platform
            ORDER BY cnt DESC
            "#,
        )
        .bind(app_id)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|(platform, count)| PlatformStat { platform, count })
        .collect();

        Ok(DeviceStatsOverview {
            total: total.0,
            recent_online: recent_online.0,
            new_in_period: new_in_period.0,
            by_platform,
        })
    }
}

impl PgDeviceRepository {
    async fn update_existing(
        &self,
        id: &str,
        app_id: &str,
        package_name: &str,
        platform: &str,
        push_token: &str,
        online_token: Option<&str>,
    ) -> AppResult<Device> {
        // 新 token 若已被其他 device 占用，删掉冲突行，避免 UNIQUE(platform, push_token) 失败
        sqlx::query(
            r#"
            DELETE FROM devices
            WHERE platform = $1 AND push_token = $2 AND id <> $3
            "#,
        )
        .bind(platform)
        .bind(push_token)
        .bind(id)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            UPDATE devices SET
                app_id = $1,
                package_name = $2,
                platform = $3,
                push_token = $4,
                online_token = COALESCE($5, online_token),
                updated_at = NOW()
            WHERE id = $6
            "#,
        )
        .bind(app_id)
        .bind(package_name)
        .bind(platform)
        .bind(push_token)
        .bind(online_token)
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| crate::AppError::Database(sqlx::Error::RowNotFound))
    }
}
