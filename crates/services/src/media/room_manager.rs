use bson::oid::ObjectId;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

/// Tracks active conference rooms and their mediasoup state.
/// Will be fully implemented in Phase 5 when mediasoup crate is integrated.
pub struct RoomManager {
    rooms: DashMap<ObjectId, MediaRoom>,
}

#[derive(Debug)]
pub struct MediaRoom {
    pub conference_id: ObjectId,
    // Phase 5: mediasoup Router, transports, producers, consumers
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtpCapabilities {
    // Placeholder for mediasoup RTP capabilities
    pub codecs: Vec<serde_json::Value>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: DashMap::new(),
        }
    }

    pub fn has_room(&self, conference_id: &ObjectId) -> bool {
        self.rooms.contains_key(conference_id)
    }

    pub fn create_room(&self, conference_id: ObjectId) -> bool {
        if self.rooms.contains_key(&conference_id) {
            return false;
        }
        self.rooms.insert(
            conference_id,
            MediaRoom { conference_id },
        );
        true
    }

    pub fn remove_room(&self, conference_id: &ObjectId) -> bool {
        self.rooms.remove(conference_id).is_some()
    }

    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }
}

impl Default for RoomManager {
    fn default() -> Self {
        Self::new()
    }
}
