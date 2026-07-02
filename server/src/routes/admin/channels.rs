use axum::{
    extract::{Path, State},
    routing::{delete, get, put},
    Json, Router,
};
use uuid::Uuid;

use crate::db::{NewPushChannel, UpdatePushChannel};
use crate::models::{
    normalize_platform, CreatePushChannelRequest, PushChannel, UpdatePushChannelRequest,
};
use crate::state::AppState;
use crate::{AppError, AppResult};

use super::helpers::{
    load_app, normalize_channel_code, normalize_optional, validate_channel_request,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/admin/apps/{id}/channels",
            get(list_channels).post(create_channel),
        )
        .route(
            "/api/v1/admin/channels/{id}",
            put(update_channel).delete(delete_channel),
        )
}

async fn list_channels(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<PushChannel>>> {
    load_app(&state, &id).await?;
    let channels = state.db.channels().list_by_app_id(&id).await?;
    Ok(Json(channels))
}

async fn create_channel(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<CreatePushChannelRequest>,
) -> AppResult<Json<PushChannel>> {
    load_app(&state, &id).await?;
    validate_channel_request(&body.platform, &body.name, &body.code)?;
    let platform = normalize_platform(&body.platform)
        .ok_or_else(|| {
            AppError::BadRequest(
                "platform must be xiaomi, huawei, oppo, vivo, honor or meizu".into(),
            )
        })?
        .to_string();

    let channel = state
        .db
        .channels()
        .create(NewPushChannel {
            id: Uuid::new_v4().to_string(),
            app_id: id,
            code: normalize_channel_code(&platform, &body.code),
            platform,
            name: body.name.trim().to_string(),
            description: normalize_optional(body.description),
            is_default: body.is_default,
        })
        .await?;

    Ok(Json(channel))
}

async fn update_channel(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdatePushChannelRequest>,
) -> AppResult<Json<PushChannel>> {
    validate_channel_request(&body.platform, &body.name, &body.code)?;
    state
        .db
        .channels()
        .find_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("channel not found: {id}")))?;
    let platform = normalize_platform(&body.platform)
        .ok_or_else(|| {
            AppError::BadRequest(
                "platform must be xiaomi, huawei, oppo, vivo, honor or meizu".into(),
            )
        })?
        .to_string();

    let channel = state
        .db
        .channels()
        .update(
            &id,
            UpdatePushChannel {
                code: normalize_channel_code(&platform, &body.code),
                platform,
                name: body.name.trim().to_string(),
                description: normalize_optional(body.description),
                is_default: body.is_default,
            },
        )
        .await?;

    Ok(Json(channel))
}

async fn delete_channel(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    state.db.channels().delete(&id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}
