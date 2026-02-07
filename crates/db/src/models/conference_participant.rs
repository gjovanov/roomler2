use bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConferenceParticipant {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub tenant_id: ObjectId,
    pub conference_id: ObjectId,
    pub user_id: Option<ObjectId>,
    pub display_name: String,
    pub email: Option<String>,
    #[serde(default)]
    pub is_external: bool,
    #[serde(default)]
    pub role: ParticipantRole,
    #[serde(default)]
    pub sessions: Vec<ParticipantSession>,
    #[serde(default)]
    pub is_muted: bool,
    #[serde(default)]
    pub is_video_on: bool,
    #[serde(default)]
    pub is_screen_sharing: bool,
    #[serde(default)]
    pub is_hand_raised: bool,
    #[serde(default)]
    pub total_duration: u32,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantRole {
    Organizer,
    CoOrganizer,
    Presenter,
    #[default]
    Attendee,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantSession {
    pub joined_at: DateTime,
    pub left_at: Option<DateTime>,
    pub duration: Option<u32>,
    pub device_type: String,
}

impl ConferenceParticipant {
    pub const COLLECTION: &'static str = "conference_participants";
}
