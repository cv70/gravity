use axum::{extract::{Extension, Path, State}, Json};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::campaign::domain::CampaignRepository;
use crate::domain::campaign::schema::{
    Campaign, CampaignListResponse, CampaignResponse, CampaignType, CreateCampaignRequest, UpdateCampaignRequest,
};
use crate::utils::{ApiError, ApiResponse};
use crate::state::{AppState, UserContext};

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
) -> Result<ApiResponse<CampaignListResponse>, ApiError> {
    let repo = CampaignRepository::new(app_state.infra.db.clone());
    let campaigns = repo
        .list(ctx.tenant_id)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    let data: Vec<CampaignResponse> = campaigns.into_iter().map(to_response).collect();
    let total = data.len() as i64;

    Ok(ApiResponse::success(CampaignListResponse { data, total }))
}

pub async fn create_campaign(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Json(req): Json<CreateCampaignRequest>,
) -> Result<ApiResponse<CampaignResponse>, ApiError> {
    let repo = CampaignRepository::new(app_state.infra.db.clone());
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
    let repo = CampaignRepository::new(app_state.infra.db.clone());
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
    let repo = CampaignRepository::new(app_state.infra.db.clone());
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
    let repo = CampaignRepository::new(app_state.infra.db.clone());
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
    let repo = CampaignRepository::new(app_state.infra.db.clone());
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
    let repo = CampaignRepository::new(app_state.infra.db.clone());
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
