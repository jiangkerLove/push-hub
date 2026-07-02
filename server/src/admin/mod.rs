pub mod auth;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::state::AppState;
use crate::AppError;

#[derive(Clone, Debug)]
pub struct AdminSession {
    pub username: String,
}

pub async fn require_admin(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;

    let claims = auth::verify_token(token, &state.config)?;
    let user = state
        .db
        .admin_users()
        .find_by_username(&claims.sub)
        .await?
        .ok_or_else(|| AppError::Unauthorized("invalid or expired token".into()))?;

    if user.password_version() != claims.pwd_at {
        return Err(AppError::Unauthorized(
            "session expired, please login again".into(),
        ));
    }

    req.extensions_mut().insert(AdminSession {
        username: claims.sub,
    });
    Ok(next.run(req).await)
}
