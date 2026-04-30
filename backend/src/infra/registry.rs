use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;

use crate::config::AppConfig;
use crate::datasource::dbdao::DBDao;

#[derive(Clone)]
pub struct Registry {
    pub db: PgPool,
    pub db_dao: DBDao,
    pub config: Arc<AppConfig>,
}

impl Registry {
    pub async fn new(config: &AppConfig) -> Result<Self> {
        let db = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.database.max_connections)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .connect(&config.database.connection_string())
            .await?;

        Ok(Self {
            db_dao: DBDao::new(db.clone()),
            db,
            config: Arc::new(config.clone()),
        })
    }
}
