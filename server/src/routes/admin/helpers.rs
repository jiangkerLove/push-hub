use axum::http::HeaderMap;
use uuid::Uuid;

use crate::admin::auth;
use crate::models::{
    normalize_platform, AdminProfile, AdminUserSummary, PushApp, PushAppConfigView, PushAppSummary,
};
use crate::models::user::AdminUser;
use crate::state::AppState;
use crate::{AppError, AppResult};

#[derive(Debug, serde::Deserialize)]
pub(super) struct ListDevicesQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Debug, serde::Deserialize)]
pub(super) struct PushStatsQuery {
    #[serde(default = "default_stats_days")]
    pub days: i64,
}

pub(super) fn default_limit() -> i64 {
    50
}

pub(super) fn default_stats_days() -> i64 {
    7
}

pub(super) fn admin_user_summary(user: AdminUser) -> AdminUserSummary {
    AdminUserSummary {
        id: user.id,
        username: user.username,
        is_owner: user.is_owner,
        created_at: user.created_at,
    }
}

pub(super) async fn current_user(state: &AppState, username: &str) -> AppResult<AdminUser> {
    state
        .db
        .admin_users()
        .find_by_username(username)
        .await?
        .ok_or_else(|| AppError::Unauthorized("用户不存在".into()))
}

pub(super) async fn require_owner(state: &AppState, username: &str) -> AppResult<AdminUser> {
    let user = current_user(state, username).await?;
    if !user.is_owner {
        return Err(AppError::Forbidden("仅主账号可执行此操作".into()));
    }
    Ok(user)
}

pub(super) async fn build_admin_profile(
    state: &AppState,
    username: &str,
) -> AppResult<AdminProfile> {
    let user = state
        .db
        .admin_users()
        .find_by_username(username)
        .await?
        .ok_or_else(|| AppError::Unauthorized("用户不存在".into()))?;
    let owner = state.db.admin_users().find_owner().await?;
    Ok(AdminProfile {
        username: user.username,
        is_owner: user.is_owner,
        display_time_zone: owner.and_then(|owner| owner.display_time_zone),
    })
}

pub(super) fn validate_display_time_zone(time_zone: &str) -> AppResult<()> {
    let time_zone = time_zone.trim();
    if time_zone.is_empty() {
        return Err(AppError::BadRequest("请填写时区".into()));
    }
    if time_zone.len() > 64 {
        return Err(AppError::BadRequest("时区名称过长".into()));
    }
    if !time_zone
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '_' | '-' | '+'))
    {
        return Err(AppError::BadRequest("时区格式无效".into()));
    }
    Ok(())
}

pub(super) async fn load_app(state: &AppState, id: &str) -> AppResult<PushApp> {
    state
        .db
        .apps()
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("app not found: {id}")))
}

pub(super) fn validate_app_request(name: &str) -> AppResult<()> {
    if name.trim().is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    Ok(())
}

pub(super) fn normalize_identifier(value: Option<String>) -> String {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_default()
}

pub(super) fn validate_channel_request(platform: &str, name: &str, code: &str) -> AppResult<()> {
    if normalize_platform(platform).is_none() {
        return Err(AppError::BadRequest(
            "platform must be xiaomi, huawei, oppo, vivo, honor or meizu".into(),
        ));
    }
    if platform.eq_ignore_ascii_case("honor") {
        return Err(AppError::BadRequest(
            "honor does not support push channel configuration; configure push credentials only"
                .into(),
        ));
    }
    if name.trim().is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    if code.trim().is_empty() {
        return Err(AppError::BadRequest("code is required".into()));
    }
    Ok(())
}

pub(super) fn normalize_channel_code(platform: &str, code: &str) -> String {
    let trimmed = code.trim();
    if matches!(platform, "vivo" | "huawei" | "honor" | "meizu" | "oppo") {
        trimmed.to_uppercase()
    } else {
        trimmed.to_string()
    }
}

pub(super) fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

pub(super) fn validate_username(username: &str) -> AppResult<()> {
    if username.is_empty() {
        return Err(AppError::BadRequest("请填写用户名".into()));
    }
    if username.len() < 3 || username.len() > 32 {
        return Err(AppError::BadRequest("用户名长度需为 3–32 个字符".into()));
    }
    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(AppError::BadRequest(
            "用户名仅支持字母、数字、下划线和短横线".into(),
        ));
    }
    Ok(())
}

pub(super) fn validate_password(password: &str) -> AppResult<()> {
    if password.len() < 6 {
        return Err(AppError::BadRequest("密码至少 6 位".into()));
    }
    if password.len() > 72 {
        return Err(AppError::BadRequest("密码过长".into()));
    }
    Ok(())
}

pub(super) fn validate_admin_credentials(username: &str, password: &str) -> AppResult<()> {
    validate_username(username)?;
    validate_password(password)?;
    Ok(())
}

pub(super) async fn login_response(
    state: &AppState,
    username: &str,
    password: &str,
) -> AppResult<crate::models::AdminLoginResponse> {
    use crate::models::AdminLoginResponse;

    if state.db.admin_users().count().await? == 0 {
        return Err(AppError::BadRequest(
            "尚未创建管理员，请先完成初始化".into(),
        ));
    }

    let user = state
        .db
        .admin_users()
        .find_by_username(username)
        .await?
        .ok_or_else(|| AppError::Unauthorized("用户名或密码错误".into()))?;

    if !auth::verify_password(password, &user.password_hash)? {
        return Err(AppError::Unauthorized("用户名或密码错误".into()));
    }

    let token = auth::create_token(username, user.password_version(), &state.config)?;
    Ok(AdminLoginResponse {
        token,
        username: username.to_string(),
    })
}

