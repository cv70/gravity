use anyhow::Result;
use uuid::Uuid;

use crate::datasource::dbdao::DBDao;

use super::schema::{AuditLog, AuditLogListResponse};

pub struct AuditRepository {
    db_dao: DBDao,
}

impl AuditRepository {
    pub fn new(db_dao: DBDao) -> Self {
        Self { db_dao }
    }

    pub async fn record(
        &self,
        tenant_id: Uuid,
        user_id: Option<Uuid>,
        action: &str,
        target_type: Option<&str>,
        target_id: Option<Uuid>,
        metadata: serde_json::Value,
    ) -> Result<AuditLog> {
        let row = self
            .db_dao
            .create_audit_log(
                Uuid::new_v4(),
                tenant_id,
                user_id,
                action,
                target_type,
                target_id,
                metadata,
            )
            .await?;
        Ok(Self::to_audit_log(row))
    }

    pub async fn list(
        &self,
        tenant_id: Uuid,
        page: i64,
        limit: i64,
        action: Option<&str>,
        target_type: Option<&str>,
        start_at: Option<chrono::DateTime<chrono::Utc>>,
        end_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(Vec<AuditLog>, i64)> {
        let offset = (page - 1) * limit;
        let (rows, total) = self
            .db_dao
            .list_audit_logs(tenant_id, limit, offset, action, target_type, start_at, end_at)
            .await?;
        Ok((rows.into_iter().map(Self::to_audit_log).collect(), total))
    }

    fn to_audit_log(row: crate::datasource::dbdao::schema::AuditLogRow) -> AuditLog {
        AuditLog {
            id: row.id,
            tenant_id: row.tenant_id,
            user_id: row.user_id,
            action: row.action,
            target_type: row.target_type,
            target_id: row.target_id,
            metadata: row.metadata,
            created_at: row.created_at,
        }
    }
}
