use axum::{
    extract::{Path, State},
    http::HeaderMap,
    routing::{get, post},
    Json, Router,
};

use crate::models::{
    AppInitSnippet, CreateAppRequest, PushAppConfigView, PushAppSummary, UpdateAppRequest,
    ValidateAppCredentialsRequest, ValidateAppCredentialsResponse,
};
use crate::push::{credentials, init_snippet};
use crate::state::AppState;
use crate::{AppError, AppResult};

use super::helpers::{
    create_app_from_request, load_app, origin_from_headers, update_app_from_request,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/admin/apps", get(list_apps).post(create_app))
        .route(
            "/api/v1/admin/apps/{id}",
            get(get_app).put(update_app).delete(delete_app),
        )
        .route(
            "/api/v1/admin/apps/{id}/validate-credentials",
            post(validate_app_credentials),
        )
        .route("/api/v1/admin/apps/{id}/default", post(set_default_app))
        .route("/api/v1/admin/apps/{id}/init-snippet", get(get_init_snippet))
}

async fn list_apps(State(state): State<AppState>) -> AppResult<Json<Vec<PushAppSummary>>> {
    let apps = state.db.apps().list().await?;
    Ok(Json(apps.into_iter().map(PushAppSummary::from).collect()))
}

async fn get_app(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<PushAppConfigView>> {
    let app = load_app(&state, &id).await?;
    Ok(Json(PushAppConfigView::from(app)))
}

async fn create_app(
    State(state): State<AppState>,
    Json(body): Json<CreateAppRequest>,
) -> AppResult<Json<PushAppSummary>> {
    Ok(Json(create_app_from_request(&state, body).await?))
}

async fn validate_app_credentials(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<ValidateAppCredentialsRequest>,
) -> AppResult<Json<ValidateAppCredentialsResponse>> {
    let app = load_app(&state, &id).await?;
    let results = credentials::validate_app_credentials(&app, &body).await?;
    Ok(Json(ValidateAppCredentialsResponse { results }))
}

async fn update_app(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateAppRequest>,
) -> AppResult<Json<PushAppConfigView>> {
    Ok(Json(update_app_from_request(&state, &id, body).await?))
}

async fn delete_app(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let app = load_app(&state, &id).await?;
    if app.is_default {
        return Err(AppError::BadRequest("cannot delete default app".into()));
    }
    state.db.apps().delete(&id).await?;
    state.hub_manager.invalidate(&id);
    Ok(Json(serde_json::json!({ "deleted": true })))
}

async fn set_default_app(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<PushAppSummary>> {
    load_app(&state, &id).await?;
    state.db.apps().set_default(&id).await?;
    let app = load_app(&state, &id).await?;
    Ok(Json(PushAppSummary::from(app)))
}

async fn get_init_snippet(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> AppResult<Json<AppInitSnippet>> {
    let app = load_app(&state, &id).await?;
    let request_origin = origin_from_headers(&headers);
    Ok(Json(init_snippet::generate(
        &app,
        request_origin.as_deref(),
    )))
}
