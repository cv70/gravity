use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;

use super::dao::DBDao;
use super::schema::SegmentRow;

impl DBDao {
    pub async fn create_segment(
        &self,
        id: Uuid,
        tenant_id: Uuid,
        name: &str,
        definition: serde_json::Value,
        is_dynamic: bool,
        status: &str,
    ) -> Result<SegmentRow> {
        let row = sqlx::query_as::<_, SegmentRow>(
            r#"
            INSERT INTO segments (id, tenant_id, name, definition, is_dynamic, status)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, tenant_id, name, definition, is_dynamic, status, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(name)
        .bind(definition)
        .bind(is_dynamic)
        .bind(status)
        .fetch_one(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn list_segments(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<SegmentRow>, i64)> {
        let rows = sqlx::query_as::<_, SegmentRow>(
            r#"
            SELECT id, tenant_id, name, definition, is_dynamic, status, created_at, updated_at
            FROM segments
            WHERE tenant_id = $1
            ORDER BY updated_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let count_row = sqlx::query("SELECT COUNT(*) FROM segments WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_one(&self.db)
            .await?;

        Ok((rows, count_row.get::<i64, _>(0)))
    }

    pub async fn get_segment_by_id(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Option<SegmentRow>> {
        let row = sqlx::query_as::<_, SegmentRow>(
            "SELECT id, tenant_id, name, definition, is_dynamic, status, created_at, updated_at FROM segments WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn update_segment(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        name: Option<&str>,
        definition: Option<serde_json::Value>,
        is_dynamic: Option<bool>,
        status: Option<&str>,
    ) -> Result<Option<SegmentRow>> {
        let row = sqlx::query_as::<_, SegmentRow>(
            r#"
            UPDATE segments
            SET name = COALESCE($1, name),
                definition = COALESCE($2, definition),
                is_dynamic = COALESCE($3, is_dynamic),
                status = COALESCE($4, status),
                updated_at = NOW()
            WHERE id = $5 AND tenant_id = $6
            RETURNING id, tenant_id, name, definition, is_dynamic, status, created_at, updated_at
            "#,
        )
        .bind(name)
        .bind(definition)
        .bind(is_dynamic)
        .bind(status)
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn delete_segment(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM segments WHERE id = $1 AND tenant_id = $2")
            .bind(id)
            .bind(tenant_id)
            .execute(&self.db)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
