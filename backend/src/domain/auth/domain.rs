use anyhow::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use super::schema::{AuthResponse, Organization, RegisterRequest, User, UserResponse};
use crate::config::ServerConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub user_id: String,
    pub tenant_id: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Clone)]
pub struct AuthService {
    pool: PgPool,
    server_config: Arc<ServerConfig>,
}

impl AuthService {
    pub fn new(pool: PgPool, server_config: Arc<ServerConfig>) -> Self {
        Self { pool, server_config }
    }

    pub async fn register(&self, req: &RegisterRequest) -> Result<AuthResponse> {
        let password_hash = hash(&req.password, DEFAULT_COST)?;

        // Create organization
        let org = Organization::new(req.organization_name.clone());
        let org: Organization = sqlx::query_as(
            "INSERT INTO organizations (id, name, plan, settings) VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(org.id)
        .bind(&org.name)
        .bind(&org.plan)
        .bind(&org.settings)
        .fetch_one(&self.pool)
        .await?;

        // Create user
        let mut user = User::new(org.id, req.email.clone(), password_hash, req.name.clone());
        user.role = "organization_owner".to_string();

        let user: User = sqlx::query_as(
            r#"
            INSERT INTO users (id, tenant_id, email, password_hash, name, role)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(user.id)
        .bind(user.tenant_id)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.name)
        .bind(&user.role)
        .fetch_one(&self.pool)
        .await?;

        let tokens = self.generate_tokens(&user)?;
        Ok(tokens)
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<Option<AuthResponse>> {
        let user: Option<User> = sqlx::query_as("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        let user = match user {
            Some(u) => u,
            None => return Ok(None),
        };

        if !verify(password, &user.password_hash)? {
            return Ok(None);
        }

        // Update last login
        sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = $1")
            .bind(user.id)
            .execute(&self.pool)
            .await?;

        let tokens = self.generate_tokens(&user)?;
        Ok(Some(tokens))
    }

    pub fn generate_tokens(&self, user: &User) -> Result<AuthResponse> {
        let now = chrono::Utc::now().timestamp() as usize;
        let access_exp = now + 15 * 60; // 15 minutes
        let refresh_exp = now + 7 * 24 * 60 * 60; // 7 days

        let access_claims = Claims {
            sub: user.email.clone(),
            user_id: user.id.to_string(),
            tenant_id: user.tenant_id.to_string(),
            exp: access_exp,
            iat: now,
        };

        let refresh_claims = Claims {
            sub: user.email.clone(),
            user_id: user.id.to_string(),
            tenant_id: user.tenant_id.to_string(),
            exp: refresh_exp,
            iat: now,
        };

        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(self.server_config.jwt_secret.as_bytes()),
        )?;

        let refresh_token = encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(self.server_config.jwt_secret.as_bytes()),
        )?;

        Ok(AuthResponse {
            user: UserResponse {
                id: user.id,
                email: user.email.clone(),
                name: user.name.clone(),
                role: user.role.clone(),
            },
            access_token,
            refresh_token,
        })
    }

    pub fn verify_token(&self, token: &str) -> Result<Option<Claims>> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.server_config.jwt_secret.as_bytes()),
            &Validation::default(),
        );

        match token_data {
            Ok(data) => Ok(Some(data.claims)),
            Err(_) => Ok(None),
        }
    }
}
