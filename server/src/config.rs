use crate::error::{AppError, AppResult};

#[derive(Clone, Debug)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub database_url: String,
    pub online_push_fallback_secs: i64,
    pub online_message_cache_secs: i64,
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> AppResult<Self> {
        Ok(Self {
            server_host: env_or("SERVER_HOST", "0.0.0.0"),
            server_port: env_or("SERVER_PORT", "3000")
                .parse()
                .map_err(|_| AppError::Config("SERVER_PORT must be a valid u16".into()))?,
            database_url: env_or(
                "DATABASE_URL",
                "postgres://postgres:postgres@127.0.0.1:5432/push_hub",
            ),
            online_push_fallback_secs: std::env::var("ONLINE_PUSH_FALLBACK_SECS")
                .ok()
                .or_else(|| std::env::var("DEVICE_ONLINE_TTL_SECS").ok())
                .unwrap_or_else(|| "90".into())
                .parse()
                .map_err(|_| {
                    AppError::Config("ONLINE_PUSH_FALLBACK_SECS must be a valid i64".into())
                })?,
            online_message_cache_secs: std::env::var("ONLINE_MESSAGE_CACHE_SECS")
                .unwrap_or_else(|_| "86400".into())
                .parse()
                .map_err(|_| {
                    AppError::Config("ONLINE_MESSAGE_CACHE_SECS must be a valid i64".into())
                })?,
            jwt_secret: env_or("JWT_SECRET", "push-hub-dev-secret-change-me"),
        })
    }

    pub fn listen_addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
