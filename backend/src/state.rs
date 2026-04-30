use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Extension, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use crate::domain::auth::domain::AuthService;
use crate::infra::Registry;

#[derive(Clone, Copy)]
pub struct UserContext {
    pub user_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
}

pub async fn auth_middleware(
    Extension(auth_service): Extension<AuthService>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let token = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    match token {
        Some(token) => {
            let claims = match auth_service.verify_token(token) {
                Ok(Some(c)) => c,
                Ok(None) => {
                    return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"code": 401, "message": "Invalid token"}))).into_response();
                }
                Err(e) => {
                    tracing::warn!("JWT verification error: {}", e);
                    return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"code": 401, "message": "Invalid token"}))).into_response();
                }
            };

            let user_id = match uuid::Uuid::parse_str(&claims.user_id) {
                Ok(id) => id,
                Err(_) => {
                    return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"code": 401, "message": "Invalid token claims"}))).into_response();
                }
            };
            let tenant_id = match uuid::Uuid::parse_str(&claims.tenant_id) {
                Ok(id) => id,
                Err(_) => {
                    return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"code": 401, "message": "Invalid token claims"}))).into_response();
                }
            };

            request.extensions_mut().insert(UserContext { user_id, tenant_id });
            next.run(request).await
        }
        None => {
            (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"code": 401, "message": "Missing authorization token"}))).into_response()
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub registry: Registry,
    pub config: crate::config::AppConfig,
    pub auth_service: AuthService,
}

impl AppState {
    pub async fn new(config: crate::config::AppConfig) -> anyhow::Result<Self> {
        let registry = Registry::new(&config).await?;
        let auth_service = AuthService::new(registry.db_dao.clone(), Arc::new(config.server.clone()));

        Ok(Self {
            registry,
            config,
            auth_service,
        })
    }
}

impl axum::extract::FromRef<AppState> for AuthService {
    fn from_ref(state: &AppState) -> Self {
        state.auth_service.clone()
    }
}

impl axum::extract::FromRef<AppState> for Registry {
    fn from_ref(state: &AppState) -> Self {
        state.registry.clone()
    }
}
