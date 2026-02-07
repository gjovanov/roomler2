use axum::{
    extract::{Query, State, WebSocketUpgrade, ws::{Message, WebSocket}},
    response::Response,
};
use bson::oid::ObjectId;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct WsParams {
    pub token: String,
}

pub async fn ws_upgrade(
    State(state): State<AppState>,
    Query(params): Query<WsParams>,
    ws: WebSocketUpgrade,
) -> Response {
    // Verify JWT before accepting the WebSocket
    let claims = match state.auth.verify_access_token(&params.token) {
        Ok(c) => c,
        Err(_) => {
            return Response::builder()
                .status(401)
                .body("Unauthorized".into())
                .unwrap();
        }
    };

    let user_id = match ObjectId::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return Response::builder()
                .status(400)
                .body("Invalid user ID".into())
                .unwrap();
        }
    };

    ws.on_upgrade(move |socket| handle_socket(socket, state, user_id))
}

async fn handle_socket(socket: WebSocket, state: AppState, user_id: ObjectId) {
    info!(?user_id, "WebSocket connected");

    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));

    // Register connection
    state.ws_storage.add(user_id, sender.clone());

    // Send connected message
    {
        let msg = serde_json::json!({
            "type": "connected",
            "user_id": user_id.to_hex(),
        });
        let mut guard = sender.lock().await;
        let _ = guard.send(Message::text(serde_json::to_string(&msg).unwrap())).await;
    }

    // Message loop
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                handle_client_message(&state, &user_id, &text).await;
            }
            Ok(Message::Ping(data)) => {
                let mut guard = sender.lock().await;
                let _ = guard.send(Message::Pong(data)).await;
            }
            Ok(Message::Close(_)) => {
                break;
            }
            Err(e) => {
                warn!(?user_id, %e, "WebSocket error");
                break;
            }
            _ => {}
        }
    }

    // Cleanup
    state.ws_storage.remove(&user_id, &sender);
    info!(?user_id, "WebSocket disconnected");
}

async fn handle_client_message(state: &AppState, user_id: &ObjectId, text: &str) {
    let parsed: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => return,
    };

    let msg_type = parsed.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let data = parsed.get("data");

    match msg_type {
        "ping" => {
            debug!(?user_id, "WS ping received");
            // Reply with pong via WS text (not protocol-level pong)
            let pong = serde_json::json!({ "type": "pong" });
            super::dispatcher::send_to_user(&state.ws_storage, user_id, &pong).await;
        }
        "typing:start" | "typing:stop" => {
            // Broadcast typing indicator to channel members (except sender)
            if let Some(channel_id_str) = data.and_then(|d| d.get("channel_id")).and_then(|c| c.as_str()) {
                if let Ok(cid) = ObjectId::parse_str(channel_id_str) {
                    if let Ok(member_ids) = state.channels.find_member_user_ids(cid).await {
                        let recipients: Vec<ObjectId> = member_ids
                            .into_iter()
                            .filter(|id| id != user_id)
                            .collect();
                        let event = serde_json::json!({
                            "type": msg_type,
                            "data": {
                                "channel_id": channel_id_str,
                                "user_id": user_id.to_hex(),
                            }
                        });
                        super::dispatcher::broadcast(&state.ws_storage, &recipients, &event).await;
                    }
                }
            }
        }
        "presence:update" => {
            // Broadcast presence to all connected users
            if let Some(presence) = data.and_then(|d| d.get("presence")).and_then(|p| p.as_str()) {
                let all_users = state.ws_storage.all_user_ids();
                let event = serde_json::json!({
                    "type": "presence:update",
                    "data": {
                        "user_id": user_id.to_hex(),
                        "presence": presence,
                    }
                });
                super::dispatcher::broadcast(&state.ws_storage, &all_users, &event).await;
            }
        }
        _ => {
            debug!(?user_id, msg_type, "Unknown WS message type");
        }
    }
}
