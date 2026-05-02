use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::state::{AppState, UserContext};
use crate::utils::{ApiError, ApiResponse};

use super::domain::ApprovalRepository;
use super::schema::{Approval, ApprovalListResponse, CreateApprovalRequest, ReviewApprovalRequest};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub status: Option<String>,
    pub object_type: Option<String>,
}

pub async fn list_approvals(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Query(query): Query<ListQuery>,
) -> Result<ApiResponse<ApprovalListResponse>, ApiError> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let repo = ApprovalRepository::new(app_state.registry.db_dao.clone());
    let (data, total) = repo
        .list(
            ctx.tenant_id,
            page,
            limit,
            query.status.as_deref(),
            query.object_type.as_deref(),
        )
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;
    Ok(ApiResponse::success(ApprovalListResponse { data, total }))
}

pub async fn create_approval(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Json(req): Json<CreateApprovalRequest>,
) -> Result<ApiResponse<Approval>, ApiError> {
    let repo = ApprovalRepository::new(app_state.registry.db_dao.clone());
    let approval = repo
        .create(ctx.tenant_id, ctx.user_id, &req)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;
    let mut resp = ApiResponse::success(approval);
    resp.code = 201;
    Ok(resp)
}

pub async fn update_approval(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<ReviewApprovalRequest>,
) -> Result<ApiResponse<Approval>, ApiError> {
    let repo = ApprovalRepository::new(app_state.registry.db_dao.clone());
    let approval = repo
        .review(ctx.tenant_id, id, ctx.user_id, &req)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;

    approval
        .map(ApiResponse::success)
        .ok_or_else(|| ApiError::not_found("Approval not found"))
}
