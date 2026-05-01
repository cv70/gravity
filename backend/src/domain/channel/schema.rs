use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelAccount {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub platform: String,
    pub name: String,
    pub credentials_encrypted: String,
    pub settings: serde_json::Value,
    pub status: String,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChannelAccountRequest {
    pub platform: String,
    pub name: String,
    pub credentials_encrypted: String,
    pub settings: serde_json::Value,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateChannelAccountRequest {
    pub name: Option<String>,
    pub status: Option<String>,
    pub settings: Option<serde_json::Value>,
    pub last_sync_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelAccountListResponse {
    pub data: Vec<ChannelAccount>,
    pub total: i64,
}
