use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use super::schema::{Content, CreateContentRequest};

pub struct ContentRepository {
    pool: PgPool,
}

impl ContentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, tenant_id: Uuid, req: &CreateContentRequest) -> Result<Content> {
        let content = Content::new(tenant_id, req.name.clone(), req.content_type.clone(), req.content.clone());

        let result = sqlx::query_as::<_, Content>(
            r#"
            INSERT INTO contents (id, tenant_id, campaign_id, name, content_type, content, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(content.id)
        .bind(content.tenant_id)
        .bind(&content.campaign_id)
        .bind(&content.name)
        .bind(&content.content_type)
        .bind(&content.content)
        .bind(&content.status)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn list(&self, tenant_id: Uuid) -> Result<Vec<Content>> {
        let contents = sqlx::query_as::<_, Content>(
            "SELECT * FROM contents WHERE tenant_id = $1 ORDER BY created_at DESC",
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(contents)
    }

    pub async fn get_by_id(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Content>> {
        let result = sqlx::query_as::<_, Content>(
            "SELECT * FROM contents WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn delete(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM contents WHERE id = $1 AND tenant_id = $2")
            .bind(id)
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
