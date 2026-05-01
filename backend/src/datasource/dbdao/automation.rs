use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;

use super::dao::DBDao;
use super::schema::{
    ApprovalRequestRow, AutomationActionRow, AutomationJobRow, AutomationRunRow, ExperimentRow,
    PolicyRuleRow,
};

impl DBDao {
    pub async fn create_automation_job(
        &self,
        id: Uuid,
        tenant_id: Uuid,
        goal: &str,
        target_audience: serde_json::Value,
        channel_preferences: Vec<String>,
        strategy: serde_json::Value,
        status: &str,
        risk_level: &str,
        approval_required: bool,
        budget_limit: Option<f64>,
        currency: &str,
        next_action_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<AutomationJobRow> {
        let row = sqlx::query_as::<_, AutomationJobRow>(
            r#"
            INSERT INTO automation_jobs (
                id, tenant_id, goal, target_audience, channel_preferences, strategy,
                status, risk_level, approval_required, budget_limit, currency, next_action_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, tenant_id, goal, target_audience, channel_preferences, strategy,
                      status, risk_level, approval_required, budget_limit, currency, next_action_at,
                      created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(goal)
        .bind(target_audience)
        .bind(channel_preferences)
        .bind(strategy)
        .bind(status)
        .bind(risk_level)
        .bind(approval_required)
        .bind(budget_limit)
        .bind(currency)
        .bind(next_action_at)
        .fetch_one(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn get_automation_job_by_id(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Option<AutomationJobRow>> {
        let row = sqlx::query_as::<_, AutomationJobRow>(
            "SELECT id, tenant_id, goal, target_audience, channel_preferences, strategy, status, risk_level, approval_required, budget_limit, currency, next_action_at, created_at, updated_at FROM automation_jobs WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn list_automation_jobs(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<AutomationJobRow>, i64)> {
        let rows = sqlx::query_as::<_, AutomationJobRow>(
            r#"
            SELECT id, tenant_id, goal, target_audience, channel_preferences, strategy, status,
                   risk_level, approval_required, budget_limit, currency, next_action_at,
                   created_at, updated_at
            FROM automation_jobs
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

        let count_row = sqlx::query("SELECT COUNT(*) FROM automation_jobs WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_one(&self.db)
            .await?;

        Ok((rows, count_row.get::<i64, _>(0)))
    }

    pub async fn list_automation_jobs_global(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<AutomationJobRow>, i64)> {
        let rows = sqlx::query_as::<_, AutomationJobRow>(
            r#"
            SELECT id, tenant_id, goal, target_audience, channel_preferences, strategy, status,
                   risk_level, approval_required, budget_limit, currency, next_action_at,
                   created_at, updated_at
            FROM automation_jobs
            ORDER BY COALESCE(next_action_at, created_at) ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let count_row = sqlx::query("SELECT COUNT(*) FROM automation_jobs")
            .fetch_one(&self.db)
            .await?;

        Ok((rows, count_row.get::<i64, _>(0)))
    }

    pub async fn update_automation_job_status(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        status: &str,
        next_action_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Option<AutomationJobRow>> {
        let row = sqlx::query_as::<_, AutomationJobRow>(
            r#"
            UPDATE automation_jobs
            SET status = $1,
                next_action_at = COALESCE($2, next_action_at),
                updated_at = NOW()
            WHERE id = $3 AND tenant_id = $4
            RETURNING id, tenant_id, goal, target_audience, channel_preferences, strategy,
                      status, risk_level, approval_required, budget_limit, currency, next_action_at,
                      created_at, updated_at
            "#,
        )
        .bind(status)
        .bind(next_action_at)
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn create_automation_run(
        &self,
        id: Uuid,
        tenant_id: Uuid,
        job_id: Uuid,
        status: &str,
        current_step: &str,
        input_context: serde_json::Value,
        output_context: serde_json::Value,
    ) -> Result<AutomationRunRow> {
        let row = sqlx::query_as::<_, AutomationRunRow>(
            r#"
            INSERT INTO automation_runs (
                id, tenant_id, job_id, status, current_step, input_context, output_context
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, tenant_id, job_id, status, current_step, input_context, output_context,
                      started_at, completed_at, last_error
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(job_id)
        .bind(status)
        .bind(current_step)
        .bind(input_context)
        .bind(output_context)
        .fetch_one(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn list_automation_runs(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<AutomationRunRow>, i64)> {
        let rows = sqlx::query_as::<_, AutomationRunRow>(
            r#"
            SELECT id, tenant_id, job_id, status, current_step, input_context, output_context,
                   started_at, completed_at, last_error
            FROM automation_runs
            WHERE tenant_id = $1
            ORDER BY started_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let count_row = sqlx::query("SELECT COUNT(*) FROM automation_runs WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_one(&self.db)
            .await?;

        Ok((rows, count_row.get::<i64, _>(0)))
    }

    pub async fn list_automation_runs_global(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<AutomationRunRow>, i64)> {
        let rows = sqlx::query_as::<_, AutomationRunRow>(
            r#"
            SELECT id, tenant_id, job_id, status, current_step, input_context, output_context,
                   started_at, completed_at, last_error
            FROM automation_runs
            WHERE status IN ('queued', 'running', 'waiting_approval')
            ORDER BY started_at ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let count_row = sqlx::query("SELECT COUNT(*) FROM automation_runs WHERE status IN ('queued', 'running', 'waiting_approval')")
            .fetch_one(&self.db)
            .await?;

        Ok((rows, count_row.get::<i64, _>(0)))
    }

    pub async fn create_automation_action(
        &self,
        id: Uuid,
        tenant_id: Uuid,
        run_id: Uuid,
        action_type: &str,
        channel: &str,
        payload: serde_json::Value,
        risk_level: &str,
        status: &str,
        requires_approval: bool,
    ) -> Result<AutomationActionRow> {
        let row = sqlx::query_as::<_, AutomationActionRow>(
            r#"
            INSERT INTO automation_actions (
                id, tenant_id, run_id, action_type, channel, payload, risk_level, status, requires_approval
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, tenant_id, run_id, action_type, channel, payload, risk_level, status,
                      requires_approval, approved_by, approved_at, executed_at, failure_reason,
                      created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(run_id)
        .bind(action_type)
        .bind(channel)
        .bind(payload)
        .bind(risk_level)
        .bind(status)
        .bind(requires_approval)
        .fetch_one(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn update_automation_action_status(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        approved_at: Option<chrono::DateTime<chrono::Utc>>,
        executed_at: Option<chrono::DateTime<chrono::Utc>>,
        failure_reason: Option<&str>,
    ) -> Result<Option<AutomationActionRow>> {
        let row = sqlx::query_as::<_, AutomationActionRow>(
            r#"
            UPDATE automation_actions
            SET status = $1,
                approved_by = COALESCE($2, approved_by),
                approved_at = COALESCE($3, approved_at),
                executed_at = COALESCE($4, executed_at),
                failure_reason = COALESCE($5, failure_reason),
                updated_at = NOW()
            WHERE id = $6 AND tenant_id = $7
            RETURNING id, tenant_id, run_id, action_type, channel, payload, risk_level, status,
                      requires_approval, approved_by, approved_at, executed_at, failure_reason,
                      created_at, updated_at
            "#,
        )
        .bind(status)
        .bind(approved_by)
        .bind(approved_at)
        .bind(executed_at)
        .bind(failure_reason)
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn update_automation_run_status(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        status: &str,
        current_step: Option<&str>,
        output_context: Option<serde_json::Value>,
        last_error: Option<&str>,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Option<AutomationRunRow>> {
        let row = sqlx::query_as::<_, AutomationRunRow>(
            r#"
            UPDATE automation_runs
            SET status = $1,
                current_step = COALESCE($2, current_step),
                output_context = COALESCE($3, output_context),
                last_error = COALESCE($4, last_error),
                completed_at = COALESCE($5, completed_at)
            WHERE id = $6 AND tenant_id = $7
            RETURNING id, tenant_id, job_id, status, current_step, input_context, output_context,
                      started_at, completed_at, last_error
            "#,
        )
        .bind(status)
        .bind(current_step)
        .bind(output_context)
        .bind(last_error)
        .bind(completed_at)
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn get_automation_run_by_id(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Option<AutomationRunRow>> {
        let row = sqlx::query_as::<_, AutomationRunRow>(
            "SELECT id, tenant_id, job_id, status, current_step, input_context, output_context, started_at, completed_at, last_error FROM automation_runs WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn list_automation_actions(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<AutomationActionRow>, i64)> {
        let rows = sqlx::query_as::<_, AutomationActionRow>(
            r#"
            SELECT id, tenant_id, run_id, action_type, channel, payload, risk_level, status,
                   requires_approval, approved_by, approved_at, executed_at, failure_reason,
                   created_at, updated_at
            FROM automation_actions
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

        let count_row = sqlx::query("SELECT COUNT(*) FROM automation_actions WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_one(&self.db)
            .await?;

        Ok((rows, count_row.get::<i64, _>(0)))
    }

    pub async fn get_automation_action_by_id(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Option<AutomationActionRow>> {
        let row = sqlx::query_as::<_, AutomationActionRow>(
            "SELECT id, tenant_id, run_id, action_type, channel, payload, risk_level, status, requires_approval, approved_by, approved_at, executed_at, failure_reason, created_at, updated_at FROM automation_actions WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn create_approval_request(
        &self,
        id: Uuid,
        tenant_id: Uuid,
        action_id: Uuid,
        title: &str,
        reason: &str,
        status: &str,
        requested_by: Option<Uuid>,
    ) -> Result<ApprovalRequestRow> {
        let row = sqlx::query_as::<_, ApprovalRequestRow>(
            r#"
            INSERT INTO approval_requests (
                id, tenant_id, action_id, title, reason, status, requested_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, tenant_id, action_id, title, reason, status, requested_by,
                      reviewed_by, reviewed_at, decision_note, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(action_id)
        .bind(title)
        .bind(reason)
        .bind(status)
        .bind(requested_by)
        .fetch_one(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn list_approval_requests(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ApprovalRequestRow>, i64)> {
        let rows = sqlx::query_as::<_, ApprovalRequestRow>(
            r#"
            SELECT id, tenant_id, action_id, title, reason, status, requested_by,
                   reviewed_by, reviewed_at, decision_note, created_at, updated_at
            FROM approval_requests
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

        let count_row = sqlx::query("SELECT COUNT(*) FROM approval_requests WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_one(&self.db)
            .await?;

        Ok((rows, count_row.get::<i64, _>(0)))
    }

    pub async fn update_approval_request_status(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        status: &str,
        reviewed_by: Option<Uuid>,
        decision_note: Option<&str>,
    ) -> Result<Option<ApprovalRequestRow>> {
        let row = sqlx::query_as::<_, ApprovalRequestRow>(
            r#"
            UPDATE approval_requests
            SET status = $1,
                reviewed_by = COALESCE($2, reviewed_by),
                reviewed_at = NOW(),
                decision_note = COALESCE($3, decision_note),
                updated_at = NOW()
            WHERE id = $4 AND tenant_id = $5
            RETURNING id, tenant_id, action_id, title, reason, status, requested_by,
                      reviewed_by, reviewed_at, decision_note, created_at, updated_at
            "#,
        )
        .bind(status)
        .bind(reviewed_by)
        .bind(decision_note)
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn list_policy_rules(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<PolicyRuleRow>, i64)> {
        let rows = sqlx::query_as::<_, PolicyRuleRow>(
            r#"
            SELECT id, tenant_id, name, rule_type, scope, settings, enabled, created_at, updated_at
            FROM policy_rules
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

        let count_row = sqlx::query("SELECT COUNT(*) FROM policy_rules WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_one(&self.db)
            .await?;

        Ok((rows, count_row.get::<i64, _>(0)))
    }

    pub async fn create_policy_rule(
        &self,
        id: Uuid,
        tenant_id: Uuid,
        name: &str,
        rule_type: &str,
        scope: serde_json::Value,
        settings: serde_json::Value,
        enabled: bool,
    ) -> Result<PolicyRuleRow> {
        let row = sqlx::query_as::<_, PolicyRuleRow>(
            r#"
            INSERT INTO policy_rules (id, tenant_id, name, rule_type, scope, settings, enabled)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, tenant_id, name, rule_type, scope, settings, enabled, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(name)
        .bind(rule_type)
        .bind(scope)
        .bind(settings)
        .bind(enabled)
        .fetch_one(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn list_experiments(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ExperimentRow>, i64)> {
        let rows = sqlx::query_as::<_, ExperimentRow>(
            r#"
            SELECT id, tenant_id, job_id, name, hypothesis, variant_a, variant_b, status,
                   winner, metric, created_at, updated_at
            FROM experiments
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

        let count_row = sqlx::query("SELECT COUNT(*) FROM experiments WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_one(&self.db)
            .await?;

        Ok((rows, count_row.get::<i64, _>(0)))
    }

    pub async fn create_experiment(
        &self,
        id: Uuid,
        tenant_id: Uuid,
        job_id: Option<Uuid>,
        name: &str,
        hypothesis: &str,
        variant_a: serde_json::Value,
        variant_b: serde_json::Value,
        status: &str,
    ) -> Result<ExperimentRow> {
        let row = sqlx::query_as::<_, ExperimentRow>(
            r#"
            INSERT INTO experiments (id, tenant_id, job_id, name, hypothesis, variant_a, variant_b, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, tenant_id, job_id, name, hypothesis, variant_a, variant_b, status, winner, metric, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(job_id)
        .bind(name)
        .bind(hypothesis)
        .bind(variant_a)
        .bind(variant_b)
        .bind(status)
        .fetch_one(&self.db)
        .await?;

        Ok(row)
    }
}
