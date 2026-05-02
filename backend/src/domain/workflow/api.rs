use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::state::{AppState, UserContext};
use crate::utils::{ApiError, ApiResponse};

use super::domain::WorkflowRepository;
use super::schema::{
    CreateWorkflowRequest, TestWorkflowRequest, UpdateWorkflowRequest, Workflow,
    WorkflowExecution, WorkflowExecutionListResponse, WorkflowListResponse, WorkflowTestResponse,
};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

pub async fn list_workflows(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Query(query): Query<ListQuery>,
) -> Result<ApiResponse<WorkflowListResponse>, ApiError> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let repo = WorkflowRepository::new(app_state.registry.db_dao.clone());
    let (data, total) = repo
        .list(ctx.tenant_id, page, limit)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;
    Ok(ApiResponse::success(WorkflowListResponse { data, total }))
}

pub async fn create_workflow(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Json(req): Json<CreateWorkflowRequest>,
) -> Result<ApiResponse<Workflow>, ApiError> {
    let repo = WorkflowRepository::new(app_state.registry.db_dao.clone());
    let workflow = repo
        .create(ctx.tenant_id, &req)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;

    let mut resp = ApiResponse::success(workflow);
    resp.code = 201;
    Ok(resp)
}

pub async fn get_workflow(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<Workflow>, ApiError> {
    let repo = WorkflowRepository::new(app_state.registry.db_dao.clone());
    repo.get_by_id(ctx.tenant_id, id)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?
        .map(ApiResponse::success)
        .ok_or_else(|| ApiError::not_found("Workflow not found"))
}

pub async fn update_workflow(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateWorkflowRequest>,
) -> Result<ApiResponse<Workflow>, ApiError> {
    let repo = WorkflowRepository::new(app_state.registry.db_dao.clone());
    repo.update(ctx.tenant_id, id, &req)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?
        .map(ApiResponse::success)
        .ok_or_else(|| ApiError::not_found("Workflow not found"))
}

pub async fn activate_workflow(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<Workflow>, ApiError> {
    let repo = WorkflowRepository::new(app_state.registry.db_dao.clone());
    repo.activate(ctx.tenant_id, id)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?
        .map(ApiResponse::success)
        .ok_or_else(|| ApiError::not_found("Workflow not found"))
}

pub async fn deactivate_workflow(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<Workflow>, ApiError> {
    let repo = WorkflowRepository::new(app_state.registry.db_dao.clone());
    repo.deactivate(ctx.tenant_id, id)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?
        .map(ApiResponse::success)
        .ok_or_else(|| ApiError::not_found("Workflow not found"))
}

pub async fn delete_workflow(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<()>, ApiError> {
    let repo = WorkflowRepository::new(app_state.registry.db_dao.clone());
    let deleted = repo
        .delete(ctx.tenant_id, id)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    if deleted {
        let mut resp = ApiResponse::success(());
        resp.code = 204;
        Ok(resp)
    } else {
        Err(ApiError::not_found("Workflow not found"))
    }
}

pub async fn list_executions(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> Result<ApiResponse<WorkflowExecutionListResponse>, ApiError> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let repo = WorkflowRepository::new(app_state.registry.db_dao.clone());
    let (data, total) = repo
        .list_executions(ctx.tenant_id, id, page, limit)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    Ok(ApiResponse::success(WorkflowExecutionListResponse { data, total }))
}

pub async fn get_execution(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path((_workflow_id, execution_id)): Path<(Uuid, Uuid)>,
) -> Result<ApiResponse<WorkflowExecution>, ApiError> {
    let repo = WorkflowRepository::new(app_state.registry.db_dao.clone());
    repo.get_execution(ctx.tenant_id, execution_id)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?
        .map(ApiResponse::success)
        .ok_or_else(|| ApiError::not_found("Workflow execution not found"))
}

pub async fn test_workflow(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<TestWorkflowRequest>,
) -> Result<ApiResponse<WorkflowTestResponse>, ApiError> {
    let repo = WorkflowRepository::new(app_state.registry.db_dao.clone());
    let result = repo
        .test(ctx.tenant_id, id, &req)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;
    Ok(ApiResponse::success(result))
}
