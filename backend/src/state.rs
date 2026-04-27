use std::sync::Arc;
use axum::{
    body::Body,
    extract::{Request, Extension},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::domain::auth::domain::AuthService;

#[derive(Clone, Copy)]
pub struct UserContext {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
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

    if let Some(token) = token {
        if let Some(claims) = auth_service.verify_token(token).ok().flatten() {
            if let (Ok(user_id), Ok(tenant_id)) = (
                Uuid::parse_str(&claims.user_id),
                Uuid::parse_str(&claims.tenant_id),
            ) {
                request.extensions_mut().insert(UserContext { user_id, tenant_id });
            }
        }
    }

    next.run(request).await
}

#[derive(Clone)]
pub struct AppState {
    pub infra: Infra,
    pub config: AppConfig,
    pub auth_service: AuthService,
}

use crate::datasource::Infra;
use crate::config::AppConfig;

impl AppState {
    pub async fn new(config: AppConfig) -> Result<Self, sqlx::Error> {
        let infra = Infra::new(&config).await?;
        let auth_service = AuthService::new(infra.db.clone(), Arc::new(config.server.clone()));

        Ok(Self {
            infra,
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

impl axum::extract::FromRef<AppState> for Infra {
    fn from_ref(state: &AppState) -> Self {
        state.infra.clone()
    }
}
