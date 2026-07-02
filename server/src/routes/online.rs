use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::mpsc;

use crate::models::{AckOnlineMessagesRequest, OutboxMessage};
use crate::state::AppState;
use crate::AppError;
use crate::AppResult;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/online/messages", get(fetch_messages).post(ack_messages))
        .route("/api/v1/online/ws", get(online_ws))
}

#[derive(Debug, Deserialize)]
struct FetchOnlineMessagesQuery {
    push_token: String,
    #[serde(default = "default_limit")]
    limit: i64,
}

#[derive(Debug, Deserialize)]
struct OnlineWsQuery {
    push_token: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum WsClientMessage {
    Ack {
        #[serde(default)]
        ids: Vec<String>,
        #[serde(default)]
        acks: Vec<crate::models::OnlineMessageAck>,
    },
    Ping,
}

fn default_limit() -> i64 {
    20
}

async fn fetch_messages(
    State(state): State<AppState>,
    Query(query): Query<FetchOnlineMessagesQuery>,
) -> AppResult<Json<Vec<OutboxMessage>>> {
    let push_token = query.push_token.trim();
    if push_token.is_empty() {
        return Err(AppError::BadRequest("push_token is required".into()));
    }

    state.db.devices().touch_online(push_token).await?;

    let messages = state
        .db
        .outbox()
        .fetch_pending(push_token, query.limit)
        .await?;

    Ok(Json(messages))
}

async fn ack_messages(
    State(state): State<AppState>,
    Json(body): Json<AckOnlineMessagesRequest>,
) -> AppResult<Json<AckOnlineMessagesResponse>> {
    let push_token = body.push_token.trim();
    if push_token.is_empty() {
        return Err(AppError::BadRequest("push_token is required".into()));
    }
    let acks = body.normalized_acks();
    if acks.is_empty() {
        return Err(AppError::BadRequest("acks cannot be empty".into()));
    }

    let ids: Vec<String> = acks.iter().map(|ack| ack.id.clone()).collect();
    let acked = state.db.outbox().ack(push_token, &ids).await?;
    let _ = crate::push::trace::record_online_delivery(&state.db, &acks).await;
    Ok(Json(AckOnlineMessagesResponse { acked }))
}

async fn online_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<OnlineWsQuery>,
) -> Response {
    let push_token = query.push_token.trim().to_string();
    if push_token.is_empty() {
        return AppError::BadRequest("push_token is required".into()).into_response();
    }

    ws.on_upgrade(move |socket| handle_online_ws(socket, state, push_token))
}

async fn handle_online_ws(socket: WebSocket, state: AppState, push_token: String) {
    if state.db.devices().touch_online(&push_token).await.is_err() {
        return;
    }

    let subscription = state.online_hub.subscribe(&push_token).await;
    let conn_id = subscription.conn_id;
    let mut hub_rx = subscription.rx;

    let pending = state
        .db
        .outbox()
        .fetch_pending(&push_token, 50)
        .await
        .unwrap_or_default();

    let (mut ws_tx, mut ws_rx) = socket.split();
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<Message>();

    let writer = tokio::spawn(async move {
        while let Some(msg) = out_rx.recv().await {
            if ws_tx.send(msg).await.is_err() {
                break;
            }
        }
    });

    for message in pending {
        let text = ws_server_payload(&message);
        if out_tx.send(Message::Text(text.into())).is_err() {
            writer.abort();
            state.online_hub.unsubscribe(&push_token, conn_id).await;
            return;
        }
    }

    let hub_out = out_tx.clone();
    let hub_forward = tokio::spawn(async move {
        while let Some(text) = hub_rx.recv().await {
            if hub_out
                .send(Message::Text(text.into()))
                .is_err()
            {
                break;
            }
        }
    });

    let ping_out = out_tx.clone();
    let ping_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        interval.tick().await;
        loop {
            interval.tick().await;
            if ping_out.send(Message::Ping(vec![].into())).is_err() {
                break;
            }
        }
    });

    let db = state.db.clone();
    let token = push_token.clone();
    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(Message::Text(text)) => {
                if let Ok(client_msg) = serde_json::from_str::<WsClientMessage>(&text) {
                    match client_msg {
                        WsClientMessage::Ack { ids, acks } => {
                            let normalized = if !acks.is_empty() {
                                acks
                            } else {
                                ids.into_iter()
                                    .map(|id| crate::models::OnlineMessageAck {
                                        id,
                                        displayed: true,
                                        reason: None,
                                    })
                                    .collect()
                            };
                            if normalized.is_empty() {
                                continue;
                            }
                            let outbox_ids: Vec<String> =
                                normalized.iter().map(|ack| ack.id.clone()).collect();
                            let _ = db.outbox().ack(&token, &outbox_ids).await;
                            let _ = crate::push::trace::record_online_delivery(
                                db.as_ref(),
                                &normalized,
                            )
                            .await;
                            let _ = db.devices().touch_online(&token).await;
                        }
                        WsClientMessage::Ping => {
                            let _ = out_tx.send(Message::Text(r#"{"type":"pong"}"#.into()));
                            let _ = db.devices().touch_online(&token).await;
                        }
                    }
                }
            }
            Ok(Message::Pong(_)) => {
                let _ = db.devices().touch_online(&token).await;
            }
            Ok(Message::Close(_)) => break,
            Err(_) => break,
            _ => {}
        }
    }

    ping_task.abort();
    hub_forward.abort();
    writer.abort();
    state.online_hub.unsubscribe(&push_token, conn_id).await;
}

fn ws_server_payload(message: &OutboxMessage) -> String {
    message.to_online_ws_payload()
}

#[derive(Debug, serde::Serialize)]
struct AckOnlineMessagesResponse {
    acked: usize,
}
