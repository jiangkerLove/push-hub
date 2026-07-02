use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PushApp {
    pub id: String,
    pub name: String,
    pub package_name: String,
    pub ios_bundle_id: Option<String>,
    pub harmony_bundle_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub server_base_url: Option<String>,
    pub push_api_key: String,
    pub xiaomi_app_id: Option<String>,
    pub xiaomi_app_key: Option<String>,
    pub xiaomi_channel_id: Option<String>,
    #[serde(skip_serializing)]
    pub xiaomi_app_secret: Option<String>,
    #[serde(skip_serializing)]
    pub huawei_app_id: Option<String>,
    #[serde(skip_serializing)]
    pub huawei_oauth_client_id: Option<String>,
    #[serde(skip_serializing)]
    pub huawei_app_secret: Option<String>,
    pub oppo_app_key: Option<String>,
    #[serde(skip_serializing)]
    pub oppo_app_secret: Option<String>,
    #[serde(skip_serializing)]
    pub oppo_master_secret: Option<String>,
    pub vivo_app_id: Option<String>,
    pub vivo_app_key: Option<String>,
    #[serde(skip_serializing)]
    pub vivo_app_secret: Option<String>,
    #[serde(skip_serializing)]
    pub honor_app_id: Option<String>,
    #[serde(skip_serializing)]
    pub honor_oauth_client_id: Option<String>,
    #[serde(skip_serializing)]
    pub honor_app_secret: Option<String>,
    pub meizu_app_id: Option<String>,
    pub meizu_app_key: Option<String>,
    #[serde(skip_serializing)]
    pub meizu_app_secret: Option<String>,
    pub online_push_fallback_secs: i64,
    pub online_message_cache_secs: i64,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PushAppSummary {
    pub id: String,
    pub name: String,
    pub package_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub has_xiaomi: bool,
    pub has_huawei: bool,
    pub has_oppo: bool,
    pub has_vivo: bool,
    pub has_honor: bool,
    pub has_meizu: bool,
    pub online_push_fallback_secs: i64,
    pub online_message_cache_secs: i64,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PushApp> for PushAppSummary {
    fn from(app: PushApp) -> Self {
        Self {
            id: app.id,
            name: app.name,
            package_name: app.package_name,
            description: app.description,
            has_xiaomi: app.xiaomi_app_secret.as_ref().is_some_and(|v| !v.is_empty()),
            has_huawei: app
                .huawei_app_secret
                .as_ref()
                .is_some_and(|v| !v.is_empty())
                && app.huawei_app_id.as_ref().is_some_and(|v| !v.is_empty()),
            has_oppo: app
                .oppo_master_secret
                .as_ref()
                .is_some_and(|v| !v.is_empty())
                && app.oppo_app_key.as_ref().is_some_and(|v| !v.is_empty()),
            has_vivo: app
                .vivo_app_secret
                .as_ref()
                .is_some_and(|v| !v.is_empty())
                && app.vivo_app_id.as_ref().is_some_and(|v| !v.is_empty())
                && app.vivo_app_key.as_ref().is_some_and(|v| !v.is_empty()),
            has_honor: app
                .honor_app_secret
                .as_ref()
                .is_some_and(|v| !v.is_empty())
                && app.honor_app_id.as_ref().is_some_and(|v| !v.is_empty())
                && app
                    .honor_oauth_client_id
                    .as_ref()
                    .is_some_and(|v| !v.is_empty()),
            has_meizu: app
                .meizu_app_secret
                .as_ref()
                .is_some_and(|v| !v.is_empty())
                && app.meizu_app_id.as_ref().is_some_and(|v| !v.is_empty()),
            online_push_fallback_secs: app.online_push_fallback_secs,
            online_message_cache_secs: app.online_message_cache_secs,
            is_default: app.is_default,
            created_at: app.created_at,
            updated_at: app.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PushAppConfigView {
    pub id: String,
    pub name: String,
    pub package_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ios_bundle_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub harmony_bundle_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub server_base_url: Option<String>,
    pub push_api_key: String,
    pub has_xiaomi: bool,
    pub has_huawei: bool,
    pub has_oppo: bool,
    pub has_vivo: bool,
    pub has_honor: bool,
    pub has_meizu: bool,
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

impl From<PushApp> for PushAppConfigView {
    fn from(app: PushApp) -> Self {
        let summary = PushAppSummary::from(app.clone());
        Self {
            id: app.id,
            name: app.name,
            package_name: app.package_name,
            ios_bundle_id: app.ios_bundle_id,
            harmony_bundle_name: app.harmony_bundle_name,
            description: app.description,
            server_base_url: app.server_base_url,
            push_api_key: app.push_api_key,
            has_xiaomi: summary.has_xiaomi,
            has_huawei: summary.has_huawei,
            has_oppo: summary.has_oppo,
            has_vivo: summary.has_vivo,
            has_honor: summary.has_honor,
            has_meizu: summary.has_meizu,
            xiaomi_app_id: app.xiaomi_app_id,
            xiaomi_app_key: app.xiaomi_app_key,
            xiaomi_channel_id: app.xiaomi_channel_id,
            xiaomi_app_secret: app.xiaomi_app_secret,
            huawei_app_id: app.huawei_app_id,
            huawei_oauth_client_id: app.huawei_oauth_client_id,
            huawei_app_secret: app.huawei_app_secret,
            oppo_app_key: app.oppo_app_key,
            oppo_app_secret: app.oppo_app_secret,
            oppo_master_secret: app.oppo_master_secret,
            vivo_app_id: app.vivo_app_id,
            vivo_app_key: app.vivo_app_key,
            vivo_app_secret: app.vivo_app_secret,
            honor_app_id: app.honor_app_id,
            honor_oauth_client_id: app.honor_oauth_client_id,
            honor_app_secret: app.honor_app_secret,
            meizu_app_id: app.meizu_app_id,
            meizu_app_key: app.meizu_app_key,
            meizu_app_secret: app.meizu_app_secret,
            online_push_fallback_secs: app.online_push_fallback_secs,
            online_message_cache_secs: app.online_message_cache_secs,
            is_default: app.is_default,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAppRequest {
    pub name: String,
    #[serde(default)]
    pub package_name: Option<String>,
    #[serde(default)]
    pub ios_bundle_id: Option<String>,
    #[serde(default)]
    pub harmony_bundle_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub server_base_url: Option<String>,
    #[serde(default)]
    pub xiaomi_app_id: Option<String>,
    #[serde(default)]
    pub xiaomi_app_key: Option<String>,
    #[serde(default)]
    pub xiaomi_channel_id: Option<String>,
    #[serde(default)]
    pub xiaomi_app_secret: Option<String>,
    #[serde(default)]
    pub huawei_app_id: Option<String>,
    #[serde(default)]
    pub huawei_oauth_client_id: Option<String>,
    #[serde(default)]
    pub huawei_app_secret: Option<String>,
    #[serde(default)]
    pub oppo_app_key: Option<String>,
    #[serde(default)]
    pub oppo_app_secret: Option<String>,
    #[serde(default)]
    pub oppo_master_secret: Option<String>,
    #[serde(default)]
    pub vivo_app_id: Option<String>,
    #[serde(default)]
    pub vivo_app_key: Option<String>,
    #[serde(default)]
    pub vivo_app_secret: Option<String>,
    #[serde(default)]
    pub honor_app_id: Option<String>,
    #[serde(default)]
    pub honor_oauth_client_id: Option<String>,
    #[serde(default)]
    pub honor_app_secret: Option<String>,
    #[serde(default)]
    pub meizu_app_id: Option<String>,
    #[serde(default)]
    pub meizu_app_key: Option<String>,
    #[serde(default)]
    pub meizu_app_secret: Option<String>,
    #[serde(default)]
    pub online_push_fallback_secs: Option<i64>,
    #[serde(default)]
    pub online_message_cache_secs: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAppRequest {
    pub name: String,
    #[serde(default)]
    pub package_name: Option<String>,
    #[serde(default)]
    pub ios_bundle_id: Option<String>,
    #[serde(default)]
    pub harmony_bundle_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub server_base_url: Option<String>,
    #[serde(default)]
    pub xiaomi_app_id: Option<String>,
    #[serde(default)]
    pub xiaomi_app_key: Option<String>,
    #[serde(default)]
    pub xiaomi_channel_id: Option<String>,
    #[serde(default)]
    pub xiaomi_app_secret: Option<String>,
    #[serde(default)]
    pub huawei_app_id: Option<String>,
    #[serde(default)]
    pub huawei_oauth_client_id: Option<String>,
    #[serde(default)]
    pub huawei_app_secret: Option<String>,
    #[serde(default)]
    pub oppo_app_key: Option<String>,
    #[serde(default)]
    pub oppo_app_secret: Option<String>,
    #[serde(default)]
    pub oppo_master_secret: Option<String>,
    #[serde(default)]
    pub vivo_app_id: Option<String>,
    #[serde(default)]
    pub vivo_app_key: Option<String>,
    #[serde(default)]
    pub vivo_app_secret: Option<String>,
    #[serde(default)]
    pub honor_app_id: Option<String>,
    #[serde(default)]
    pub honor_oauth_client_id: Option<String>,
    #[serde(default)]
    pub honor_app_secret: Option<String>,
    #[serde(default)]
    pub meizu_app_id: Option<String>,
    #[serde(default)]
    pub meizu_app_key: Option<String>,
    #[serde(default)]
    pub meizu_app_secret: Option<String>,
    #[serde(default)]
    pub online_push_fallback_secs: Option<i64>,
    #[serde(default)]
    pub online_message_cache_secs: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ValidateAppCredentialsRequest {
    /// 仅验证指定厂商；为空则验证所有已填写的厂商
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub package_name: Option<String>,
    #[serde(default)]
    pub xiaomi_app_secret: Option<String>,
    #[serde(default)]
    pub huawei_app_id: Option<String>,
    #[serde(default)]
    pub huawei_oauth_client_id: Option<String>,
    #[serde(default)]
    pub huawei_app_secret: Option<String>,
    #[serde(default)]
    pub oppo_app_key: Option<String>,
    #[serde(default)]
    pub oppo_master_secret: Option<String>,
    #[serde(default)]
    pub vivo_app_id: Option<String>,
    #[serde(default)]
    pub vivo_app_key: Option<String>,
    #[serde(default)]
    pub vivo_app_secret: Option<String>,
    #[serde(default)]
    pub honor_app_id: Option<String>,
    #[serde(default)]
    pub honor_oauth_client_id: Option<String>,
    #[serde(default)]
    pub honor_app_secret: Option<String>,
    #[serde(default)]
    pub meizu_app_id: Option<String>,
    #[serde(default)]
    pub meizu_app_secret: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VendorCredentialValidation {
    pub platform: String,
    pub label: String,
    /// skipped | incomplete | ok | failed
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidateAppCredentialsResponse {
    pub results: Vec<VendorCredentialValidation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdminLoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdminSetupRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminLoginResponse {
    pub token: String,
    pub username: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminBootstrapStatus {
    /// 尚无管理员账号时为 true，需先调用 setup 创建账号
    pub needs_setup: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminProfile {
    pub username: String,
    pub is_owner: bool,
    /// 主账号设置的展示时区；子账号与主账号共用
    pub display_time_zone: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateDisplayTimeZoneRequest {
    pub display_time_zone: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminUserSummary {
    pub id: String,
    pub username: String,
    pub is_owner: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAdminUserRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResetAdminUserPasswordRequest {
    pub new_password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateMyUsernameRequest {
    pub username: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateMyUsernameResponse {
    pub token: String,
    pub username: String,
    pub is_owner: bool,
    pub display_time_zone: Option<String>,
}
