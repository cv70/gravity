use axum::{
    extract::{Extension, State},
    Json,
};
use std::sync::Arc;

use crate::state::{AppState, UserContext};
use crate::utils::{ApiError, ApiResponse};

use super::domain::InsightsRepository;
use super::schema::{InsightItem, InsightListResponse};

pub async fn get_recommendations(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
) -> Result<ApiResponse<InsightListResponse>, ApiError> {
    let repo = InsightsRepository::new(app_state.registry.db_dao.clone());
    let data = repo
        .recommendations(ctx.tenant_id)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;
    Ok(ApiResponse::success(InsightListResponse { data }))
}

pub async fn get_anomalies(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
) -> Result<ApiResponse<InsightListResponse>, ApiError> {
    let repo = InsightsRepository::new(app_state.registry.db_dao.clone());
    let data = repo
        .anomalies(ctx.tenant_id)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;
    Ok(ApiResponse::success(InsightListResponse { data }))
}

pub async fn get_opportunities(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
) -> Result<ApiResponse<InsightListResponse>, ApiError> {
    let repo = InsightsRepository::new(app_state.registry.db_dao.clone());
    let data = repo
        .opportunities(ctx.tenant_id)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;
    Ok(ApiResponse::success(InsightListResponse { data }))
}
