use bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conference {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub tenant_id: ObjectId,
    pub channel_id: Option<ObjectId>,
    pub subject: String,
    pub description: Option<String>,
    #[serde(default)]
    pub conference_type: ConferenceType,
    #[serde(default)]
    pub status: ConferenceStatus,
    pub start_time: Option<DateTime>,
    pub end_time: Option<DateTime>,
    pub actual_start_time: Option<DateTime>,
    pub actual_end_time: Option<DateTime>,
    pub duration: Option<u32>,
    pub timezone: Option<String>,
    pub recurrence: Option<Recurrence>,
    pub join_url: String,
    pub meeting_code: String,
    pub passcode: Option<String>,
    #[serde(default)]
    pub waiting_room: bool,
    pub organizer_id: ObjectId,
    #[serde(default)]
    pub co_organizer_ids: Vec<ObjectId>,
    #[serde(default)]
    pub settings: ConferenceSettings,
    #[serde(default)]
    pub participant_count: u32,
    #[serde(default)]
    pub peak_participant_count: u32,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConferenceType {
    #[default]
    Instant,
    Scheduled,
    Recurring,
    Persistent,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConferenceStatus {
    #[default]
    Scheduled,
    InProgress,
    Ended,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recurrence {
    pub pattern: RecurrencePattern,
    pub interval: u32,
    pub days_of_week: Option<Vec<u8>>,
    pub end_date: Option<DateTime>,
    pub max_occurrences: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecurrencePattern {
    Daily,
    Weekly,
    Monthly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConferenceSettings {
    #[serde(default = "bool_true")]
    pub host_video: bool,
    #[serde(default = "bool_true")]
    pub participant_video: bool,
    #[serde(default)]
    pub join_before_host: bool,
    #[serde(default)]
    pub mute_on_entry: bool,
    #[serde(default = "bool_true")]
    pub chat_enabled: bool,
    #[serde(default = "bool_true")]
    pub screen_share_enabled: bool,
    #[serde(default)]
    pub auto_recording: RecordingMode,
    #[serde(default)]
    pub auto_transcription: bool,
    pub max_participants: Option<u32>,
}

impl Default for ConferenceSettings {
    fn default() -> Self {
        Self {
            host_video: true,
            participant_video: true,
            join_before_host: false,
            mute_on_entry: false,
            chat_enabled: true,
            screen_share_enabled: true,
            auto_recording: RecordingMode::default(),
            auto_transcription: false,
            max_participants: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecordingMode {
    #[default]
    None,
    Local,
    Cloud,
}

fn bool_true() -> bool {
    true
}

impl Conference {
    pub const COLLECTION: &'static str = "conferences";
}
