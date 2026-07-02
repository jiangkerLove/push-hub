use chrono::{DateTime, Utc};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AdminUser {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub is_owner: bool,
    pub display_time_zone: Option<String>,
    pub password_updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl AdminUser {
    pub fn password_version(&self) -> i64 {
        self.password_updated_at.timestamp()
    }
}
