use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAutomationJobRequest {
    pub goal: String,
    pub target_audience: serde_json::Value,
    pub channel_preferences: Vec<String>,
    pub budget_limit: Option<f64>,
    pub currency: Option<String>,
    pub desired_outcome: Option<String>,
    pub approval_required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteAutomationJobRequest {
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewApprovalRequest {
    pub approved: bool,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePolicyRuleRequest {
    pub name: String,
    pub rule_type: String,
    pub scope: serde_json::Value,
    pub settings: serde_json::Value,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationJobResponse {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationRunResponse {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationActionResponse {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequestResponse {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRuleResponse {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResponse {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationKpiResponse {
    pub total_jobs: i64,
    pub active_jobs: i64,
    pub runs_in_progress: i64,
    pub pending_approvals: i64,
    pub blocked_actions: i64,
    pub enabled_policies: i64,
    pub experiments_running: i64,
    pub automation_coverage: f64,
    pub human_intervention_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationDashboardResponse {
    pub overview: AutomationKpiResponse,
    pub jobs: Vec<AutomationJobResponse>,
    pub runs: Vec<AutomationRunResponse>,
    pub actions: Vec<AutomationActionResponse>,
    pub approvals: Vec<ApprovalRequestResponse>,
    pub policies: Vec<PolicyRuleResponse>,
    pub experiments: Vec<ExperimentResponse>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationJobListResponse {
    pub data: Vec<AutomationJobResponse>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationRunListResponse {
    pub data: Vec<AutomationRunResponse>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationActionListResponse {
    pub data: Vec<AutomationActionResponse>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequestListResponse {
    pub data: Vec<ApprovalRequestResponse>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRuleListResponse {
    pub data: Vec<PolicyRuleResponse>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentListResponse {
    pub data: Vec<ExperimentResponse>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationBootstrapResponse {
    pub jobs_created: Vec<AutomationJobResponse>,
    pub policies_created: Vec<PolicyRuleResponse>,
    pub experiments_created: Vec<ExperimentResponse>,
    pub messages: Vec<String>,
}