pub(super) async fn create_app_from_request(
    state: &AppState,
    body: crate::models::CreateAppRequest,
) -> AppResult<PushAppSummary> {
    use crate::db::NewApp;
    use crate::push::api_auth;

    validate_app_request(&body.name)?;

    let fallback_secs = body.online_push_fallback_secs.unwrap_or(90);
    let cache_secs = body
        .online_message_cache_secs
        .unwrap_or(state.config.online_message_cache_secs);
    let is_default = state.db.apps().count().await? == 0;

    let app = state
        .db
        .apps()
        .create(NewApp {
            id: Uuid::new_v4().to_string(),
            name: body.name.trim().to_string(),
            package_name: normalize_identifier(body.package_name),
            ios_bundle_id: normalize_optional(body.ios_bundle_id),
            harmony_bundle_name: normalize_optional(body.harmony_bundle_name),
            description: normalize_optional(body.description),
            server_base_url: normalize_optional(body.server_base_url),
            push_api_key: api_auth::generate_push_api_key(),
            xiaomi_app_id: normalize_optional(body.xiaomi_app_id),
            xiaomi_app_key: normalize_optional(body.xiaomi_app_key),
            xiaomi_channel_id: normalize_optional(body.xiaomi_channel_id),
            xiaomi_app_secret: normalize_optional(body.xiaomi_app_secret),
            huawei_app_id: normalize_optional(body.huawei_app_id),
            huawei_oauth_client_id: normalize_optional(body.huawei_oauth_client_id),
            huawei_app_secret: normalize_optional(body.huawei_app_secret),
            oppo_app_key: normalize_optional(body.oppo_app_key),
            oppo_app_secret: normalize_optional(body.oppo_app_secret),
            oppo_master_secret: normalize_optional(body.oppo_master_secret),
            vivo_app_id: normalize_optional(body.vivo_app_id),
            vivo_app_key: normalize_optional(body.vivo_app_key),
            vivo_app_secret: normalize_optional(body.vivo_app_secret),
            honor_app_id: normalize_optional(body.honor_app_id),
            honor_oauth_client_id: normalize_optional(body.honor_oauth_client_id),
            honor_app_secret: normalize_optional(body.honor_app_secret),
            meizu_app_id: normalize_optional(body.meizu_app_id),
            meizu_app_key: normalize_optional(body.meizu_app_key),
            meizu_app_secret: normalize_optional(body.meizu_app_secret),
            online_push_fallback_secs: fallback_secs,
            online_message_cache_secs: cache_secs,
            is_default,
        })
        .await?;

    Ok(PushAppSummary::from(app))
}

pub(super) async fn update_app_from_request(
    state: &AppState,
    id: &str,
    body: crate::models::UpdateAppRequest,
) -> AppResult<PushAppConfigView> {
    use crate::db::UpdateApp;

    validate_app_request(&body.name)?;
    load_app(state, id).await?;

    let fallback_secs = body.online_push_fallback_secs.unwrap_or(90);
    let cache_secs = body
        .online_message_cache_secs
        .unwrap_or(state.config.online_message_cache_secs);
    let app = state
        .db
        .apps()
        .update(
            id,
            UpdateApp {
                name: body.name.trim().to_string(),
                package_name: normalize_identifier(body.package_name),
                ios_bundle_id: normalize_optional(body.ios_bundle_id),
                harmony_bundle_name: normalize_optional(body.harmony_bundle_name),
                description: normalize_optional(body.description),
                server_base_url: normalize_optional(body.server_base_url),
                xiaomi_app_id: normalize_optional(body.xiaomi_app_id),
                xiaomi_app_key: normalize_optional(body.xiaomi_app_key),
                xiaomi_channel_id: normalize_optional(body.xiaomi_channel_id),
                xiaomi_app_secret: normalize_optional(body.xiaomi_app_secret),
                huawei_app_id: normalize_optional(body.huawei_app_id),
                huawei_oauth_client_id: normalize_optional(body.huawei_oauth_client_id),
                huawei_app_secret: normalize_optional(body.huawei_app_secret),
                oppo_app_key: normalize_optional(body.oppo_app_key),
                oppo_app_secret: normalize_optional(body.oppo_app_secret),
                oppo_master_secret: normalize_optional(body.oppo_master_secret),
                vivo_app_id: normalize_optional(body.vivo_app_id),
                vivo_app_key: normalize_optional(body.vivo_app_key),
                vivo_app_secret: normalize_optional(body.vivo_app_secret),
                honor_app_id: normalize_optional(body.honor_app_id),
                honor_oauth_client_id: normalize_optional(body.honor_oauth_client_id),
                honor_app_secret: normalize_optional(body.honor_app_secret),
                meizu_app_id: normalize_optional(body.meizu_app_id),
                meizu_app_key: normalize_optional(body.meizu_app_key),
                meizu_app_secret: normalize_optional(body.meizu_app_secret),
                online_push_fallback_secs: fallback_secs,
                online_message_cache_secs: cache_secs,
            },
        )
        .await?;

    state.hub_manager.invalidate(id);
    Ok(PushAppConfigView::from(app))
}

pub(super) fn origin_from_headers(headers: &HeaderMap) -> Option<String> {
    crate::push::init_snippet::origin_from_headers(headers)
}
