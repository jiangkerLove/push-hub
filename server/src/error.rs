use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("push provider error: {0}")]
    Push(String),

    #[error("http client error: {0}")]
    Http(#[from] reqwest::Error),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::Config(_)
            | AppError::Database(_)
            | AppError::Push(_)
            | AppError::Http(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let message = match &self {
            AppError::Config(err) => {
                tracing::error!(error = %err, "configuration error");
                "internal server error".to_string()
            }
            AppError::Database(err) => {
                tracing::error!(error = %err, "database error");
                "internal server error".to_string()
            }
            AppError::Http(err) => {
                tracing::error!(error = %err, "http client error");
                "internal server error".to_string()
            }
            AppError::Push(err) => {
                tracing::warn!(error = %err, "push provider error");
                err.to_string()
            }
            _ => self.to_string(),
        };

        (status, Json(ErrorBody { error: message })).into_response()
    }
}
