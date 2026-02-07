use bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub tenant_id: ObjectId,
    pub parent_id: Option<ObjectId>,
    pub channel_type: ChannelType,
    pub name: String,
    pub path: String,
    pub topic: Option<TopicInfo>,
    pub purpose: Option<String>,
    pub icon: Option<String>,
    #[serde(default)]
    pub position: u32,
    #[serde(default)]
    pub is_private: bool,
    #[serde(default)]
    pub is_archived: bool,
    #[serde(default)]
    pub is_read_only: bool,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default)]
    pub permission_overwrites: Vec<PermissionOverwrite>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub media_settings: Option<MediaSettings>,
    pub creator_id: ObjectId,
    pub last_message_id: Option<ObjectId>,
    pub last_activity_at: Option<DateTime>,
    #[serde(default)]
    pub member_count: u32,
    #[serde(default)]
    pub message_count: u64,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub deleted_at: Option<DateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    Category,
    #[default]
    Text,
    Voice,
    Announcement,
    Forum,
    Stage,
    Dm,
    GroupDm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicInfo {
    pub value: Option<String>,
    pub set_by: Option<ObjectId>,
    pub set_at: Option<DateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionOverwrite {
    pub target_id: ObjectId,
    pub target_type: OverwriteTarget,
    pub allow: u64,
    pub deny: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverwriteTarget {
    Role,
    User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaSettings {
    #[serde(default = "default_bitrate")]
    pub bitrate: u32,
    #[serde(default)]
    pub user_limit: u32,
    #[serde(default)]
    pub video_quality_mode: VideoQuality,
}

fn default_bitrate() -> u32 {
    256_000
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VideoQuality {
    #[default]
    Auto,
    Hd720,
}

impl Channel {
    pub const COLLECTION: &'static str = "channels";
}
