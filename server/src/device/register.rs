//! 设备注册与 app 归属校验（SDK `POST /api/v1/devices` 使用）。

use crate::models::{PushApp, RegisterDeviceRequest};
use crate::state::AppState;
use crate::{AppError, AppResult};

const ANDROID_PLATFORMS: &[&str] = &[
    "xiaomi", "huawei", "oppo", "vivo", "honor", "meizu", "online",
];

pub async fn resolve_registration_app(
    state: &AppState,
    body: &RegisterDeviceRequest,
) -> AppResult<PushApp> {
    let client_identifier = body.package_name.trim();
    if client_identifier.is_empty() {
        return Err(AppError::BadRequest("package_name is required".into()));
    }

    let platform = body.platform.trim().to_lowercase();
    if platform.is_empty() {
        return Err(AppError::BadRequest("platform is required".into()));
    }

    if let Some(app_id) = body
        .app_id
        .as_ref()
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
    {
        let app = state
            .db
            .apps()
            .find_by_id(app_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("app not found: {app_id}")))?;
        validate_client_identifier(&app, client_identifier, &platform)?;
        return Ok(app);
    }

    let matches = state
        .db
        .apps()
        .list()
        .await?
        .into_iter()
        .filter(|app| app.package_name.trim() == client_identifier)
        .collect::<Vec<_>>();

    match matches.len() {
        0 => Err(AppError::BadRequest(
            "app_id is required when no app matches the client identifier".into(),
        )),
        1 => {
            validate_client_identifier(&matches[0], client_identifier, &platform)?;
            Ok(matches[0].clone())
        }
        _ => Err(AppError::BadRequest(
            "multiple apps share this identifier; app_id is required".into(),
        )),
    }
}

fn validate_client_identifier(
    app: &PushApp,
    client_identifier: &str,
    platform: &str,
) -> AppResult<()> {
    if ANDROID_PLATFORMS.contains(&platform) {
        if !app.package_name.is_empty() && app.package_name != client_identifier {
            return Err(AppError::BadRequest(format!(
                "client identifier '{client_identifier}' does not match app's Android package '{}'",
                app.package_name
            )));
        }
        return Ok(());
    }

    if platform == "ios" {
        if let Some(expected) = app.ios_bundle_id.as_ref().filter(|v| !v.is_empty()) {
            if expected != client_identifier {
                return Err(AppError::BadRequest(format!(
                    "client identifier '{client_identifier}' does not match app's iOS bundle id '{expected}'"
                )));
            }
        }
        return Ok(());
    }

    if platform == "harmony" || platform == "harmonyos" {
        if let Some(expected) = app
            .harmony_bundle_name
            .as_ref()
            .filter(|v| !v.is_empty())
        {
            if expected != client_identifier {
                return Err(AppError::BadRequest(format!(
                    "client identifier '{client_identifier}' does not match app's Harmony bundle '{expected}'"
                )));
            }
        }
        return Ok(());
    }

    Ok(())
}
