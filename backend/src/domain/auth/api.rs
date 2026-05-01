use axum::{
    extract::{Extension, State},
    Json,
};
use std::sync::Arc;

use crate::domain::auth::schema::{
    AuthResponse, LoginRequest, MeResponse, RefreshRequest, RegisterRequest,
};
use crate::state::{AppState, UserContext};
use crate::utils::{ApiError, ApiResponse};

pub async fn register(
    State(app_state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<ApiResponse<AuthResponse>, ApiError> {
    let response = app_state
        .auth_service
        .register(&req)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;

    let mut resp = ApiResponse::success(response);
    resp.code = 201;
    Ok(resp)
}

pub async fn login(
    State(app_state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<ApiResponse<AuthResponse>, ApiError> {
    let response = app_state
        .auth_service
        .login(&req.email, &req.password, &req.organization_name)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    match response {
        Some(auth) => Ok(ApiResponse::success(auth)),
        None => Err(ApiError::unauthorized("Invalid credentials")),
    }
}

pub async fn refresh(
    State(app_state): State<Arc<AppState>>,
    Json(req): Json<RefreshRequest>,
) -> Result<ApiResponse<serde_json::Value>, ApiError> {
    let claims = app_state
        .auth_service
        .verify_refresh_token(&req.refresh_token)
        .map_err(|e| ApiError::internal_error(e.to_string()))?
        .ok_or_else(|| ApiError::unauthorized("Invalid or expired refresh token"))?;

    let access_token = app_state
        .auth_service
        .generate_access_from_refresh(&claims)
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    let response = serde_json::json!({
        "access_token": access_token,
    });
    Ok(ApiResponse::success(response))
}

pub async fn me(
    State(app_state): State<Arc<AppState>>,
    Extension(ctx): Extension<UserContext>,
) -> Result<ApiResponse<MeResponse>, ApiError> {
    let me = app_state
        .auth_service
        .get_me(ctx.user_id, ctx.tenant_id)
        .await
        .map_err(|e| ApiError::internal_error(e.to_string()))?
        .ok_or_else(|| ApiError::not_found("User profile not found"))?;

    Ok(ApiResponse::success(me))
}
