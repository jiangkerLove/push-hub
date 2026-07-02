mod apps;
mod auth;
mod channels;
mod helpers;
mod operations;
mod templates;

use axum::Router;

use crate::state::AppState;

pub fn public_router() -> Router<AppState> {
    auth::public_router()
}

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .merge(auth::protected_router())
        .merge(apps::router())
        .merge(templates::router())
        .merge(channels::router())
        .merge(operations::router())
}
