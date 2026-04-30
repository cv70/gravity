use anyhow::Result;
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct DBDao {
    pub db: PgPool,
}

impl DBDao {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn begin(&self) -> Result<sqlx::Transaction<'_, sqlx::Postgres>> {
        let tx = self.db.begin().await?;
        Ok(tx)
    }
}
