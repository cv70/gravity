use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CampaignType {
    Social,
    Email,
    Content,
    Ads,
}

impl std::fmt::Display for CampaignType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CampaignType::Social => write!(f, "social"),
            CampaignType::Email => write!(f, "email"),
            CampaignType::Content => write!(f, "content"),
            CampaignType::Ads => write!(f, "ads"),
        }
    }
}

impl From<String> for CampaignType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "social" => CampaignType::Social,
            "email" => CampaignType::Email,
            "content" => CampaignType::Content,
            "ads" => CampaignType::Ads,
            _ => CampaignType::Email,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CampaignStatus {
    Draft,
    Active,
    Paused,
    Completed,
}

impl std::fmt::Display for CampaignStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CampaignStatus::Draft => write!(f, "draft"),
            CampaignStatus::Active => write!(f, "active"),
            CampaignStatus::Paused => write!(f, "paused"),
            CampaignStatus::Completed => write!(f, "completed"),
        }
    }
}

impl From<String> for CampaignStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "draft" => CampaignStatus::Draft,
            "active" => CampaignStatus::Active,
            "paused" => CampaignStatus::Paused,
            "completed" => CampaignStatus::Completed,
            _ => CampaignStatus::Draft,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Campaign {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub campaign_type: String,
    pub status: String,
    pub description: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Campaign {
    pub fn new(tenant_id: Uuid, name: String, campaign_type: CampaignType) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            name,
            campaign_type: campaign_type.to_string(),
            status: CampaignStatus::Draft.to_string(),
            description: None,
            start_date: None,
            end_date: None,
            settings: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignMetrics {
    pub sent: i64,
    pub opened: i64,
    pub clicked: i64,
    pub converted: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub campaign_type: CampaignType,
    pub status: CampaignStatus,
    pub description: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub metrics: Option<CampaignMetrics>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCampaignRequest {
    pub name: String,
    pub campaign_type: CampaignType,
    pub description: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCampaignRequest {
    pub name: Option<String>,
    pub status: Option<CampaignStatus>,
    pub description: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignListResponse {
    pub data: Vec<CampaignResponse>,
    pub total: i64,
}
