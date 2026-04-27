use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Contact {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub name: String,
    pub phone: Option<String>,
    pub tags: Vec<String>,
    pub attributes: serde_json::Value,
    pub subscribed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Contact {
    pub fn new(tenant_id: Uuid, email: String, name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            email,
            name,
            phone: None,
            tags: vec![],
            attributes: serde_json::json!({}),
            subscribed: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContactRequest {
    pub email: String,
    pub name: String,
    pub phone: Option<String>,
    pub tags: Option<Vec<String>>,
    pub attributes: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateContactRequest {
    pub email: Option<String>,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub tags: Option<Vec<String>>,
    pub attributes: Option<serde_json::Value>,
    pub subscribed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactListResponse {
    pub data: Vec<Contact>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
}
