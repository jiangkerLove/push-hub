use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;

use crate::db::{
    AdminUserRepository, AppRepository, ChannelRepository, DeviceRepository, OutboxRepository,
    PushTraceRepository, TemplateRepository,
};
use crate::AppResult;

mod admin_user;
mod app;
mod channel;
mod device;
mod outbox;
pub mod push_trace;
mod template;

pub async fn connect(url: &str) -> AppResult<(
    Arc<dyn DeviceRepository>,
    Arc<dyn TemplateRepository>,
    Arc<dyn OutboxRepository>,
    Arc<dyn AppRepository>,
    Arc<dyn AdminUserRepository>,
    Arc<dyn PushTraceRepository>,
    Arc<dyn ChannelRepository>,
)> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(url)
        .await?;

    let apps = app::create_repository(pool.clone()).await?;
    let admin_users = admin_user::create_repository(pool.clone()).await?;
    let devices = device::create_repository(pool.clone()).await?;
    let templates = template::create_repository(pool.clone()).await?;
    let outbox = outbox::create_repository(pool.clone()).await?;
    let push_trace = push_trace::create_repository(pool.clone()).await?;
    let channels = channel::create_repository(pool).await?;

    Ok((devices, templates, outbox, apps, admin_users, push_trace, channels))
}

pub(crate) fn placeholders(start: usize, count: usize) -> String {
    (0..count)
        .map(|i| format!("${}", start + i))
        .collect::<Vec<_>>()
        .join(", ")
}
