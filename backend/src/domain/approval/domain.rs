use anyhow::Result;
use uuid::Uuid;

use crate::datasource::dbdao::DBDao;

use super::schema::{Approval, ApprovalListResponse, CreateApprovalRequest, ReviewApprovalRequest};

pub struct ApprovalRepository {
    db_dao: DBDao,
}

impl ApprovalRepository {
    pub fn new(db_dao: DBDao) -> Self {
        Self { db_dao }
    }

    pub async fn create(
        &self,
        tenant_id: Uuid,
        requested_by: Uuid,
        req: &CreateApprovalRequest,
    ) -> Result<Approval> {
        let row = sqlx::query_as::<_, crate::datasource::dbdao::schema::ApprovalRow>(
            r#"
            INSERT INTO approvals (id, tenant_id, object_type, object_id, status, requested_by, reason)
            VALUES ($1, $2, $3, $4, 'pending', $5, $6)
            RETURNING id, tenant_id, object_type, object_id, status, requested_by, approved_by, reason, decided_at, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(tenant_id)
        .bind(&req.object_type)
        .bind(req.object_id)
        .bind(requested_by)
        .bind(req.reason.as_deref())
        .fetch_one(&self.db_dao.db)
        .await?;

        Ok(Self::to_approval(row))
    }

    pub async fn list(
        &self,
        tenant_id: Uuid,
        page: i64,
        limit: i64,
        status: Option<&str>,
        object_type: Option<&str>,
    ) -> Result<(Vec<Approval>, i64)> {
        let offset = (page - 1) * limit;
        let (rows, total) = self
            .db_dao
            .list_approvals(tenant_id, limit, offset, status, object_type)
            .await?;
        Ok((rows.into_iter().map(Self::to_approval).collect(), total))
    }

    pub async fn review(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        reviewer: Uuid,
        req: &ReviewApprovalRequest,
    ) -> Result<Option<Approval>> {
        let status = if req.approved { "approved" } else { "rejected" };
        let row = sqlx::query_as::<_, crate::datasource::dbdao::schema::ApprovalRow>(
            r#"
            UPDATE approvals
            SET status = $1,
                approved_by = COALESCE($2, approved_by),
                decided_at = NOW(),
                reason = COALESCE($3, reason),
                updated_at = NOW()
            WHERE id = $4 AND tenant_id = $5
            RETURNING id, tenant_id, object_type, object_id, status, requested_by, approved_by, reason, decided_at, created_at, updated_at
            "#,
        )
        .bind(status)
        .bind(Some(reviewer))
        .bind(req.note.as_deref())
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.db_dao.db)
        .await?;

        Ok(row.map(Self::to_approval))
    }

    fn to_approval(row: crate::datasource::dbdao::schema::ApprovalRow) -> Approval {
        Approval {
            id: row.id,
            tenant_id: row.tenant_id,
            object_type: row.object_type,
            object_id: row.object_id,
            status: row.status,
            requested_by: row.requested_by,
            approved_by: row.approved_by,
            reason: row.reason,
            decided_at: row.decided_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
