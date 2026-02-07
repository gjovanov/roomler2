use bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

use super::tenant::NotificationLevel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMember {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub tenant_id: ObjectId,
    pub channel_id: ObjectId,
    pub user_id: ObjectId,
    pub joined_at: DateTime,
    pub last_read_message_id: Option<ObjectId>,
    pub last_read_at: Option<DateTime>,
    #[serde(default)]
    pub unread_count: u32,
    #[serde(default)]
    pub mention_count: u32,
    pub notification_override: Option<NotificationLevel>,
    #[serde(default)]
    pub is_muted: bool,
    #[serde(default)]
    pub is_pinned: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

impl ChannelMember {
    pub const COLLECTION: &'static str = "channel_members";
}
