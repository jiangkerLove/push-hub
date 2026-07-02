use async_trait::async_trait;

use crate::models::{Device, EnqueueResult, OutboxFallbackJob, OutboxMessage, PushTemplate, RenderedNotification, TemplateChannels, ClickAction};
use crate::AppResult;

#[derive(Debug, Clone)]
pub struct NewDevice {
    pub app_id: String,
    /// 客户端已绑定的 device_id；存在时优先按该 id 更新 push_token，保持身份稳定
    pub device_id: Option<String>,
    pub package_name: String,
    pub platform: String,
    pub push_token: String,
    pub online_token: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewTemplate {
    pub id: String,
    pub app_id: String,
    pub name: String,
    pub kind: String,
    pub content_mode: String,
    pub title: String,
    pub body: String,
    pub channels: TemplateChannels,
    pub click_action: ClickAction,
    pub message_cache_days: i64,
}

#[derive(Debug, Clone)]
pub struct UpdateTemplate {
    pub name: String,
    pub kind: String,
    pub content_mode: String,
    pub title: String,
    pub body: String,
    pub channels: TemplateChannels,
    pub click_action: ClickAction,
    pub message_cache_days: i64,
}

#[async_trait]
pub trait DeviceRepository: Send + Sync {
    async fn upsert(&self, device: NewDevice) -> AppResult<Device>;
    async fn find_by_id(&self, id: &str) -> AppResult<Option<Device>>;
    async fn find_by_push_token(&self, platform: &str, push_token: &str) -> AppResult<Option<Device>>;
    async fn find_by_ids(&self, ids: &[String]) -> AppResult<Vec<Device>>;
    async fn list(&self, limit: i64, offset: i64) -> AppResult<Vec<Device>>;
    async fn list_by_app_id(
        &self,
        app_id: &str,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<Device>>;
    async fn list_by_package_name(
        &self,
        package_name: &str,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<Device>>;
    async fn touch_online(&self, online_token: &str) -> AppResult<()>;
    async fn find_by_online_push_token(&self, token: &str) -> AppResult<Option<Device>>;
    async fn stats_for_app(
        &self,
        app_id: &str,
        since_days: i64,
        online_within_secs: i64,
    ) -> AppResult<crate::models::DeviceStatsOverview>;
}

#[async_trait]
pub trait OutboxRepository: Send + Sync {
    async fn enqueue(
        &self,
        push_tokens: &[String],
        notification: &RenderedNotification,
    ) -> AppResult<EnqueueResult>;

    async fn fetch_pending(
        &self,
        push_token: &str,
        limit: i64,
    ) -> AppResult<Vec<OutboxMessage>>;

    async fn ack(&self, push_token: &str, ids: &[String]) -> AppResult<usize>;

    /// 超时未 ack 且尚未降级厂商推送的 outbox 消息
    async fn list_stale_fallbacks(&self, older_than_secs: i64) -> AppResult<Vec<OutboxFallbackJob>>;

    async fn mark_fallback_sent(&self, ids: &[String]) -> AppResult<usize>;
    async fn find_fallback_jobs_by_ids(&self, ids: &[String]) -> AppResult<Vec<OutboxFallbackJob>>;
    async fn clear_fallback_targets(&self, ids: &[String]) -> AppResult<()>;
    async fn mark_delivered(&self, ids: &[String]) -> AppResult<usize>;
}

#[async_trait]
pub trait TemplateRepository: Send + Sync {
    async fn create(&self, template: NewTemplate) -> AppResult<PushTemplate>;
    async fn update(&self, id: &str, template: UpdateTemplate) -> AppResult<PushTemplate>;
    async fn find_by_id(&self, id: &str) -> AppResult<Option<PushTemplate>>;
    async fn list(&self) -> AppResult<Vec<PushTemplate>>;
    async fn list_by_app_id(&self, app_id: &str) -> AppResult<Vec<PushTemplate>>;
    async fn delete(&self, id: &str) -> AppResult<()>;
}

use crate::models::PushApp;

#[derive(Debug, Clone)]
pub struct NewApp {
    pub id: String,
    pub name: String,
    pub package_name: String,
    pub ios_bundle_id: Option<String>,
    pub harmony_bundle_name: Option<String>,
    pub description: Option<String>,
    pub server_base_url: Option<String>,
    pub push_api_key: String,
    pub xiaomi_app_id: Option<String>,
    pub xiaomi_app_key: Option<String>,
    pub xiaomi_channel_id: Option<String>,
    pub xiaomi_app_secret: Option<String>,
    pub huawei_app_id: Option<String>,
    pub huawei_oauth_client_id: Option<String>,
    pub huawei_app_secret: Option<String>,
    pub oppo_app_key: Option<String>,
    pub oppo_app_secret: Option<String>,
    pub oppo_master_secret: Option<String>,
    pub vivo_app_id: Option<String>,
    pub vivo_app_key: Option<String>,
    pub vivo_app_secret: Option<String>,
    pub honor_app_id: Option<String>,
    pub honor_oauth_client_id: Option<String>,
    pub honor_app_secret: Option<String>,
    pub meizu_app_id: Option<String>,
    pub meizu_app_key: Option<String>,
    pub meizu_app_secret: Option<String>,
    pub online_push_fallback_secs: i64,
    pub online_message_cache_secs: i64,
    pub is_default: bool,
}

#[derive(Debug, Clone)]
pub struct UpdateApp {
    pub name: String,
    pub package_name: String,
    pub ios_bundle_id: Option<String>,
    pub harmony_bundle_name: Option<String>,
    pub description: Option<String>,
    pub server_base_url: Option<String>,
    pub xiaomi_app_id: Option<String>,
    pub xiaomi_app_key: Option<String>,
    pub xiaomi_channel_id: Option<String>,
    pub xiaomi_app_secret: Option<String>,
    pub huawei_app_id: Option<String>,
    pub huawei_oauth_client_id: Option<String>,
    pub huawei_app_secret: Option<String>,
    pub oppo_app_key: Option<String>,
    pub oppo_app_secret: Option<String>,
    pub oppo_master_secret: Option<String>,
    pub vivo_app_id: Option<String>,
    pub vivo_app_key: Option<String>,
    pub vivo_app_secret: Option<String>,
    pub honor_app_id: Option<String>,
    pub honor_oauth_client_id: Option<String>,
    pub honor_app_secret: Option<String>,
    pub meizu_app_id: Option<String>,
    pub meizu_app_key: Option<String>,
    pub meizu_app_secret: Option<String>,
    pub online_push_fallback_secs: i64,
    pub online_message_cache_secs: i64,
}

#[async_trait]
pub trait AppRepository: Send + Sync {
    async fn create(&self, app: NewApp) -> AppResult<PushApp>;
    async fn update(&self, id: &str, app: UpdateApp) -> AppResult<PushApp>;
    async fn find_by_id(&self, id: &str) -> AppResult<Option<PushApp>>;
    async fn find_by_package_name(&self, package_name: &str) -> AppResult<Option<PushApp>>;
    async fn list(&self) -> AppResult<Vec<PushApp>>;
    async fn delete(&self, id: &str) -> AppResult<()>;
    async fn count(&self) -> AppResult<i64>;
    async fn clear_default(&self) -> AppResult<()>;
    async fn set_default(&self, id: &str) -> AppResult<()>;
}

#[async_trait]
pub trait AdminUserRepository: Send + Sync {
    async fn create(
        &self,
        id: &str,
        username: &str,
        password_hash: &str,
        is_owner: bool,
    ) -> AppResult<crate::models::user::AdminUser>;
    async fn find_by_username(&self, username: &str) -> AppResult<Option<crate::models::user::AdminUser>>;
    async fn find_by_id(&self, id: &str) -> AppResult<Option<crate::models::user::AdminUser>>;
    async fn list(&self) -> AppResult<Vec<crate::models::user::AdminUser>>;
    async fn find_owner(&self) -> AppResult<Option<crate::models::user::AdminUser>>;
    async fn update_owner_display_time_zone(&self, time_zone: &str) -> AppResult<()>;
    async fn update_password(&self, id: &str, password_hash: &str) -> AppResult<()>;
    async fn update_username(&self, id: &str, username: &str) -> AppResult<()>;
    async fn delete(&self, id: &str) -> AppResult<()>;
    async fn count(&self) -> AppResult<i64>;
}

#[derive(Debug, Clone)]
pub struct NewPushJob {
    pub id: String,
    pub app_id: String,
    pub template_id: String,
    pub template_name: String,
    pub title: String,
    pub body: String,
    pub total_targets: i64,
}

#[derive(Debug, Clone)]
pub struct NewPushJobTarget {
    pub id: String,
    pub job_id: String,
    pub device_id: Option<String>,
    pub platform: String,
    pub push_token: String,
}

#[derive(Debug, Clone)]
pub struct UpdatePushJobTarget {
    pub route_decision: String,
}

#[derive(Debug, Clone)]
pub struct NewPushJobEvent {
    pub id: String,
    pub job_id: String,
    pub target_id: Option<String>,
    pub stage: String,
    pub status: String,
    pub platform: Option<String>,
    pub detail: String,
    pub metadata: Option<String>,
}

#[async_trait]
pub trait PushTraceRepository: Send + Sync {
    async fn create_job(&self, job: NewPushJob) -> AppResult<crate::models::PushJob>;
    async fn finish_job(&self, job_id: &str, success: i64, failed: i64) -> AppResult<()>;
    async fn set_job_batch_id(&self, job_id: &str, batch_id: &str) -> AppResult<()>;
    async fn create_target(&self, target: NewPushJobTarget) -> AppResult<crate::models::PushJobTarget>;
    async fn update_target(&self, target_id: &str, update: UpdatePushJobTarget) -> AppResult<()>;
    async fn set_target_outbox(&self, target_id: &str, outbox_id: &str) -> AppResult<()>;
    async fn set_target_vendor_message(
        &self,
        target_id: &str,
        vendor_message_id: &str,
    ) -> AppResult<()>;
    async fn finish_target(
        &self,
        target_id: &str,
        status: &str,
        channel: Option<&str>,
    ) -> AppResult<()>;
    async fn add_event(&self, event: NewPushJobEvent) -> AppResult<()>;
    async fn link_outbox(&self, outbox_id: &str, job_id: &str, target_id: &str) -> AppResult<()>;
    async fn find_job_id_by_outbox(&self, outbox_id: &str) -> AppResult<Option<String>>;
    async fn find_target_id_by_outbox(&self, outbox_id: &str) -> AppResult<Option<String>>;
    async fn list_jobs(
        &self,
        app_id: &str,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<crate::models::PushJob>>;
    async fn get_job_detail(
        &self,
        job_id: &str,
    ) -> AppResult<Option<crate::models::PushJobDetail>>;
    async fn stats(&self, app_id: &str, days: i64) -> AppResult<crate::models::PushStatsOverview>;
}

#[derive(Debug, Clone)]
pub struct NewPushChannel {
    pub id: String,
    pub app_id: String,
    pub platform: String,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub is_default: bool,
}

#[derive(Debug, Clone)]
pub struct UpdatePushChannel {
    pub platform: String,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub is_default: bool,
}

#[async_trait]
pub trait ChannelRepository: Send + Sync {
    async fn create(&self, channel: NewPushChannel) -> AppResult<crate::models::PushChannel>;
    async fn update(&self, id: &str, channel: UpdatePushChannel) -> AppResult<crate::models::PushChannel>;
    async fn find_by_id(&self, id: &str) -> AppResult<Option<crate::models::PushChannel>>;
    async fn list_by_app_id(&self, app_id: &str) -> AppResult<Vec<crate::models::PushChannel>>;
    async fn delete(&self, id: &str) -> AppResult<()>;
    async fn clear_default(&self, app_id: &str, platform: &str) -> AppResult<()>;
}

pub mod postgres;
pub mod seed;

use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub enum DatabaseKind {
    Postgres,
}

pub struct Database {
    kind: DatabaseKind,
    devices: Arc<dyn DeviceRepository>,
    templates: Arc<dyn TemplateRepository>,
    outbox: Arc<dyn OutboxRepository>,
    apps: Arc<dyn AppRepository>,
    admin_users: Arc<dyn AdminUserRepository>,
    push_trace: Arc<dyn PushTraceRepository>,
    channels: Arc<dyn ChannelRepository>,
}

impl Database {
    pub async fn connect(url: &str) -> AppResult<Self> {
        let postgres_url = url.starts_with("postgres://")
            || url.starts_with("postgresql://")
            || url.starts_with("postgres:")
            || url.starts_with("postgresql:");
        if !postgres_url {
            return Err(crate::AppError::Config(format!(
                "unsupported database url: {url} (expect postgres:// or postgresql://)"
            )));
        }

        let (devices, templates, outbox, apps, admin_users, push_trace, channels) =
            postgres::connect(url).await?;
        Ok(Self {
            kind: DatabaseKind::Postgres,
            devices,
            templates,
            outbox,
            apps,
            admin_users,
            push_trace,
            channels,
        })
    }

    pub fn kind(&self) -> &DatabaseKind {
        &self.kind
    }

    pub fn devices(&self) -> &dyn DeviceRepository {
        self.devices.as_ref()
    }

    pub fn templates(&self) -> &dyn TemplateRepository {
        self.templates.as_ref()
    }

    pub fn outbox(&self) -> Arc<dyn OutboxRepository> {
        self.outbox.clone()
    }

    pub fn apps(&self) -> &dyn AppRepository {
        self.apps.as_ref()
    }

    pub fn admin_users(&self) -> &dyn AdminUserRepository {
        self.admin_users.as_ref()
    }

    pub fn push_trace(&self) -> &dyn PushTraceRepository {
        self.push_trace.as_ref()
    }

    pub fn channels(&self) -> &dyn ChannelRepository {
        self.channels.as_ref()
    }
}
