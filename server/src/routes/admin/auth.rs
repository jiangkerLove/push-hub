use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use uuid::Uuid;

use crate::admin::{auth, AdminSession};
use crate::models::{
    AdminBootstrapStatus, AdminLoginRequest, AdminLoginResponse, AdminProfile, AdminSetupRequest,
    AdminUserSummary, ChangePasswordRequest, CreateAdminUserRequest, ResetAdminUserPasswordRequest,
    UpdateDisplayTimeZoneRequest, UpdateMyUsernameRequest, UpdateMyUsernameResponse,
};
use crate::state::AppState;
use crate::{AppError, AppResult};

use super::helpers::{
    admin_user_summary, build_admin_profile, current_user, login_response, require_owner,
    validate_admin_credentials, validate_display_time_zone, validate_username,
};

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/admin/bootstrap", get(bootstrap_status))
        .route("/api/v1/admin/setup", post(setup_admin))
        .route("/api/v1/admin/login", post(login))
}

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/admin/me", get(me))
        .route(
            "/api/v1/admin/settings/display-timezone",
            put(update_display_time_zone),
        )
        .route("/api/v1/admin/me/password", put(change_my_password))
        .route("/api/v1/admin/me/username", put(update_my_username))
        .route(
            "/api/v1/admin/users",
            get(list_admin_users).post(create_admin_user),
        )
        .route("/api/v1/admin/users/{id}", delete(delete_admin_user))
        .route(
            "/api/v1/admin/users/{id}/password",
            put(reset_admin_user_password),
        )
}

async fn bootstrap_status(State(state): State<AppState>) -> AppResult<Json<AdminBootstrapStatus>> {
    let count = state.db.admin_users().count().await?;
    Ok(Json(AdminBootstrapStatus {
        needs_setup: count == 0,
    }))
}

async fn setup_admin(
    State(state): State<AppState>,
    Json(body): Json<AdminSetupRequest>,
) -> AppResult<Json<AdminLoginResponse>> {
    if state.db.admin_users().count().await? > 0 {
        return Err(AppError::BadRequest(
            "管理员已初始化，请直接登录".into(),
        ));
    }

    let username = body.username.trim();
    let password = body.password.trim();
    validate_admin_credentials(username, password)?;

    let password_hash = auth::hash_password(password)?;
    let user = state
        .db
        .admin_users()
        .create(
            &Uuid::new_v4().to_string(),
            username,
            &password_hash,
            true,
        )
        .await?;

    tracing::info!("created initial admin user '{username}'");
    let token = auth::create_token(username, user.password_version(), &state.config)?;
    Ok(Json(AdminLoginResponse {
        token,
        username: username.to_string(),
    }))
}

async fn login(
    State(state): State<AppState>,
    Json(body): Json<AdminLoginRequest>,
) -> AppResult<Json<AdminLoginResponse>> {
    let username = body.username.trim();
    if username.is_empty() || body.password.is_empty() {
        return Err(AppError::BadRequest("请填写用户名和密码".into()));
    }
    Ok(Json(login_response(&state, username, &body.password).await?))
}

async fn list_admin_users(
    State(state): State<AppState>,
    Extension(session): Extension<AdminSession>,
) -> AppResult<Json<Vec<AdminUserSummary>>> {
    require_owner(&state, &session.username).await?;
    let users = state.db.admin_users().list().await?;
    Ok(Json(users.into_iter().map(admin_user_summary).collect()))
}

async fn create_admin_user(
    State(state): State<AppState>,
    Extension(session): Extension<AdminSession>,
    Json(body): Json<CreateAdminUserRequest>,
) -> AppResult<Json<AdminUserSummary>> {
    require_owner(&state, &session.username).await?;

    let username = body.username.trim();
    let password = body.password.trim();
    validate_admin_credentials(username, password)?;

    if state
        .db
        .admin_users()
        .find_by_username(username)
        .await?
        .is_some()
    {
        return Err(AppError::BadRequest("用户名已存在".into()));
    }

    let password_hash = auth::hash_password(password)?;
    let user = state
        .db
        .admin_users()
        .create(
            &Uuid::new_v4().to_string(),
            username,
            &password_hash,
            false,
        )
        .await?;

    tracing::info!("owner '{}' created sub-account '{username}'", session.username);
    Ok(Json(admin_user_summary(user)))
}

