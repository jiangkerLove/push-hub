use axum::{extract::State, http::HeaderMap, routing::post, Json, Router};

use crate::models::{PushApp, SendPushRequest, SendPushResponse};
use crate::push::api_auth::{extract_push_api_key, verify_push_api_key};
use crate::state::AppState;
use crate::AppError;
use crate::AppResult;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/v1/push", post(send_push))
}

async fn send_push(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SendPushRequest>,
) -> AppResult<Json<SendPushResponse>> {
    let api_key = extract_push_api_key(&headers).ok_or_else(|| {
        AppError::Unauthorized(
            "缺少 Push API Key，请在请求头携带 Authorization: Bearer <push_api_key> 或 X-Push-Hub-Api-Key".into(),
        )
    })?;
    let app = resolve_push_app(&state, body.app_id.as_deref()).await?;
    verify_push_api_key(&app, &api_key)?;

    let hub = state.hub_manager.hub_for_app(&app)?;
    let response = state
        .push_service
        .send_with(
            &state.db,
            hub,
            Some(&app.id),
            app.package_name.clone(),
            app.online_push_fallback_secs,
            body,
        )
        .await?;
    Ok(Json(response))
}

async fn resolve_push_app(state: &AppState, app_id: Option<&str>) -> AppResult<PushApp> {
    if let Some(id) = app_id.map(str::trim).filter(|id| !id.is_empty()) {
        return state
            .db
            .apps()
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("app not found: {id}")));
    }

    let apps = state.db.apps().list().await?;
    if let Some(app) = apps.iter().find(|app| app.is_default) {
        return Ok(app.clone());
    }
    apps.into_iter()
        .next()
        .ok_or_else(|| AppError::BadRequest("no app configured; create one in admin console".into()))
}
