use axum::{extract::State, Json};
use std::sync::Arc;

use crate::domain::auth::domain::AuthService;
use crate::domain::auth::schema::{AuthResponse, LoginRequest, RefreshRequest, RegisterRequest};
use crate::utils::{ApiError, ApiResponse};
use crate::state::AppState;

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
    // Verify it's a valid refresh token
    let claims = app_state
        .auth_service
        .verify_refresh_token(&req.refresh_token)
        .map_err(|e| ApiError::internal_error(e.to_string()))?
        .ok_or_else(|| ApiError::unauthorized("Invalid or expired refresh token"))?;

    // Generate new access token
    let access_token = app_state
        .auth_service
        .generate_access_from_refresh(&claims)
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    let response = serde_json::json!({
        "access_token": access_token,
    });
    Ok(ApiResponse::success(response))
}