async fn delete_admin_user(
    State(state): State<AppState>,
    Extension(session): Extension<AdminSession>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    require_owner(&state, &session.username).await?;

    let target = state
        .db
        .admin_users()
        .find_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound("用户不存在".into()))?;

    if target.username == session.username {
        return Err(AppError::BadRequest("不能删除当前登录账号".into()));
    }

    state.db.admin_users().delete(&id).await?;
    tracing::info!(
        "owner '{}' deleted sub-account '{}'",
        session.username,
        target.username
    );
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn change_my_password(
    State(state): State<AppState>,
    Extension(session): Extension<AdminSession>,
    Json(body): Json<ChangePasswordRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let user = current_user(&state, &session.username).await?;

    if body.current_password.is_empty() || body.new_password.trim().is_empty() {
        return Err(AppError::BadRequest("请填写当前密码和新密码".into()));
    }
    validate_admin_credentials(&user.username, body.new_password.trim())?;

    if !auth::verify_password(&body.current_password, &user.password_hash)? {
        return Err(AppError::BadRequest("当前密码不正确".into()));
    }

    let password_hash = auth::hash_password(body.new_password.trim())?;
    state
        .db
        .admin_users()
        .update_password(&user.id, &password_hash)
        .await?;

    tracing::info!("admin user '{}' changed password", session.username);
    Ok(Json(serde_json::json!({ "ok": true, "require_relogin": true })))
}

async fn update_my_username(
    State(state): State<AppState>,
    Extension(session): Extension<AdminSession>,
    Json(body): Json<UpdateMyUsernameRequest>,
) -> AppResult<Json<UpdateMyUsernameResponse>> {
    let user = current_user(&state, &session.username).await?;
    let username = body.username.trim();
    validate_username(username)?;

    if username == user.username {
        let profile = build_admin_profile(&state, &user.username).await?;
        let token = auth::create_token(&user.username, user.password_version(), &state.config)?;
        return Ok(Json(UpdateMyUsernameResponse {
            token,
            username: profile.username,
            is_owner: profile.is_owner,
            display_time_zone: profile.display_time_zone,
        }));
    }

    if state
        .db
        .admin_users()
        .find_by_username(username)
        .await?
        .is_some()
    {
        return Err(AppError::BadRequest("用户名已存在".into()));
    }

    state.db.admin_users().update_username(&user.id, username).await?;

    let updated = state
        .db
        .admin_users()
        .find_by_id(&user.id)
        .await?
        .ok_or_else(|| AppError::NotFound("用户不存在".into()))?;

    tracing::info!(
        "admin user '{}' renamed to '{username}'",
        session.username
    );

    let profile = build_admin_profile(&state, &updated.username).await?;
    let token = auth::create_token(&updated.username, updated.password_version(), &state.config)?;
    Ok(Json(UpdateMyUsernameResponse {
        token,
        username: profile.username,
        is_owner: profile.is_owner,
        display_time_zone: profile.display_time_zone,
    }))
}

async fn reset_admin_user_password(
    State(state): State<AppState>,
    Extension(session): Extension<AdminSession>,
    Path(id): Path<String>,
    Json(body): Json<ResetAdminUserPasswordRequest>,
) -> AppResult<Json<serde_json::Value>> {
    require_owner(&state, &session.username).await?;

    let target = state
        .db
        .admin_users()
        .find_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound("用户不存在".into()))?;

    if target.is_owner {
        return Err(AppError::BadRequest(
            "请使用「修改密码」功能修改主账号密码".into(),
        ));
    }

    let password = body.new_password.trim();
    if password.is_empty() {
        return Err(AppError::BadRequest("请填写新密码".into()));
    }
    validate_admin_credentials(&target.username, password)?;

    let password_hash = auth::hash_password(password)?;
    state
        .db
        .admin_users()
        .update_password(&target.id, &password_hash)
        .await?;

    tracing::info!(
        "owner '{}' reset password for sub-account '{}'",
        session.username,
        target.username
    );
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn me(
    State(state): State<AppState>,
    Extension(session): Extension<AdminSession>,
) -> AppResult<Json<AdminProfile>> {
    Ok(Json(build_admin_profile(&state, &session.username).await?))
}

async fn update_display_time_zone(
    State(state): State<AppState>,
    Extension(session): Extension<AdminSession>,
    Json(body): Json<UpdateDisplayTimeZoneRequest>,
) -> AppResult<Json<AdminProfile>> {
    let user = state
        .db
        .admin_users()
        .find_by_username(&session.username)
        .await?
        .ok_or_else(|| AppError::Unauthorized("用户不存在".into()))?;
    if !user.is_owner {
        return Err(AppError::Forbidden("仅主账号可修改展示时区".into()));
    }

    let time_zone = body.display_time_zone.trim();
    validate_display_time_zone(time_zone)?;
    state
        .db
        .admin_users()
        .update_owner_display_time_zone(time_zone)
        .await?;

    Ok(Json(build_admin_profile(&state, &session.username).await?))
}
