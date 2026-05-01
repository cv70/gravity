use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::state::{AppState, UserContext};
use crate::utils::{ApiError, ApiResponse};

use super::domain::ChannelRepository;
use super::schema::{
    ChannelAccount, ChannelAccountListResponse, CreateChannelAccountRequest,
    UpdateChannelAccountRequest,
};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

fn pagination(query: &ListQuery) -> (i64, i64) {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    (page, limit)
}

pub async fn list_channels(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Query(query): Query<ListQuery>,
) -> Result<ApiResponse<ChannelAccountListResponse>, ApiError> {
    let (page, limit) = pagination(&query);
    let repo = ChannelRepository::new(app_state.registry.db_dao.clone());
    let data = repo
        .list(ctx.tenant_id, page, limit)
        .await
        .map_err(|err| ApiError::internal_error(err.to_string()))?;

    Ok(ApiResponse::success(data))
}

pub async fn create_channel(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Json(req): Json<CreateChannelAccountRequest>,
) -> Result<ApiResponse<ChannelAccount>, ApiError> {
    let repo = ChannelRepository::new(app_state.registry.db_dao.clone());
    let channel = repo
        .create(ctx.tenant_id, &req)
        .await
        .map_err(|err| ApiError::bad_request(err.to_string()))?;

    let mut resp = ApiResponse::success(channel);
    resp.code = 201;
    Ok(resp)
}

pub async fn update_channel(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateChannelAccountRequest>,
) -> Result<ApiResponse<ChannelAccount>, ApiError> {
    let repo = ChannelRepository::new(app_state.registry.db_dao.clone());
    let channel = repo
        .update(ctx.tenant_id, id, &req)
        .await
        .map_err(|err| ApiError::internal_error(err.to_string()))?;

    match channel {
        Some(channel) => Ok(ApiResponse::success(channel)),
        None => Err(ApiError::not_found("Channel account not found")),
    }
}

pub async fn delete_channel(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<()>, ApiError> {
    let repo = ChannelRepository::new(app_state.registry.db_dao.clone());
    let deleted = repo
        .delete(ctx.tenant_id, id)
        .await
        .map_err(|err| ApiError::internal_error(err.to_string()))?;

    if deleted {
        let mut resp = ApiResponse::success(());
        resp.code = 204;
        Ok(resp)
    } else {
        Err(ApiError::not_found("Channel account not found"))
    }
}
