use axum::{
    extract::{Extension, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::state::{AppState, UserContext};
use crate::utils::{ApiError, ApiResponse};

use super::domain::AuditRepository;
use super::schema::{AuditLog, AuditLogListResponse};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub action: Option<String>,
    pub target_type: Option<String>,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
}

pub async fn list_audit_logs(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Query(query): Query<ListQuery>,
) -> Result<ApiResponse<AuditLogListResponse>, ApiError> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let repo = AuditRepository::new(app_state.registry.db_dao.clone());
    let action_filter = query.action.map(|value| format!("%{}%", value));
    let start_at = parse_time(query.start_at.as_deref())?;
    let end_at = parse_time(query.end_at.as_deref())?;
    let (data, total) = repo
        .list(
            ctx.tenant_id,
            page,
            limit,
            action_filter.as_deref(),
            query.target_type.as_deref(),
            start_at,
            end_at,
        )
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    Ok(ApiResponse::success(AuditLogListResponse { data, total }))
}

fn parse_time(value: Option<&str>) -> Result<Option<chrono::DateTime<chrono::Utc>>, ApiError> {
    let Some(value) = value else {
        return Ok(None);
    };

    chrono::DateTime::parse_from_rfc3339(value)
        .map(|ts| Some(ts.with_timezone(&chrono::Utc)))
        .map_err(|_| ApiError::bad_request("Invalid RFC3339 timestamp"))
}
