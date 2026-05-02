use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub definition: serde_json::Value,
    pub is_dynamic: bool,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSegmentRequest {
    pub name: String,
    pub definition: serde_json::Value,
    pub is_dynamic: Option<bool>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSegmentRequest {
    pub name: Option<String>,
    pub definition: Option<serde_json::Value>,
    pub is_dynamic: Option<bool>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentPreviewRequest {
    pub definition: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentListResponse {
    pub data: Vec<Segment>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentPreviewContact {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub lifecycle_stage: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentPreviewResponse {
    pub matching_count: i64,
    pub sample_contacts: Vec<SegmentPreviewContact>,
}
