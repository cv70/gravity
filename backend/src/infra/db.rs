use crate::config::DatabaseConfig;
use crate::datasource::dbdao::DBDao;

use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

pub async fn new_db(c: &DatabaseConfig) -> Result<DBDao> {
    let pool = PgPoolOptions::new()
        .max_connections(c.max_connections)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&c.connection_string())
        .await?;
    Ok(DBDao::new(pool))
}
