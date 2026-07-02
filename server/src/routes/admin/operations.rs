use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};

use crate::models::{
    Device, PushJob, PushJobDetail, PushStatsOverview, SendPushRequest, SendPushResponse,
};
use crate::state::AppState;
use crate::{AppError, AppResult};

use super::helpers::{load_app, ListDevicesQuery, PushStatsQuery};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/admin/apps/{id}/devices", get(list_devices))
        .route("/api/v1/admin/apps/{id}/push", post(send_push))
        .route("/api/v1/admin/apps/{id}/push/stats", get(get_push_stats))
        .route("/api/v1/admin/apps/{id}/push/jobs", get(list_push_jobs))
        .route(
            "/api/v1/admin/apps/{id}/push/jobs/{job_id}",
            get(get_push_job),
        )
}

async fn list_devices(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<ListDevicesQuery>,
) -> AppResult<Json<Vec<Device>>> {
    load_app(&state, &id).await?;
    let devices = state
        .db
        .devices()
        .list_by_app_id(&id, query.limit.min(200), query.offset)
        .await?;
    Ok(Json(devices))
}

async fn send_push(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<SendPushRequest>,
) -> AppResult<Json<SendPushResponse>> {
    let app = load_app(&state, &id).await?;
    let hub = state.hub_manager.hub_for_app(&app)?;
    let response = state
        .push_service
        .send_traced(
            state.db.clone(),
            hub,
            &id,
            app.package_name.clone(),
            app.online_push_fallback_secs,
            body,
        )
        .await?;
    Ok(Json(response))
}

async fn get_push_stats(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<PushStatsQuery>,
) -> AppResult<Json<PushStatsOverview>> {
    let app = load_app(&state, &id).await?;
    let mut stats = state.db.push_trace().stats(&id, query.days).await?;
    stats.devices = state
        .db
        .devices()
        .stats_for_app(&id, query.days, app.online_push_fallback_secs)
        .await?;
    stats.template_count = state.db.templates().list_by_app_id(&id).await?.len() as i64;
    Ok(Json(stats))
}

async fn list_push_jobs(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<ListDevicesQuery>,
) -> AppResult<Json<Vec<PushJob>>> {
    load_app(&state, &id).await?;
    let jobs = state
        .db
        .push_trace()
        .list_jobs(&id, query.limit.min(200), query.offset)
        .await?;
    Ok(Json(jobs))
}

async fn get_push_job(
    State(state): State<AppState>,
    Path((id, job_id)): Path<(String, String)>,
) -> AppResult<Json<PushJobDetail>> {
    load_app(&state, &id).await?;
    let detail = state
        .db
        .push_trace()
        .get_job_detail(&job_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("push job not found: {job_id}")))?;
    if detail.job.app_id != id {
        return Err(AppError::NotFound(format!("push job not found: {job_id}")));
    }
    Ok(Json(detail))
}
