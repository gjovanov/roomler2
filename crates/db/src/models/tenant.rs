use bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub owner_id: ObjectId,
    pub plan: Plan,
    pub features: Vec<String>,
    pub settings: TenantSettings,
    pub billing: Option<BillingInfo>,
    pub integrations: Option<IntegrationSettings>,
    #[serde(default)]
    pub is_archived: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub deleted_at: Option<DateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Plan {
    #[default]
    Free,
    Pro,
    Business,
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantSettings {
    #[serde(default = "default_locale")]
    pub default_locale: String,
    #[serde(default)]
    pub default_message_notifications: NotificationLevel,
    #[serde(default)]
    pub mfa_required: bool,
    #[serde(default)]
    pub allow_guest_access: bool,
    #[serde(default = "default_max_members")]
    pub max_members: u32,
    #[serde(default = "default_file_upload_limit")]
    pub file_upload_limit: u64,
}

impl Default for TenantSettings {
    fn default() -> Self {
        Self {
            default_locale: default_locale(),
            default_message_notifications: NotificationLevel::default(),
            mfa_required: false,
            allow_guest_access: false,
            max_members: default_max_members(),
            file_upload_limit: default_file_upload_limit(),
        }
    }
}

fn default_locale() -> String {
    "en-US".to_string()
}

fn default_max_members() -> u32 {
    100
}

fn default_file_upload_limit() -> u64 {
    10 * 1024 * 1024 // 10 MB
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NotificationLevel {
    #[default]
    All,
    Mentions,
    Nothing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingInfo {
    pub customer_id: Option<String>,
    pub subscription_id: Option<String>,
    pub current_period_end: Option<DateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationSettings {
    pub google_drive: Option<OAuthCredential>,
    pub onedrive: Option<OAuthCredential>,
    pub dropbox: Option<OAuthCredential>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthCredential {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime>,
}

impl Tenant {
    pub const COLLECTION: &'static str = "tenants";
}
