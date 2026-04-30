use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Content {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub campaign_id: Option<Uuid>,
    pub name: String,
    pub content_type: String,
    pub content: serde_json::Value,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Content {
    pub fn new(tenant_id: Uuid, name: String, content_type: String, content: serde_json::Value, campaign_id: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            campaign_id,
            name,
            content_type,
            content,
            status: "draft".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContentRequest {
    pub name: String,
    pub content_type: String,
    pub content: serde_json::Value,
    pub campaign_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentListResponse {
    pub data: Vec<Content>,
    pub total: i64,
}
