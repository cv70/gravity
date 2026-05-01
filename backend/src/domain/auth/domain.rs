use anyhow::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use super::schema::{AuthResponse, LoginRequest, Organization, RegisterRequest, User, UserResponse};
use crate::domain::automation::domain::AutomationRepository;
use crate::config::ServerConfig;
use crate::datasource::dbdao::DBDao;
use crate::datasource::dbdao::schema::{OrganizationRow, UserRow};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub user_id: String,
    pub tenant_id: String,
    pub token_type: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Clone)]
pub struct AuthService {
    db_dao: DBDao,
    server_config: Arc<ServerConfig>,
}

impl AuthService {
    pub fn new(db_dao: DBDao, server_config: Arc<ServerConfig>) -> Self {
        Self { db_dao, server_config }
    }

    pub async fn register(&self, req: &RegisterRequest) -> Result<AuthResponse> {
        let password_hash = hash(&req.password, DEFAULT_COST)?;

        let org_id = Uuid::new_v4();
        let org_row = self.db_dao.create_organization(
            org_id,
            &req.organization_name,
            "free",
            serde_json::json!({}),
        ).await?;

        let user_id = Uuid::new_v4();
        let user_row = self.db_dao.create_user(
            user_id,
            org_row.id,
            &req.email,
            &password_hash,
            &req.name,
            "organization_owner",
        ).await?;

        let user = User {
            id: user_row.id,
            tenant_id: user_row.tenant_id,
            email: user_row.email,
            password_hash: user_row.password_hash,
            name: user_row.name,
            role: user_row.role,
            last_login_at: user_row.last_login_at,
            created_at: user_row.created_at,
            updated_at: user_row.updated_at,
        };

        let automation_repo = AutomationRepository::new(self.db_dao.clone());
        let _ = automation_repo.bootstrap_defaults(org_row.id).await;

        let tokens = self.generate_tokens(&user)?;
        Ok(tokens)
    }

    pub async fn login(&self, email: &str, password: &str, organization_name: &str) -> Result<Option<AuthResponse>> {
        let org_row = self.db_dao.get_organization_by_name(organization_name).await?;

        let tenant_id = match org_row {
            Some(org) => org.id,
            None => return Ok(None),
        };

        let user_row = self.db_dao.get_user_by_email(email, tenant_id).await?;

        let user_row = match user_row {
            Some(u) => u,
            None => return Ok(None),
        };

        if !verify(password, &user_row.password_hash)? {
            return Ok(None);
        }

        self.db_dao.update_last_login(user_row.id).await?;

        let user = User {
            id: user_row.id,
            tenant_id: user_row.tenant_id,
            email: user_row.email,
            password_hash: user_row.password_hash,
            name: user_row.name,
            role: user_row.role,
            last_login_at: user_row.last_login_at,
            created_at: user_row.created_at,
            updated_at: user_row.updated_at,
        };

        let tokens = self.generate_tokens(&user)?;
        Ok(Some(tokens))
    }

    pub fn generate_tokens(&self, user: &User) -> Result<AuthResponse> {
        let now = chrono::Utc::now().timestamp() as usize;
        let access_exp = now + 15 * 60;
        let refresh_exp = now + 7 * 24 * 60 * 60;

        let access_claims = Claims {
            sub: user.email.clone(),
            user_id: user.id.to_string(),
            tenant_id: user.tenant_id.to_string(),
            token_type: "access".to_string(),
            exp: access_exp,
            iat: now,
        };

        let refresh_claims = Claims {
            sub: user.email.clone(),
            user_id: user.id.to_string(),
            tenant_id: user.tenant_id.to_string(),
            token_type: "refresh".to_string(),
            exp: refresh_exp,
            iat: now,
        };

        let access_token = encode(
            &Header::new(Algorithm::RS256),
            &access_claims,
            &EncodingKey::from_rsa_pem(self.server_config.jwt_private_key.as_bytes())
                .map_err(|e| anyhow::anyhow!("Invalid RSA private key: {}", e))?,
        )?;

        let refresh_token = encode(
            &Header::new(Algorithm::RS256),
            &refresh_claims,
            &EncodingKey::from_rsa_pem(self.server_config.jwt_private_key.as_bytes())
                .map_err(|e| anyhow::anyhow!("Invalid RSA private key: {}", e))?,
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
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_rsa_pem(self.server_config.jwt_public_key.as_bytes())
                .map_err(|e| anyhow::anyhow!("Invalid RSA public key: {}", e))?,
            &validation,
        );

        match token_data {
            Ok(data) => Ok(Some(data.claims)),
            Err(e) => {
                tracing::warn!("JWT verification error: {}", e);
                Ok(None)
            }
        }
    }

    pub fn verify_refresh_token(&self, token: &str) -> Result<Option<Claims>> {
        let claims = self.verify_token(token)?;
        match claims {
            Some(c) if c.token_type == "refresh" => Ok(Some(c)),
            _ => Ok(None),
        }
    }

    pub fn generate_access_from_refresh(&self, refresh_claims: &Claims) -> Result<String> {
        let now = chrono::Utc::now().timestamp() as usize;
        let access_exp = now + 15 * 60;

        let access_claims = Claims {
            sub: refresh_claims.sub.clone(),
            user_id: refresh_claims.user_id.clone(),
            tenant_id: refresh_claims.tenant_id.clone(),
            token_type: "access".to_string(),
            exp: access_exp,
            iat: now,
        };

        encode(
            &Header::new(Algorithm::RS256),
            &access_claims,
            &EncodingKey::from_rsa_pem(self.server_config.jwt_private_key.as_bytes())
                .map_err(|e| anyhow::anyhow!("Invalid RSA private key: {}", e))?,
        )
        .map_err(|e| anyhow::anyhow!("Failed to generate access token: {}", e).into())
    }
}
