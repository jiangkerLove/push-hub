pub mod admin;
pub mod devices;
pub mod health;
pub mod online;
pub mod push;

use axum::Router;
use axum::middleware;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::admin::require_admin;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let protected_admin = admin::protected_router()
        .route_layer(middleware::from_fn_with_state(state.clone(), require_admin));

    let admin_api = admin::public_router().merge(protected_admin);

    Router::new()
        .merge(health::router())
        .merge(devices::router())
        .merge(online::router())
        .merge(push::router())
        .merge(admin_api)
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}
