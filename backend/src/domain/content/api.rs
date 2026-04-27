use axum::{extract::{Extension, Path, State}, Json};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::content::domain::ContentRepository;
use crate::domain::content::schema::{Content, ContentListResponse, CreateContentRequest};
use crate::utils::{ApiError, ApiResponse};
use crate::state::{AppState, UserContext};

pub async fn list_contents(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
) -> Result<ApiResponse<ContentListResponse>, ApiError> {
    let repo = ContentRepository::new(app_state.infra.db.clone());
    let contents = repo
        .list(ctx.tenant_id)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    let total = contents.len() as i64;
    Ok(ApiResponse::success(ContentListResponse { data: contents, total }))
}

pub async fn create_content(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Json(req): Json<CreateContentRequest>,
) -> Result<ApiResponse<Content>, ApiError> {
    let repo = ContentRepository::new(app_state.infra.db.clone());
    let content = repo
        .create(ctx.tenant_id, &req)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;

    let mut resp = ApiResponse::success(content);
    resp.code = 201;
    Ok(resp)
}

pub async fn get_content(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<Content>, ApiError> {
    let repo = ContentRepository::new(app_state.infra.db.clone());
    let content = repo
        .get_by_id(ctx.tenant_id, id)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    match content {
        Some(c) => Ok(ApiResponse::success(c)),
        None => Err(ApiError::not_found("Content not found")),
    }
}

pub async fn delete_content(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<()>, ApiError> {
    let repo = ContentRepository::new(app_state.infra.db.clone());
    let deleted = repo
        .delete(ctx.tenant_id, id)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    if deleted {
        let mut resp = ApiResponse::success(());
        resp.code = 204;
        Ok(resp)
    } else {
        Err(ApiError::not_found("Content not found"))
    }
}
