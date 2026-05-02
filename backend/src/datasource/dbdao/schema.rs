use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct OrganizationRow {
    pub id: Uuid,
    pub name: String,
    pub plan: String,
    pub settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub role: String,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CampaignRow {
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

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ContactRow {
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

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ContentRow {
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

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AutomationJobRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub goal: String,
    pub target_audience: serde_json::Value,
    pub channel_preferences: Vec<String>,
    pub strategy: serde_json::Value,
    pub status: String,
    pub risk_level: String,
    pub approval_required: bool,
    pub budget_limit: Option<f64>,
    pub currency: String,
    pub next_action_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AutomationRunRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub job_id: Uuid,
    pub status: String,
    pub current_step: String,
    pub input_context: serde_json::Value,
    pub output_context: serde_json::Value,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AutomationActionRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub run_id: Uuid,
    pub action_type: String,
    pub channel: String,
    pub payload: serde_json::Value,
    pub risk_level: String,
    pub status: String,
    pub requires_approval: bool,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub executed_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ApprovalRequestRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub action_id: Uuid,
    pub title: String,
    pub reason: String,
    pub status: String,
    pub requested_by: Option<Uuid>,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub decision_note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PolicyRuleRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub rule_type: String,
    pub scope: serde_json::Value,
    pub settings: serde_json::Value,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ExperimentRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub job_id: Option<Uuid>,
    pub name: String,
    pub hypothesis: String,
    pub variant_a: serde_json::Value,
    pub variant_b: serde_json::Value,
    pub status: String,
    pub winner: Option<String>,
    pub metric: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ChannelAccountRow {
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

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SegmentRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub definition: serde_json::Value,
    pub is_dynamic: bool,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WorkflowRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub version: i32,
    pub trigger_type: String,
    pub trigger_config: serde_json::Value,
    pub steps: serde_json::Value,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WorkflowExecutionRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub workflow_id: Uuid,
    pub contact_id: Uuid,
    pub status: String,
    pub current_step_index: i32,
    pub context: serde_json::Value,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WorkflowScheduleRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub workflow_execution_id: Uuid,
    pub resume_at: DateTime<Utc>,
    pub payload: serde_json::Value,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ApprovalRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub object_type: String,
    pub object_id: Uuid,
    pub status: String,
    pub requested_by: Uuid,
    pub approved_by: Option<Uuid>,
    pub reason: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AuditLogRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub target_type: Option<String>,
    pub target_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}
