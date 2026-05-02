use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::state::{AppState, UserContext};
use crate::utils::{ApiError, ApiResponse};

use super::domain::SegmentRepository;
use super::schema::{
    CreateSegmentRequest, Segment, SegmentListResponse, SegmentPreviewRequest,
    SegmentPreviewResponse, UpdateSegmentRequest,
};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

pub async fn list_segments(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Query(query): Query<ListQuery>,
) -> Result<ApiResponse<SegmentListResponse>, ApiError> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);

    let repo = SegmentRepository::new(app_state.registry.db_dao.clone());
    let (data, total) = repo
        .list(ctx.tenant_id, page, limit)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    Ok(ApiResponse::success(SegmentListResponse { data, total }))
}

pub async fn create_segment(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Json(req): Json<CreateSegmentRequest>,
) -> Result<ApiResponse<Segment>, ApiError> {
    let repo = SegmentRepository::new(app_state.registry.db_dao.clone());
    let segment = repo
        .create(ctx.tenant_id, &req)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;

    let mut resp = ApiResponse::success(segment);
    resp.code = 201;
    Ok(resp)
}

pub async fn get_segment(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<Segment>, ApiError> {
    let repo = SegmentRepository::new(app_state.registry.db_dao.clone());
    let segment = repo
        .get_by_id(ctx.tenant_id, id)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    segment
        .map(ApiResponse::success)
        .ok_or_else(|| ApiError::not_found("Segment not found"))
}

pub async fn update_segment(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateSegmentRequest>,
) -> Result<ApiResponse<Segment>, ApiError> {
    let repo = SegmentRepository::new(app_state.registry.db_dao.clone());
    let segment = repo
        .update(ctx.tenant_id, id, &req)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    segment
        .map(ApiResponse::success)
        .ok_or_else(|| ApiError::not_found("Segment not found"))
}

pub async fn delete_segment(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<()>, ApiError> {
    let repo = SegmentRepository::new(app_state.registry.db_dao.clone());
    let deleted = repo
        .delete(ctx.tenant_id, id)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    if deleted {
        let mut resp = ApiResponse::success(());
        resp.code = 204;
        Ok(resp)
    } else {
        Err(ApiError::not_found("Segment not found"))
    }
}

pub async fn preview_segment(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<SegmentPreviewRequest>,
) -> Result<ApiResponse<SegmentPreviewResponse>, ApiError> {
    let repo = SegmentRepository::new(app_state.registry.db_dao.clone());
    let definition = if let Some(definition) = req.definition {
        definition
    } else {
        repo
            .get_by_id(ctx.tenant_id, id)
            .await
            .map_err(|e| ApiError::internal_error(e.to_string()))?
            .ok_or_else(|| ApiError::not_found("Segment not found"))?
            .definition
    };

    let preview = repo
        .preview(ctx.tenant_id, &definition)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    Ok(ApiResponse::success(preview))
}
