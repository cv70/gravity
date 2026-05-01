use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::campaign::domain::CampaignRepository;
use crate::domain::campaign::schema::{
    Campaign, CampaignListResponse, CampaignResponse, CampaignType, CreateCampaignRequest,
    UpdateCampaignRequest,
};
use crate::state::{AppState, UserContext};
use crate::utils::{ApiError, ApiResponse};

#[derive(Debug, serde::Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

fn to_response(c: Campaign) -> CampaignResponse {
    CampaignResponse {
        id: c.id,
        tenant_id: c.tenant_id,
        name: c.name,
        campaign_type: CampaignType::from(c.campaign_type),
        status: crate::domain::campaign::schema::CampaignStatus::from(c.status),
        description: c.description,
        start_date: c.start_date,
        end_date: c.end_date,
        metrics: None,
        created_at: c.created_at,
        updated_at: c.updated_at,
    }
}

pub async fn list_campaigns(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Query(query): Query<ListQuery>,
) -> Result<ApiResponse<CampaignListResponse>, ApiError> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);

    let repo = CampaignRepository::new(app_state.registry.db_dao.clone());
    let (campaigns, total) = repo
        .list(ctx.tenant_id, page, limit)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    let data: Vec<CampaignResponse> = campaigns.into_iter().map(to_response).collect();

    Ok(ApiResponse::success(CampaignListResponse { data, total }))
}

pub async fn create_campaign(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Json(req): Json<CreateCampaignRequest>,
) -> Result<ApiResponse<CampaignResponse>, ApiError> {
    let repo = CampaignRepository::new(app_state.registry.db_dao.clone());
    let campaign = repo
        .create(ctx.tenant_id, &req)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;

    let mut resp = ApiResponse::success(to_response(campaign));
    resp.code = 201;
    Ok(resp)
}

pub async fn get_campaign(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<CampaignResponse>, ApiError> {
    let repo = CampaignRepository::new(app_state.registry.db_dao.clone());
    let campaign = repo
        .get_by_id(ctx.tenant_id, id)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    match campaign {
        Some(c) => Ok(ApiResponse::success(to_response(c))),
        None => Err(ApiError::not_found("Campaign not found")),
    }
}

pub async fn update_campaign(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCampaignRequest>,
) -> Result<ApiResponse<CampaignResponse>, ApiError> {
    let repo = CampaignRepository::new(app_state.registry.db_dao.clone());
    let campaign = repo
        .update(ctx.tenant_id, id, &req)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    match campaign {
        Some(c) => Ok(ApiResponse::success(to_response(c))),
        None => Err(ApiError::not_found("Campaign not found")),
    }
}

pub async fn launch_campaign(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<CampaignResponse>, ApiError> {
    let repo = CampaignRepository::new(app_state.registry.db_dao.clone());

    let existing = repo
        .get_by_id(ctx.tenant_id, id)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    match existing {
        Some(c) if c.status == "active" => {
            return Err(ApiError::bad_request("Campaign is already active"));
        }
        Some(c) if c.status == "completed" => {
            return Err(ApiError::bad_request("Cannot launch a completed campaign"));
        }
        None => return Err(ApiError::not_found("Campaign not found")),
        _ => {}
    }

    let campaign = repo
        .update(
            ctx.tenant_id,
            id,
            &UpdateCampaignRequest {
                name: None,
                status: Some(crate::domain::campaign::schema::CampaignStatus::Active),
                description: None,
                start_date: None,
                end_date: None,
            },
        )
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    match campaign {
        Some(c) => Ok(ApiResponse::success(to_response(c))),
        None => Err(ApiError::not_found("Campaign not found")),
    }
}

pub async fn pause_campaign(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<CampaignResponse>, ApiError> {
    let repo = CampaignRepository::new(app_state.registry.db_dao.clone());

    let existing = repo
        .get_by_id(ctx.tenant_id, id)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    match existing {
        Some(c) if c.status != "active" => {
            return Err(ApiError::bad_request("Only active campaigns can be paused"));
        }
        None => return Err(ApiError::not_found("Campaign not found")),
        _ => {}
    }

    let campaign = repo
        .update(
            ctx.tenant_id,
            id,
            &UpdateCampaignRequest {
                name: None,
                status: Some(crate::domain::campaign::schema::CampaignStatus::Paused),
                description: None,
                start_date: None,
                end_date: None,
            },
        )
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    match campaign {
        Some(c) => Ok(ApiResponse::success(to_response(c))),
        None => Err(ApiError::not_found("Campaign not found")),
    }
}

pub async fn delete_campaign(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<()>, ApiError> {
    let repo = CampaignRepository::new(app_state.registry.db_dao.clone());
    let deleted = repo
        .delete(ctx.tenant_id, id)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    if deleted {
        let mut resp = ApiResponse::success(());
        resp.code = 204;
        Ok(resp)
    } else {
        Err(ApiError::not_found("Campaign not found"))
    }
}
