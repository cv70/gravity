use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;

use super::dao::DBDao;
use super::schema::ContentRow;

impl DBDao {
    pub async fn create_content(
        &self,
        id: Uuid,
        tenant_id: Uuid,
        campaign_id: Option<Uuid>,
        name: &str,
        content_type: &str,
        content: serde_json::Value,
        status: &str,
    ) -> Result<ContentRow> {
        let row = sqlx::query_as::<_, ContentRow>(
            r#"
            INSERT INTO contents (id, tenant_id, campaign_id, name, content_type, content, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, tenant_id, campaign_id, name, content_type, content, status, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(campaign_id)
        .bind(name)
        .bind(content_type)
        .bind(&content)
        .bind(status)
        .fetch_one(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn list_contents(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ContentRow>, i64)> {
        let rows = sqlx::query_as::<_, ContentRow>(
            r#"
            SELECT id, tenant_id, campaign_id, name, content_type, content, status, created_at, updated_at
            FROM contents
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let count_row = sqlx::query("SELECT COUNT(*) FROM contents WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_one(&self.db)
            .await?;

        let total: i64 = count_row.get(0);

        Ok((rows, total))
    }

    pub async fn get_content_by_id(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<ContentRow>> {
        let row = sqlx::query_as::<_, ContentRow>(
            "SELECT id, tenant_id, campaign_id, name, content_type, content, status, created_at, updated_at FROM contents WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn delete_content(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM contents WHERE id = $1 AND tenant_id = $2")
            .bind(id)
            .bind(tenant_id)
            .execute(&self.db)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
