use bson::oid::ObjectId;
use futures::SinkExt;
use axum::extract::ws::Message;
use tracing::{debug, warn};

use super::storage::WsStorage;

/// Broadcasts a JSON message to all connections of the specified users.
pub async fn broadcast(
    ws_storage: &WsStorage,
    user_ids: &[ObjectId],
    message: &serde_json::Value,
) {
    let text = serde_json::to_string(message).unwrap_or_default();

    for user_id in user_ids {
        let senders = ws_storage.get_senders(user_id);
        for sender in senders {
            let text = text.clone();
            let mut guard = sender.lock().await;
            if let Err(e) = guard.send(Message::text(text)).await {
                warn!(?user_id, %e, "Failed to send WS message");
            } else {
                debug!(?user_id, "WS message sent");
            }
        }
    }
}

/// Sends a JSON message to a specific user's connections.
pub async fn send_to_user(
    ws_storage: &WsStorage,
    user_id: &ObjectId,
    message: &serde_json::Value,
) {
    broadcast(ws_storage, &[*user_id], message).await;
}
