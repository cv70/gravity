use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;

use super::dao::DBDao;
use super::schema::{WorkflowExecutionRow, WorkflowRow, WorkflowScheduleRow};

impl DBDao {
    pub async fn create_workflow(
        &self,
        id: Uuid,
        tenant_id: Uuid,
        name: &str,
        description: Option<&str>,
        version: i32,
        trigger_type: &str,
        trigger_config: serde_json::Value,
        steps: serde_json::Value,
        status: &str,
    ) -> Result<WorkflowRow> {
        let row = sqlx::query_as::<_, WorkflowRow>(
            r#"
            INSERT INTO workflows (
                id, tenant_id, name, description, version, trigger_type, trigger_config, steps, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, tenant_id, name, description, version, trigger_type, trigger_config, steps, status, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(name)
        .bind(description)
        .bind(version)
        .bind(trigger_type)
        .bind(trigger_config)
        .bind(steps)
        .bind(status)
        .fetch_one(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn list_workflows(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<WorkflowRow>, i64)> {
        let rows = sqlx::query_as::<_, WorkflowRow>(
            r#"
            SELECT id, tenant_id, name, description, version, trigger_type, trigger_config, steps, status, created_at, updated_at
            FROM workflows
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

        let count_row = sqlx::query("SELECT COUNT(*) FROM workflows WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_one(&self.db)
            .await?;

        Ok((rows, count_row.get::<i64, _>(0)))
    }

    pub async fn get_workflow_by_id(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Option<WorkflowRow>> {
        let row = sqlx::query_as::<_, WorkflowRow>(
            "SELECT id, tenant_id, name, description, version, trigger_type, trigger_config, steps, status, created_at, updated_at FROM workflows WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn update_workflow(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        version: Option<i32>,
        trigger_type: Option<&str>,
        trigger_config: Option<serde_json::Value>,
        steps: Option<serde_json::Value>,
        status: Option<&str>,
    ) -> Result<Option<WorkflowRow>> {
        let row = sqlx::query_as::<_, WorkflowRow>(
            r#"
            UPDATE workflows
            SET name = COALESCE($1, name),
                description = COALESCE($2, description),
                version = COALESCE($3, version),
                trigger_type = COALESCE($4, trigger_type),
                trigger_config = COALESCE($5, trigger_config),
                steps = COALESCE($6, steps),
                status = COALESCE($7, status),
                updated_at = NOW()
            WHERE id = $8 AND tenant_id = $9
            RETURNING id, tenant_id, name, description, version, trigger_type, trigger_config, steps, status, created_at, updated_at
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(version)
        .bind(trigger_type)
        .bind(trigger_config)
        .bind(steps)
        .bind(status)
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn delete_workflow(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM workflows WHERE id = $1 AND tenant_id = $2")
            .bind(id)
            .bind(tenant_id)
            .execute(&self.db)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn create_workflow_execution(
        &self,
        id: Uuid,
        tenant_id: Uuid,
        workflow_id: Uuid,
        contact_id: Uuid,
        status: &str,
        current_step_index: i32,
        context: serde_json::Value,
    ) -> Result<WorkflowExecutionRow> {
        let row = sqlx::query_as::<_, WorkflowExecutionRow>(
            r#"
            INSERT INTO workflow_executions (
                id, tenant_id, workflow_id, contact_id, status, current_step_index, context
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, tenant_id, workflow_id, contact_id, status, current_step_index, context, started_at, completed_at
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(workflow_id)
        .bind(contact_id)
        .bind(status)
        .bind(current_step_index)
        .bind(context)
        .fetch_one(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn list_workflow_executions(
        &self,
        tenant_id: Uuid,
        workflow_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<WorkflowExecutionRow>, i64)> {
        let rows = if let Some(workflow_id) = workflow_id {
            sqlx::query_as::<_, WorkflowExecutionRow>(
                r#"
                SELECT id, tenant_id, workflow_id, contact_id, status, current_step_index, context, started_at, completed_at
                FROM workflow_executions
                WHERE tenant_id = $1 AND workflow_id = $2
                ORDER BY started_at DESC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(tenant_id)
            .bind(workflow_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query_as::<_, WorkflowExecutionRow>(
                r#"
                SELECT id, tenant_id, workflow_id, contact_id, status, current_step_index, context, started_at, completed_at
                FROM workflow_executions
                WHERE tenant_id = $1
                ORDER BY started_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(tenant_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await?
        };

        let total = if let Some(workflow_id) = workflow_id {
            sqlx::query("SELECT COUNT(*) FROM workflow_executions WHERE tenant_id = $1 AND workflow_id = $2")
                .bind(tenant_id)
                .bind(workflow_id)
                .fetch_one(&self.db)
                .await?
                .get::<i64, _>(0)
        } else {
            sqlx::query("SELECT COUNT(*) FROM workflow_executions WHERE tenant_id = $1")
                .bind(tenant_id)
                .fetch_one(&self.db)
                .await?
                .get::<i64, _>(0)
        };

        Ok((rows, total))
    }

    pub async fn get_workflow_execution_by_id(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Option<WorkflowExecutionRow>> {
        let row = sqlx::query_as::<_, WorkflowExecutionRow>(
            "SELECT id, tenant_id, workflow_id, contact_id, status, current_step_index, context, started_at, completed_at FROM workflow_executions WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn update_workflow_execution(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        status: &str,
        current_step_index: Option<i32>,
        context: Option<serde_json::Value>,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Option<WorkflowExecutionRow>> {
        let row = sqlx::query_as::<_, WorkflowExecutionRow>(
            r#"
            UPDATE workflow_executions
            SET status = $1,
                current_step_index = COALESCE($2, current_step_index),
                context = COALESCE($3, context),
                completed_at = COALESCE($4, completed_at)
            WHERE id = $5 AND tenant_id = $6
            RETURNING id, tenant_id, workflow_id, contact_id, status, current_step_index, context, started_at, completed_at
            "#,
        )
        .bind(status)
        .bind(current_step_index)
        .bind(context)
        .bind(completed_at)
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn create_workflow_schedule(
        &self,
        id: Uuid,
        tenant_id: Uuid,
        workflow_execution_id: Uuid,
        resume_at: chrono::DateTime<chrono::Utc>,
        payload: serde_json::Value,
        status: &str,
    ) -> Result<WorkflowScheduleRow> {
        let row = sqlx::query_as::<_, WorkflowScheduleRow>(
            r#"
            INSERT INTO workflow_schedules (
                id, tenant_id, workflow_execution_id, resume_at, payload, status
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, tenant_id, workflow_execution_id, resume_at, payload, status, created_at
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(workflow_execution_id)
        .bind(resume_at)
        .bind(payload)
        .bind(status)
        .fetch_one(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn list_workflow_schedules(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<WorkflowScheduleRow>, i64)> {
        let rows = sqlx::query_as::<_, WorkflowScheduleRow>(
            r#"
            SELECT id, tenant_id, workflow_execution_id, resume_at, payload, status, created_at
            FROM workflow_schedules
            WHERE tenant_id = $1
            ORDER BY resume_at ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let total = sqlx::query("SELECT COUNT(*) FROM workflow_schedules WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_one(&self.db)
            .await?
            .get::<i64, _>(0);

        Ok((rows, total))
    }
}
