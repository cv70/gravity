use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::state::{AppState, UserContext};
use crate::utils::{ApiError, ApiResponse};

use super::domain::AutomationRepository;
use super::schema::{
    ApprovalRequestListResponse, AutomationActionListResponse, AutomationBootstrapResponse,
    AutomationDashboardResponse, AutomationJobListResponse, AutomationJobResponse,
    AutomationRunListResponse, CreateAutomationJobRequest, CreatePolicyRuleRequest,
    ExperimentListResponse, PolicyRuleListResponse, ReviewApprovalRequest,
};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, serde::Serialize)]
pub struct ExecuteAutomationJobResponse {
    pub run: super::schema::AutomationRunResponse,
    pub approval: Option<super::schema::ApprovalRequestResponse>,
    pub action: Option<super::schema::AutomationActionResponse>,
}

fn pagination(query: &ListQuery) -> (i64, i64) {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(10).clamp(1, 100);
    (page, limit)
}

pub async fn get_dashboard(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Query(query): Query<ListQuery>,
) -> Result<ApiResponse<AutomationDashboardResponse>, ApiError> {
    let limit = query.limit.unwrap_or(5).clamp(1, 20);
    let repo = AutomationRepository::new(app_state.registry.db_dao.clone());
    let dashboard = repo
        .dashboard(ctx.tenant_id, limit)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    Ok(ApiResponse::success(dashboard))
}

pub async fn list_jobs(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Query(query): Query<ListQuery>,
) -> Result<ApiResponse<AutomationJobListResponse>, ApiError> {
    let (page, limit) = pagination(&query);
    let repo = AutomationRepository::new(app_state.registry.db_dao.clone());
    let (data, total) = repo
        .list_jobs(ctx.tenant_id, page, limit)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    Ok(ApiResponse::success(AutomationJobListResponse {
        data,
        total,
    }))
}

pub async fn create_job(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Json(req): Json<CreateAutomationJobRequest>,
) -> Result<ApiResponse<AutomationJobResponse>, ApiError> {
    let repo = AutomationRepository::new(app_state.registry.db_dao.clone());
    let job = repo
        .create_job(ctx.tenant_id, &req)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;

    let mut resp = ApiResponse::success(job);
    resp.code = 201;
    Ok(resp)
}

pub async fn execute_job(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<super::schema::ExecuteAutomationJobRequest>,
) -> Result<ApiResponse<ExecuteAutomationJobResponse>, ApiError> {
    let repo = AutomationRepository::new(app_state.registry.db_dao.clone());
    let (run, approval, action) = repo
        .execute_job(ctx.tenant_id, id, req.note)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;

    Ok(ApiResponse::success(ExecuteAutomationJobResponse {
        run,
        approval,
        action,
    }))
}

pub async fn list_runs(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Query(query): Query<ListQuery>,
) -> Result<ApiResponse<AutomationRunListResponse>, ApiError> {
    let (page, limit) = pagination(&query);
    let repo = AutomationRepository::new(app_state.registry.db_dao.clone());
    let (data, total) = repo
        .list_runs(ctx.tenant_id, page, limit)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    Ok(ApiResponse::success(AutomationRunListResponse {
        data,
        total,
    }))
}

pub async fn list_actions(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Query(query): Query<ListQuery>,
) -> Result<ApiResponse<AutomationActionListResponse>, ApiError> {
    let (page, limit) = pagination(&query);
    let repo = AutomationRepository::new(app_state.registry.db_dao.clone());
    let (data, total) = repo
        .list_actions(ctx.tenant_id, page, limit)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    Ok(ApiResponse::success(AutomationActionListResponse {
        data,
        total,
    }))
}

pub async fn list_approvals(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Query(query): Query<ListQuery>,
) -> Result<ApiResponse<ApprovalRequestListResponse>, ApiError> {
    let (page, limit) = pagination(&query);
    let repo = AutomationRepository::new(app_state.registry.db_dao.clone());
    let (data, total) = repo
        .list_approvals(ctx.tenant_id, page, limit)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    Ok(ApiResponse::success(ApprovalRequestListResponse {
        data,
        total,
    }))
}

pub async fn review_approval(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<ReviewApprovalRequest>,
) -> Result<ApiResponse<serde_json::Value>, ApiError> {
    let repo = AutomationRepository::new(app_state.registry.db_dao.clone());
    let (approval, action) = repo
        .review_approval(ctx.tenant_id, id, ctx.user_id, &req)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;

    Ok(ApiResponse::success(serde_json::json!({
        "approval": approval,
        "action": action,
    })))
}

pub async fn list_policies(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Query(query): Query<ListQuery>,
) -> Result<ApiResponse<PolicyRuleListResponse>, ApiError> {
    let (page, limit) = pagination(&query);
    let repo = AutomationRepository::new(app_state.registry.db_dao.clone());
    let (data, total) = repo
        .list_policies(ctx.tenant_id, page, limit)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    Ok(ApiResponse::success(PolicyRuleListResponse { data, total }))
}

pub async fn create_policy(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Json(req): Json<CreatePolicyRuleRequest>,
) -> Result<ApiResponse<super::schema::PolicyRuleResponse>, ApiError> {
    let repo = AutomationRepository::new(app_state.registry.db_dao.clone());
    let policy = repo
        .create_policy_rule(ctx.tenant_id, &req)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;

    let mut resp = ApiResponse::success(policy);
    resp.code = 201;
    Ok(resp)
}

pub async fn bootstrap_defaults(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
) -> Result<ApiResponse<AutomationBootstrapResponse>, ApiError> {
    let repo = AutomationRepository::new(app_state.registry.db_dao.clone());
    let data = repo
        .bootstrap_defaults(ctx.tenant_id)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    Ok(ApiResponse::success(data))
}

pub async fn list_experiments(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Query(query): Query<ListQuery>,
) -> Result<ApiResponse<ExperimentListResponse>, ApiError> {
    let (page, limit) = pagination(&query);
    let repo = AutomationRepository::new(app_state.registry.db_dao.clone());
    let (data, total) = repo
        .list_experiments(ctx.tenant_id, page, limit)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    Ok(ApiResponse::success(ExperimentListResponse { data, total }))
}
