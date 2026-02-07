use bson::oid::ObjectId;
use dashmap::DashMap;
use futures::stream::SplitSink;
use axum::extract::ws::{Message, WebSocket};
use std::sync::Arc;
use tokio::sync::Mutex;

pub type WsSender = Arc<Mutex<SplitSink<WebSocket, Message>>>;

/// Tracks all active WebSocket connections by user ID.
/// Each user can have multiple connections (multiple tabs/devices).
pub struct WsStorage {
    connections: DashMap<ObjectId, Vec<WsSender>>,
}

impl WsStorage {
    pub fn new() -> Self {
        Self {
            connections: DashMap::new(),
        }
    }

    pub fn add(&self, user_id: ObjectId, sender: WsSender) {
        self.connections
            .entry(user_id)
            .or_default()
            .push(sender);
    }

    pub fn remove(&self, user_id: &ObjectId, sender: &WsSender) {
        if let Some(mut senders) = self.connections.get_mut(user_id) {
            senders.retain(|s| !Arc::ptr_eq(s, sender));
            if senders.is_empty() {
                drop(senders);
                self.connections.remove(user_id);
            }
        }
    }

    pub fn get_senders(&self, user_id: &ObjectId) -> Vec<WsSender> {
        self.connections
            .get(user_id)
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    pub fn all_user_ids(&self) -> Vec<ObjectId> {
        self.connections.iter().map(|r| *r.key()).collect()
    }

    pub fn connection_count(&self) -> usize {
        self.connections
            .iter()
            .map(|r| r.value().len())
            .sum()
    }
}

impl Default for WsStorage {
    fn default() -> Self {
        Self::new()
    }
}
