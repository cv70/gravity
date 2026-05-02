use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;

use super::dao::DBDao;
use super::schema::ApprovalRow;

impl DBDao {
    pub async fn list_approvals(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
        status: Option<&str>,
        object_type: Option<&str>,
    ) -> Result<(Vec<ApprovalRow>, i64)> {
        let rows = sqlx::query_as::<_, ApprovalRow>(
            r#"
            SELECT id, tenant_id, object_type, object_id, status, requested_by, approved_by, reason, decided_at, created_at, updated_at
            FROM approvals
            WHERE tenant_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::text IS NULL OR object_type = $3)
            ORDER BY created_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(tenant_id)
        .bind(status)
        .bind(object_type)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let total = sqlx::query(
            r#"
            SELECT COUNT(*)
            FROM approvals
            WHERE tenant_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::text IS NULL OR object_type = $3)
            "#,
        )
            .bind(tenant_id)
            .bind(status)
            .bind(object_type)
            .fetch_one(&self.db)
            .await?
            .get::<i64, _>(0);

        Ok((rows, total))
    }
}
