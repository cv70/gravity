use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackEventRequest {
    pub event: String,
    pub contact_id: Option<String>,
    pub properties: Option<serde_json::Value>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifyRequest {
    pub contact_id: Option<String>,
    pub email: Option<String>,
    pub traits: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRequest {
    pub contact_id: Option<String>,
    pub name: String,
    pub url: Option<String>,
    pub referrer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionRequest {
    pub contact_id: String,
    pub goal_id: Option<String>,
    pub value: Option<f64>,
    pub currency: Option<String>,
    pub properties: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsDashboard {
    pub total_contacts: i64,
    pub active_campaigns: i64,
    pub total_conversions: i64,
    pub conversion_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelStep {
    pub step: String,
    pub count: i64,
    pub dropoff_rate: f64,
}
