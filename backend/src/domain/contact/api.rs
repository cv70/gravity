use axum::{extract::{Extension, Path, Query, State}, Json};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::contact::domain::ContactRepository;
use crate::domain::contact::schema::{
    Contact, ContactListResponse, CreateContactRequest, UpdateContactRequest,
};
use crate::utils::{ApiError, ApiResponse};
use crate::state::{AppState, UserContext};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub search: Option<String>,
}

pub async fn list_contacts(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Query(query): Query<ListQuery>,
) -> Result<ApiResponse<ContactListResponse>, ApiError> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);

    let repo = ContactRepository::new(app_state.registry.db_dao.clone());
    let result = repo
        .list(ctx.tenant_id, page, limit, query.search.as_deref())
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    Ok(ApiResponse::success(result))
}

pub async fn create_contact(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Json(req): Json<CreateContactRequest>,
) -> Result<ApiResponse<Contact>, ApiError> {
    let repo = ContactRepository::new(app_state.registry.db_dao.clone());
    let contact = repo
        .create(ctx.tenant_id, &req)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;

    let mut resp = ApiResponse::success(contact);
    resp.code = 201;
    Ok(resp)
}

pub async fn get_contact(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<Contact>, ApiError> {
    let repo = ContactRepository::new(app_state.registry.db_dao.clone());
    let contact = repo
        .get_by_id(ctx.tenant_id, id)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    match contact {
        Some(c) => Ok(ApiResponse::success(c)),
        None => Err(ApiError::not_found("Contact not found")),
    }
}

pub async fn update_contact(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateContactRequest>,
) -> Result<ApiResponse<Contact>, ApiError> {
    let repo = ContactRepository::new(app_state.registry.db_dao.clone());
    let contact = repo
        .update(ctx.tenant_id, id, &req)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    match contact {
        Some(c) => Ok(ApiResponse::success(c)),
        None => Err(ApiError::not_found("Contact not found")),
    }
}

pub async fn delete_contact(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<()>, ApiError> {
    let repo = ContactRepository::new(app_state.registry.db_dao.clone());
    let deleted = repo
        .delete(ctx.tenant_id, id)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    if deleted {
        let mut resp = ApiResponse::success(());
        resp.code = 204;
        Ok(resp)
    } else {
        Err(ApiError::not_found("Contact not found"))
    }
}
