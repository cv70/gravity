use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;

use super::dao::DBDao;
use super::schema::AuditLogRow;

impl DBDao {
    pub async fn create_audit_log(
        &self,
        id: Uuid,
        tenant_id: Uuid,
        user_id: Option<Uuid>,
        action: &str,
        target_type: Option<&str>,
        target_id: Option<Uuid>,
        metadata: serde_json::Value,
    ) -> Result<AuditLogRow> {
        let row = sqlx::query_as::<_, AuditLogRow>(
            r#"
            INSERT INTO audit_logs (
                id, tenant_id, user_id, action, target_type, target_id, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, tenant_id, user_id, action, target_type, target_id, metadata, created_at
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(user_id)
        .bind(action)
        .bind(target_type)
        .bind(target_id)
        .bind(metadata)
        .fetch_one(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn list_audit_logs(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
        action: Option<&str>,
        target_type: Option<&str>,
        start_at: Option<chrono::DateTime<chrono::Utc>>,
        end_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(Vec<AuditLogRow>, i64)> {
        let rows = sqlx::query_as::<_, AuditLogRow>(
            r#"
            SELECT id, tenant_id, user_id, action, target_type, target_id, metadata, created_at
            FROM audit_logs
            WHERE tenant_id = $1
              AND ($2::text IS NULL OR action ILIKE $2)
              AND ($3::text IS NULL OR target_type = $3)
              AND ($4::timestamptz IS NULL OR created_at >= $4)
              AND ($5::timestamptz IS NULL OR created_at <= $5)
            ORDER BY created_at DESC
            LIMIT $6 OFFSET $7
            "#,
        )
        .bind(tenant_id)
        .bind(action)
        .bind(target_type)
        .bind(start_at)
        .bind(end_at)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let total = sqlx::query(
            r#"
            SELECT COUNT(*)
            FROM audit_logs
            WHERE tenant_id = $1
              AND ($2::text IS NULL OR action ILIKE $2)
              AND ($3::text IS NULL OR target_type = $3)
              AND ($4::timestamptz IS NULL OR created_at >= $4)
              AND ($5::timestamptz IS NULL OR created_at <= $5)
            "#,
        )
        .bind(tenant_id)
        .bind(action)
        .bind(target_type)
        .bind(start_at)
        .bind(end_at)
        .fetch_one(&self.db)
        .await?
        .get::<i64, _>(0);

        Ok((rows, total))
    }
}
