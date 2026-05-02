use axum::{
    extract::{Extension, Query, State},
    http::HeaderMap,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::analytics::domain::AnalyticsRepository;
use crate::domain::analytics::schema::{
    AnalyticsDashboard, ConversionRequest, FunnelStep, IdentifyRequest, PageRequest,
    TrackEventRequest,
};
use crate::state::AppState;
use crate::utils::{ApiError, ApiResponse};

pub async fn get_dashboard(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<crate::state::UserContext>,
) -> Result<ApiResponse<AnalyticsDashboard>, ApiError> {
    let repo = AnalyticsRepository::new(app_state.registry.db_dao.clone());
    let dashboard = repo
        .get_dashboard(ctx.tenant_id)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    Ok(ApiResponse::success(dashboard))
}

pub async fn track_event(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<TrackEventRequest>,
) -> Result<ApiResponse<()>, ApiError> {
    let repo = AnalyticsRepository::new(app_state.registry.db_dao.clone());
    let contact_id = req
        .contact_id
        .as_ref()
        .and_then(|s| Uuid::parse_str(s).ok());
    let properties = req.properties.unwrap_or(serde_json::json!({}));

    let tenant_id = resolve_tenant_id(&headers, req.tenant_id.as_deref())?;

    repo.record_event(tenant_id, contact_id, &req.event, properties)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    Ok(ApiResponse::success(()))
}

pub async fn identify(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdentifyRequest>,
) -> Result<ApiResponse<()>, ApiError> {
    let repo = AnalyticsRepository::new(app_state.registry.db_dao.clone());
    let contact_id = req
        .contact_id
        .as_ref()
        .and_then(|s| Uuid::parse_str(s).ok());
    let tenant_id = resolve_tenant_id(&headers, req.tenant_id.as_deref())?;

    repo.record_event(
        tenant_id,
        contact_id,
        "identify",
        serde_json::json!({ "traits": req.traits }),
    )
    .await
    .map_err(|e| ApiError::internal_error(e.to_string()))?;

    Ok(ApiResponse::success(()))
}

pub async fn track_page(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<PageRequest>,
) -> Result<ApiResponse<()>, ApiError> {
    let repo = AnalyticsRepository::new(app_state.registry.db_dao.clone());
    let contact_id = req
        .contact_id
        .as_ref()
        .and_then(|s| Uuid::parse_str(s).ok());
    let tenant_id = resolve_tenant_id(&headers, req.tenant_id.as_deref())?;

    repo.record_event(
        tenant_id,
        contact_id,
        "page.viewed",
        serde_json::json!({
            "name": req.name,
            "url": req.url,
            "referrer": req.referrer,
        }),
    )
    .await
    .map_err(|e| ApiError::internal_error(e.to_string()))?;

    Ok(ApiResponse::success(()))
}

pub async fn track_conversion(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ConversionRequest>,
) -> Result<ApiResponse<()>, ApiError> {
    let contact_id = Uuid::parse_str(&req.contact_id)
        .map_err(|_| ApiError::bad_request("Invalid contact_id"))?;
    let tenant_id = resolve_tenant_id(&headers, req.tenant_id.as_deref())?;

    let repo = AnalyticsRepository::new(app_state.registry.db_dao.clone());

    repo.record_conversion(
        Uuid::new_v4(),
        tenant_id,
        contact_id,
        req.goal_id.as_deref(),
        req.value,
        req.currency.as_deref().unwrap_or("CNY"),
        req.properties.unwrap_or(serde_json::json!({})),
    )
    .await
    .map_err(|e| ApiError::internal_error(e.to_string()))?;

    repo.record_event(
        tenant_id,
        Some(contact_id),
        "conversion.recorded",
        serde_json::json!({
            "goal_id": req.goal_id,
            "value": req.value,
        }),
    )
    .await
    .map_err(|e| ApiError::internal_error(e.to_string()))?;

    Ok(ApiResponse::success(()))
}

fn resolve_tenant_id(headers: &HeaderMap, body_tenant: Option<&str>) -> Result<Uuid, ApiError> {
    if let Some(tenant_id) = body_tenant {
        return Uuid::parse_str(tenant_id)
            .map_err(|_| ApiError::bad_request("Invalid tenant_id"));
    }

    if let Some(header_value) = headers.get("x-tenant-id").and_then(|value| value.to_str().ok()) {
        return Uuid::parse_str(header_value)
            .map_err(|_| ApiError::bad_request("Invalid X-Tenant-Id header"));
    }

    Ok(Uuid::nil())
}

pub async fn get_funnel(
    State(_app_state): State<Arc<AppState>>,
    Extension(_ctx): Extension<crate::state::UserContext>,
    Query(_params): Query<FunnelQuery>,
) -> Result<ApiResponse<serde_json::Value>, ApiError> {
    let funnel = vec![
        FunnelStep {
            step: "发送".to_string(),
            count: 10000,
            dropoff_rate: 0.0,
        },
        FunnelStep {
            step: "打开".to_string(),
            count: 3500,
            dropoff_rate: 0.65,
        },
        FunnelStep {
            step: "点击".to_string(),
            count: 800,
            dropoff_rate: 0.77,
        },
        FunnelStep {
            step: "转化".to_string(),
            count: 120,
            dropoff_rate: 0.85,
        },
    ];

    Ok(ApiResponse::success(serde_json::json!({ "steps": funnel })))
}

#[derive(serde::Deserialize)]
pub struct FunnelQuery {
    pub campaign_id: Option<String>,
}
