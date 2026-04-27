pub mod dbdao;
pub mod scylladao;
pub mod vectordao;

pub use dbdao::create_pg_pool;

use std::sync::Arc;
use sqlx::PgPool;
use crate::config::AppConfig;

#[derive(Clone)]
pub struct Infra {
    pub db: PgPool,
}

impl Infra {
    pub async fn new(config: &AppConfig) -> Result<Self, sqlx::Error> {
        let db = dbdao::create_pg_pool(&config.database).await?;
        Ok(Self { db })
    }
}
