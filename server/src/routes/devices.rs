use axum::{
    extract::State,
    routing::post,
    Json, Router,
};

use crate::db::NewDevice;
use crate::models::{Device, RegisterDeviceRequest};
use crate::device::resolve_registration_app;
use crate::state::AppState;
use crate::AppResult;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/v1/devices", post(register_device))
}

async fn register_device(
    State(state): State<AppState>,
    Json(body): Json<RegisterDeviceRequest>,
) -> AppResult<Json<Device>> {
    if body.push_token.trim().is_empty() {
        return Err(crate::AppError::BadRequest("push_token is required".into()));
    }

    let app = resolve_registration_app(&state, &body).await?;
    let online_token = body.normalized_online_token();
    let device_id = body
        .device_id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(str::to_string);
    let device = state
        .db
        .devices()
        .upsert(NewDevice {
            app_id: app.id,
            device_id,
            package_name: body.package_name.trim().to_string(),
            platform: body.platform.to_lowercase(),
            push_token: body.push_token,
            online_token,
        })
        .await?;

    Ok(Json(device))
}
