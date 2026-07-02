use axum::{
    extract::{Path, State},
    routing::{delete, get, put},
    Json, Router,
};
use uuid::Uuid;

use crate::db::{NewTemplate, UpdateTemplate};
use crate::models::{
    validate_template_fields, CreateTemplateRequest, PushTemplate, UpdateTemplateRequest,
};
use crate::state::AppState;
use crate::AppResult;

use super::helpers::load_app;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/admin/apps/{id}/templates",
            get(list_templates).post(create_template),
        )
        .route(
            "/api/v1/admin/templates/{id}",
            put(update_template).delete(delete_template),
        )
}

async fn list_templates(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<PushTemplate>>> {
    load_app(&state, &id).await?;
    let templates = state.db.templates().list_by_app_id(&id).await?;
    Ok(Json(templates))
}

async fn create_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<CreateTemplateRequest>,
) -> AppResult<Json<PushTemplate>> {
    load_app(&state, &id).await?;
    validate_template_fields(
        &body.name,
        body.kind,
        body.content_mode,
        &body.title,
        &body.body,
    )?;
    let template = state
        .db
        .templates()
        .create(NewTemplate {
            id: Uuid::new_v4().to_string(),
            app_id: id,
            name: body.name,
            kind: body.kind.as_db().to_string(),
            content_mode: if body.kind.is_public() {
                crate::models::TemplateContentMode::Compose.as_db().to_string()
            } else {
                body.content_mode.as_db().to_string()
            },
            title: if body.kind.is_public() || body.content_mode.is_free() {
                String::new()
            } else {
                body.title
            },
            body: if body.kind.is_public() || body.content_mode.is_free() {
                String::new()
            } else {
                body.body
            },
            channels: body.channels,
            click_action: Default::default(),
            message_cache_days: body.message_cache_days,
        })
        .await?;

    Ok(Json(template))
}

async fn update_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateTemplateRequest>,
) -> AppResult<Json<PushTemplate>> {
    validate_template_fields(
        &body.name,
        body.kind,
        body.content_mode,
        &body.title,
        &body.body,
    )?;
    let template = state
        .db
        .templates()
        .update(
            &id,
            UpdateTemplate {
                name: body.name,
                kind: body.kind.as_db().to_string(),
                content_mode: if body.kind.is_public() {
                    crate::models::TemplateContentMode::Compose.as_db().to_string()
                } else {
                    body.content_mode.as_db().to_string()
                },
                title: if body.kind.is_public() || body.content_mode.is_free() {
                    String::new()
                } else {
                    body.title
                },
                body: if body.kind.is_public() || body.content_mode.is_free() {
                    String::new()
                } else {
                    body.body
                },
                channels: body.channels,
                click_action: Default::default(),
                message_cache_days: body.message_cache_days,
            },
        )
        .await?;

    Ok(Json(template))
}

async fn delete_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    state.db.templates().delete(&id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}
