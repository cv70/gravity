use anyhow::Result;
use chrono::{Duration, Utc};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::datasource::dbdao::schema::{
    ApprovalRequestRow, AutomationActionRow, AutomationJobRow, AutomationRunRow, ExperimentRow,
    PolicyRuleRow,
};
use crate::datasource::dbdao::DBDao;
use crate::domain::channel::executor::ChannelExecutionService;

use super::schema::{
    AutomationActionResponse, AutomationDashboardResponse, AutomationJobResponse,
    AutomationKpiResponse, AutomationRunResponse, ApprovalRequestResponse, CreateAutomationJobRequest,
    CreatePolicyRuleRequest, ExperimentResponse, PolicyRuleResponse, ReviewApprovalRequest,
};

struct DefaultJobTemplate {
    goal: &'static str,
    target_audience: serde_json::Value,
    channel_preferences: &'static [&'static str],
    strategy: serde_json::Value,
    status: &'static str,
    risk_level: &'static str,
    approval_required: bool,
    budget_limit: Option<f64>,
    currency: &'static str,
    next_action_hours: i64,
}

struct DefaultPolicyTemplate {
    name: &'static str,
    rule_type: &'static str,
    scope: serde_json::Value,
    settings: serde_json::Value,
}

pub struct AutomationRepository {
    db_dao: DBDao,
}

impl AutomationRepository {
    pub fn new(db_dao: DBDao) -> Self {
        Self { db_dao }
    }

    pub async fn bootstrap_defaults(
        &self,
        tenant_id: Uuid,
    ) -> Result<super::schema::AutomationBootstrapResponse> {
        let (_, job_count) = self.list_jobs(tenant_id, 1, 1).await?;
        let (_, policy_count) = self.list_policies(tenant_id, 1, 1).await?;
        let (_, experiment_count) = self.list_experiments(tenant_id, 1, 1).await?;

        let mut messages = Vec::new();
        let mut jobs_created = Vec::new();
        let mut policies_created = Vec::new();
        let mut experiments_created = Vec::new();

        if job_count == 0 {
            for template in default_job_templates() {
                let row = self
                    .db_dao
                    .create_automation_job(
                        Uuid::new_v4(),
                        tenant_id,
                        template.goal,
                        json!(template.target_audience),
                        template.channel_preferences.iter().map(|s| s.to_string()).collect(),
                        json!(template.strategy),
                        template.status,
                        template.risk_level,
                        template.approval_required,
                        template.budget_limit,
                        template.currency,
                        Some(Utc::now() + Duration::hours(template.next_action_hours)),
                    )
                    .await?;
                let job_response = Self::job_to_response(row.clone());

                if template.approval_required {
                    let action_type = self.action_type_for_channel(
                        template
                            .channel_preferences
                            .first()
                            .copied()
                            .unwrap_or("email"),
                    );
                    let run_row = self
                        .db_dao
                        .create_automation_run(
                            Uuid::new_v4(),
                            tenant_id,
                            row.id,
                            "waiting_approval",
                            "escalate",
                            json!({
                                "reason": "seeded high-risk automation job",
                                "workflow": template.strategy["workflow_blueprint"].clone(),
                            }),
                            json!({
                                "goal": template.goal,
                                "workflow": template.strategy["workflow_blueprint"].clone(),
                            }),
                        )
                        .await?;

                    let action_row = self
                        .db_dao
                        .create_automation_action(
                            Uuid::new_v4(),
                            tenant_id,
                            run_row.id,
                            &action_type,
                            template
                                .channel_preferences
                                .first()
                                .copied()
                                .unwrap_or("email"),
                            json!({
                                "goal": template.goal,
                                "workflow": template.strategy["workflow_blueprint"].clone(),
                                "mode": "seeded_approval",
                            }),
                            template.risk_level,
                            "pending_approval",
                            true,
                        )
                        .await?;

                    let _ = self
                        .db_dao
                        .create_approval_request(
                            Uuid::new_v4(),
                            tenant_id,
                            action_row.id,
                            "Seeded high-risk automation approval",
                            "This seeded job is intentionally waiting for a human to review the high-risk path.",
                            "pending",
                            None,
                        )
                        .await?;
                }

                jobs_created.push(job_response);
            }
            messages.push("默认自动化任务模板已创建".to_string());
        }

        if policy_count == 0 {
            for template in default_policy_templates() {
                let row = self
                    .db_dao
                    .create_policy_rule(
                        Uuid::new_v4(),
                        tenant_id,
                        template.name,
                        template.rule_type,
                        json!(template.scope),
                        json!(template.settings),
                        true,
                    )
                    .await?;
                policies_created.push(Self::policy_to_response(row));
            }
            messages.push("默认风控模板已创建".to_string());
        }

        if experiment_count == 0 {
            let seed_job_id = if let Some(job) = jobs_created.first() {
                Some(job.id)
            } else {
                self.list_jobs(tenant_id, 1, 1)
                    .await
                    .ok()
                    .and_then(|(jobs, _)| jobs.first().map(|job| job.id))
            };
            let row = self
                .db_dao
                .create_experiment(
                    Uuid::new_v4(),
                    tenant_id,
                    seed_job_id,
                    "欢迎流文案 A/B",
                    "通过比较简洁版与利益驱动版文案，找到更高打开率和点击率的组合。",
                    json!({
                        "headline": "欢迎加入 Gravity",
                        "body": "让系统自动帮你把线索变成结果。",
                    }),
                    json!({
                        "headline": "欢迎加入 Gravity，马上开始自动化增长",
                        "body": "用 AI 代理减少人工操作，提升转化效率。",
                    }),
                    "draft",
                )
                .await?;
            experiments_created.push(Self::experiment_to_response(row));
            messages.push("默认实验模板已创建".to_string());
        }

        if messages.is_empty() {
            messages.push("自动化默认模板已就绪，无需重复创建".to_string());
        }

        Ok(super::schema::AutomationBootstrapResponse {
            jobs_created,
            policies_created,
            experiments_created,
            messages,
        })
    }

    pub async fn create_job(
        &self,
        tenant_id: Uuid,
        req: &CreateAutomationJobRequest,
    ) -> Result<AutomationJobResponse> {
        let risk_level = self.assess_risk(req);
        let approval_required = req.approval_required.unwrap_or(risk_level == "high");
        let status = if approval_required { "waiting_approval" } else { "draft" };
        let currency = req.currency.clone().unwrap_or_else(|| "CNY".to_string());
        let next_action_at = Some(Utc::now() + Duration::hours(2));
        let strategy = self.build_strategy(req, &risk_level, approval_required);

        let row = self
            .db_dao
            .create_automation_job(
                Uuid::new_v4(),
                tenant_id,
                &req.goal,
                req.target_audience.clone(),
                req.channel_preferences.clone(),
                strategy,
                status,
                &risk_level,
                approval_required,
                req.budget_limit,
                &currency,
                next_action_at,
            )
            .await?;

        Ok(Self::job_to_response(row))
    }

    pub async fn list_jobs(
        &self,
        tenant_id: Uuid,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<AutomationJobResponse>, i64)> {
        let offset = (page - 1) * limit;
        let (rows, total) = self
            .db_dao
            .list_automation_jobs(tenant_id, limit, offset)
            .await?;

        Ok((rows.into_iter().map(Self::job_to_response).collect(), total))
    }

    pub async fn list_runs(
        &self,
        tenant_id: Uuid,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<AutomationRunResponse>, i64)> {
        let offset = (page - 1) * limit;
        let (rows, total) = self
            .db_dao
            .list_automation_runs(tenant_id, limit, offset)
            .await?;

        Ok((rows.into_iter().map(Self::run_to_response).collect(), total))
    }

    pub async fn list_actions(
        &self,
        tenant_id: Uuid,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<AutomationActionResponse>, i64)> {
        let offset = (page - 1) * limit;
        let (rows, total) = self
            .db_dao
            .list_automation_actions(tenant_id, limit, offset)
            .await?;

        Ok((rows.into_iter().map(Self::action_to_response).collect(), total))
    }

    pub async fn list_approvals(
        &self,
        tenant_id: Uuid,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<ApprovalRequestResponse>, i64)> {
        let offset = (page - 1) * limit;
        let (rows, total) = self
            .db_dao
            .list_approval_requests(tenant_id, limit, offset)
            .await?;

        Ok((rows.into_iter().map(Self::approval_to_response).collect(), total))
    }

    pub async fn list_policies(
        &self,
        tenant_id: Uuid,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<PolicyRuleResponse>, i64)> {
        let offset = (page - 1) * limit;
        let (rows, total) = self
            .db_dao
            .list_policy_rules(tenant_id, limit, offset)
            .await?;

        Ok((rows.into_iter().map(Self::policy_to_response).collect(), total))
    }

    pub async fn list_experiments(
        &self,
        tenant_id: Uuid,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<ExperimentResponse>, i64)> {
        let offset = (page - 1) * limit;
        let (rows, total) = self
            .db_dao
            .list_experiments(tenant_id, limit, offset)
            .await?;

        Ok((rows.into_iter().map(Self::experiment_to_response).collect(), total))
    }

    pub async fn execute_job(
        &self,
        tenant_id: Uuid,
        job_id: Uuid,
        note: Option<String>,
    ) -> Result<(AutomationRunResponse, Option<ApprovalRequestResponse>, Option<AutomationActionResponse>)> {
        let job = self
            .db_dao
            .get_automation_job_by_id(tenant_id, job_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Automation job not found"))?;

        let run_id = Uuid::new_v4();
        let action_id = Uuid::new_v4();
        let now = Utc::now();
        let goal = job.goal.clone();
        let channels = job.channel_preferences.clone();
        let risk_level = job.risk_level.clone();
        let output_context = json!({
            "note": note,
            "goal": goal.clone(),
            "current_status": job.status.clone(),
            "risk_level": risk_level.clone(),
        });

        let run_row = self
            .db_dao
            .create_automation_run(
                run_id,
                tenant_id,
                job_id,
                if job.approval_required { "waiting_approval" } else { "running" },
                if job.approval_required { "escalate" } else { "evaluate" },
                json!({
                    "job": job.clone(),
                    "note": note,
                }),
                output_context.clone(),
            )
            .await?;

        let mut run_status = if job.approval_required {
            "waiting_approval"
        } else {
            "running"
        };
        let mut last_error: Option<String> = None;
        let mut completed_at: Option<chrono::DateTime<Utc>> = None;

        let (approval_response, action_response) = if job.approval_required {
            let action_type = self.action_type_for_channel(channels.first().map(String::as_str).unwrap_or("email"));
            let campaign_id = self.launch_campaign_for_job(
                tenant_id,
                &goal,
                &channels,
                &action_type,
            )
            .await?;
            let action_row = self
                .db_dao
                .create_automation_action(
                    action_id,
                    tenant_id,
                    run_id,
                    &action_type,
                    channels.first().map(String::as_str).unwrap_or("email"),
                    json!({
                        "goal": goal.clone(),
                        "channels": channels.clone(),
                        "mode": "waiting_approval",
                        "campaign_id": campaign_id.map(|id| id.to_string()),
                    }),
                    &risk_level,
                    "pending_approval",
                    true,
                )
                .await?;

            let approval_row = self
                .db_dao
                .create_approval_request(
                    Uuid::new_v4(),
                    tenant_id,
                    action_row.id,
                    "High-risk automation approval",
                    "The job touches a high-risk action and requires human review before activation.",
                    "pending",
                    None,
                )
                .await?;

            (
                Some(Self::approval_to_response(approval_row)),
                Some(Self::action_to_response(action_row)),
            )
        } else {
            let action_type = self.action_type_for_channel(channels.first().map(String::as_str).unwrap_or("email"));
            let campaign_id = self.launch_campaign_for_job(
                tenant_id,
                &goal,
                &channels,
                &action_type,
            )
            .await?;
            let payload = json!({
                "goal": goal.clone(),
                "channels": channels.clone(),
                "mode": "auto_execute",
                "channel": channels.first().map(String::as_str).unwrap_or("email"),
                "content_name": format!("{} 触达内容", goal),
                "content_type": if action_type == "publish" { "article" } else { "message" },
                "campaign_id": campaign_id.map(|id| id.to_string()),
                "content": {
                    "headline": goal.clone(),
                    "body": format!("围绕{}自动生成的触达消息", goal),
                },
            });

            let action_row = self
                .db_dao
                .create_automation_action(
                    action_id,
                    tenant_id,
                    run_id,
                    &action_type,
                    channels.first().map(String::as_str).unwrap_or("email"),
                    payload,
                    &risk_level,
                    "queued",
                    false,
                )
                .await?;

            let executed_action = self
                .execute_action_with_channel_service(tenant_id, action_row)
                .await?;

            if executed_action.status == "failed" {
                run_status = "failed";
                last_error = executed_action.failure_reason.clone();
            } else {
                run_status = "completed";
            }
            completed_at = Some(Utc::now());

            (
                None,
                Some(Self::action_to_response(executed_action)),
            )
        };

        let _ = self
            .db_dao
            .update_automation_run_status(
                tenant_id,
                run_row.id,
                run_status,
                Some(if job.approval_required { "escalate" } else { "evaluate" }),
                Some(output_context),
                last_error.as_deref(),
                completed_at,
            )
            .await?;

        let _ = self
            .db_dao
            .update_automation_job_status(
                tenant_id,
                job_id,
                if job.approval_required { "waiting_approval" } else { "active" },
                Some(now + Duration::hours(4)),
            )
            .await?;

        Ok((Self::run_to_response(run_row), approval_response, action_response))
    }

    pub async fn review_approval(
        &self,
        tenant_id: Uuid,
        approval_id: Uuid,
        reviewer_id: Uuid,
        req: &ReviewApprovalRequest,
    ) -> Result<(ApprovalRequestResponse, Option<AutomationActionResponse>)> {
        let approval = self
            .db_dao
            .update_approval_request_status(
                tenant_id,
                approval_id,
                if req.approved { "approved" } else { "rejected" },
                Some(reviewer_id),
                req.note.as_deref(),
            )
            .await?
            .ok_or_else(|| anyhow::anyhow!("Approval request not found"))?;

        let action = self
            .db_dao
            .get_automation_action_by_id(tenant_id, approval.action_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Automation action not found"))?;

        let updated_action = if req.approved {
            let approved_action = self
                .db_dao
                .update_automation_action_status(
                    tenant_id,
                    approval.action_id,
                    "approved",
                    Some(reviewer_id),
                    Some(Utc::now()),
                    None,
                    None,
                )
                .await?
                .ok_or_else(|| anyhow::anyhow!("Automation action not found"))?;

            self.execute_action_with_channel_service(tenant_id, approved_action)
                .await?
        } else {
            self.db_dao
                .update_automation_action_status(
                    tenant_id,
                    approval.action_id,
                    "rejected",
                    Some(reviewer_id),
                    Some(Utc::now()),
                    None,
                    req.note.as_deref(),
                )
                .await?
                .ok_or_else(|| anyhow::anyhow!("Automation action not found"))?
        };

        if req.approved {
            if let Some(run) = self
                .db_dao
                .get_automation_run_by_id(tenant_id, action.run_id)
                .await?
            {
                let run_status = if updated_action.status == "failed" {
                    "failed"
                } else {
                    "completed"
                };
                let _ = self
                    .db_dao
                    .update_automation_run_status(
                        tenant_id,
                        run.id,
                        run_status,
                        Some("evaluate"),
                        Some(json!({
                            "approval": approval.status,
                            "last_action": updated_action.action_type,
                        })),
                        updated_action.failure_reason.as_deref(),
                        Some(Utc::now()),
                    )
                    .await?;

                if updated_action.status != "failed" {
                    let _ = self
                        .db_dao
                        .update_automation_job_status(tenant_id, run.job_id, "active", Some(Utc::now() + Duration::hours(4)))
                        .await?;
                }
            }
        }

        Ok((Self::approval_to_response(approval), Some(Self::action_to_response(updated_action))))
    }

    pub async fn create_policy_rule(
        &self,
        tenant_id: Uuid,
        req: &CreatePolicyRuleRequest,
    ) -> Result<PolicyRuleResponse> {
        let row = self
            .db_dao
            .create_policy_rule(
                Uuid::new_v4(),
                tenant_id,
                &req.name,
                &req.rule_type,
                req.scope.clone(),
                req.settings.clone(),
                req.enabled.unwrap_or(true),
            )
            .await?;

        Ok(Self::policy_to_response(row))
    }

    pub async fn dashboard(&self, tenant_id: Uuid, limit: i64) -> Result<AutomationDashboardResponse> {
        let (jobs, total_jobs) = self.list_jobs(tenant_id, 1, limit).await?;
        let (runs, _runs_total) = self.list_runs(tenant_id, 1, limit).await?;
        let (actions, actions_total) = self.list_actions(tenant_id, 1, limit).await?;
        let (approvals, _approvals_total) = self.list_approvals(tenant_id, 1, limit).await?;
        let (policies, policies_total) = self.list_policies(tenant_id, 1, limit).await?;
        let (experiments, experiments_total) = self.list_experiments(tenant_id, 1, limit).await?;

        let active_jobs = jobs.iter().filter(|job| job.status == "active").count() as i64;
        let runs_in_progress = runs.iter().filter(|run| run.status == "running").count() as i64;
        let pending_approvals = approvals.iter().filter(|approval| approval.status == "pending").count() as i64;
        let blocked_actions = actions
            .iter()
            .filter(|action| action.status == "pending_approval" || action.status == "rejected")
            .count() as i64;
        let experiments_running = experiments.iter().filter(|exp| exp.status == "running").count() as i64;

        let automation_coverage = if total_jobs > 0 {
            active_jobs as f64 / total_jobs as f64
        } else {
            0.0
        };
        let human_intervention_rate = if actions_total > 0 {
            blocked_actions as f64 / actions_total as f64
        } else {
            0.0
        };

        let recommendations = self.build_recommendations(
            active_jobs,
            pending_approvals,
            blocked_actions,
            policies_total,
            experiments_total,
        );

        Ok(AutomationDashboardResponse {
            overview: AutomationKpiResponse {
                total_jobs,
                active_jobs,
                runs_in_progress,
                pending_approvals,
                blocked_actions,
                enabled_policies: policies_total,
                experiments_running,
                automation_coverage,
                human_intervention_rate,
            },
            jobs,
            runs,
            actions,
            approvals,
            policies,
            experiments,
            recommendations,
        })
    }

    fn assess_risk(&self, req: &CreateAutomationJobRequest) -> String {
        if req.approval_required.unwrap_or(false) {
            return "high".into();
        }

        let channel_mix = req.channel_preferences.join(",");
        if req
            .budget_limit
            .map(|budget| budget >= 10_000.0)
            .unwrap_or(false)
            || channel_mix.contains("ads")
        {
            return "high".into();
        }

        if req.channel_preferences.len() > 1 {
            "medium".into()
        } else {
            "low".into()
        }
    }

    fn build_strategy(
        &self,
        req: &CreateAutomationJobRequest,
        risk_level: &str,
        approval_required: bool,
    ) -> serde_json::Value {
        json!({
            "goal": req.goal,
            "desired_outcome": req.desired_outcome,
            "agent_chain": ["planner", "generator", "selector", "evaluator"],
            "workflow_blueprint": workflow_blueprint(req.channel_preferences.as_slice()),
            "channel_mix": req.channel_preferences,
            "risk": {
                "level": risk_level,
                "approval_required": approval_required,
            },
            "optimization_axes": ["send_time", "segment_fit", "content_variant", "channel_mix"],
        })
    }

    fn build_recommendations(
        &self,
        active_jobs: i64,
        pending_approvals: i64,
        blocked_actions: i64,
        policy_count: i64,
        experiment_count: i64,
    ) -> Vec<String> {
        let mut items = vec![];
        if pending_approvals > 0 {
            items.push(format!("有 {} 个高风险动作等待审批，建议优先处理。", pending_approvals));
        }
        if blocked_actions > 0 {
            items.push(format!("已有 {} 个动作被风控拦截，可检查频控和预算阈值。", blocked_actions));
        }
        if active_jobs == 0 {
            items.push("当前没有活跃自动化任务，可从欢迎流或再激活流开始。".to_string());
        }
        if policy_count == 0 {
            items.push("建议先配置预算、频次和敏感内容政策，减少误触发。".to_string());
        }
        if experiment_count == 0 {
            items.push("建议开启 A/B 实验以验证内容与渠道的自动优化效果。".to_string());
        }

        if items.is_empty() {
            items.push("自动化闭环运行正常，可继续扩大渠道覆盖。".to_string());
        }

        items
    }

    fn action_type_for_channel(&self, channel: &str) -> String {
        match channel {
            "content" => "publish".to_string(),
            "ads" => "launch".to_string(),
            _ => "send".to_string(),
        }
    }

    async fn launch_campaign_for_job(
        &self,
        tenant_id: Uuid,
        goal: &str,
        channels: &[String],
        action_type: &str,
    ) -> Result<Option<Uuid>> {
        if action_type != "launch" {
            return Ok(None);
        }

        let campaign = self
            .db_dao
            .create_campaign(
                Uuid::new_v4(),
                tenant_id,
                &format!("{} 自动投放", goal),
                "automation",
                "draft",
                Some(goal),
                None,
                None,
                json!({
                    "goal": goal,
                    "channels": channels,
                    "source": "automation_job",
                }),
            )
            .await?;

        Ok(Some(campaign.id))
    }

    async fn execute_action_with_channel_service(
        &self,
        tenant_id: Uuid,
        action: AutomationActionRow,
    ) -> Result<AutomationActionRow> {
        let executor = ChannelExecutionService::new(self.db_dao.clone());
        match executor.execute_action(tenant_id, &action).await {
            Ok(result) if result.success => {
                let updated = self
                    .db_dao
                    .update_automation_action_status(
                        tenant_id,
                        action.id,
                        result.status.as_str(),
                        action.approved_by,
                        action.approved_at,
                        Some(Utc::now()),
                        None,
                    )
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("Automation action not found"))?;
                Ok(updated)
            }
            Ok(result) => {
                let updated = self
                    .db_dao
                    .update_automation_action_status(
                        tenant_id,
                        action.id,
                        "failed",
                        action.approved_by,
                        action.approved_at,
                        None,
                        result.failure_reason.as_deref(),
                    )
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("Automation action not found"))?;
                Ok(updated)
            }
            Err(err) => {
                let updated = self
                    .db_dao
                    .update_automation_action_status(
                        tenant_id,
                        action.id,
                        "failed",
                        action.approved_by,
                        action.approved_at,
                        None,
                        Some(&err.to_string()),
                    )
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("Automation action not found"))?;
                Ok(updated)
            }
        }
    }

    fn job_to_response(row: AutomationJobRow) -> AutomationJobResponse {
        AutomationJobResponse {
            id: row.id,
            tenant_id: row.tenant_id,
            goal: row.goal,
            target_audience: row.target_audience,
            channel_preferences: row.channel_preferences,
            strategy: row.strategy,
            status: row.status,
            risk_level: row.risk_level,
            approval_required: row.approval_required,
            budget_limit: row.budget_limit,
            currency: row.currency,
            next_action_at: row.next_action_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }

    fn run_to_response(row: AutomationRunRow) -> AutomationRunResponse {
        AutomationRunResponse {
            id: row.id,
            tenant_id: row.tenant_id,
            job_id: row.job_id,
            status: row.status,
            current_step: row.current_step,
            input_context: row.input_context,
            output_context: row.output_context,
            started_at: row.started_at,
            completed_at: row.completed_at,
            last_error: row.last_error,
        }
    }

    fn action_to_response(row: AutomationActionRow) -> AutomationActionResponse {
        AutomationActionResponse {
            id: row.id,
            tenant_id: row.tenant_id,
            run_id: row.run_id,
            action_type: row.action_type,
            channel: row.channel,
            payload: row.payload,
            risk_level: row.risk_level,
            status: row.status,
            requires_approval: row.requires_approval,
            approved_by: row.approved_by,
            approved_at: row.approved_at,
            executed_at: row.executed_at,
            failure_reason: row.failure_reason,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }

    fn approval_to_response(row: ApprovalRequestRow) -> ApprovalRequestResponse {
        ApprovalRequestResponse {
            id: row.id,
            tenant_id: row.tenant_id,
            action_id: row.action_id,
            title: row.title,
            reason: row.reason,
            status: row.status,
            requested_by: row.requested_by,
            reviewed_by: row.reviewed_by,
            reviewed_at: row.reviewed_at,
            decision_note: row.decision_note,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }

    fn policy_to_response(row: PolicyRuleRow) -> PolicyRuleResponse {
        PolicyRuleResponse {
            id: row.id,
            tenant_id: row.tenant_id,
            name: row.name,
            rule_type: row.rule_type,
            scope: row.scope,
            settings: row.settings,
            enabled: row.enabled,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }

    fn experiment_to_response(row: ExperimentRow) -> ExperimentResponse {
        ExperimentResponse {
            id: row.id,
            tenant_id: row.tenant_id,
            job_id: row.job_id,
            name: row.name,
            hypothesis: row.hypothesis,
            variant_a: row.variant_a,
            variant_b: row.variant_b,
            status: row.status,
            winner: row.winner,
            metric: row.metric,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

fn default_job_templates() -> Vec<DefaultJobTemplate> {
    vec![
        DefaultJobTemplate {
            goal: "新用户欢迎流",
            target_audience: json!({
                "lifecycle": ["new"],
                "persona": "signup"
            }),
            channel_preferences: &["email", "wechat"],
            strategy: json!({
                "template_id": "welcome",
                "agent_chain": ["planner", "generator", "selector", "evaluator"],
                "workflow_blueprint": ["identify", "segment", "generate_content", "choose_channel", "send", "wait", "evaluate", "optimize"],
                "message": "欢迎新用户进入自动化欢迎流",
            }),
            status: "draft",
            risk_level: "low",
            approval_required: false,
            budget_limit: Some(0.0),
            currency: "CNY",
            next_action_hours: 1,
        },
        DefaultJobTemplate {
            goal: "沉默用户再激活流",
            target_audience: json!({
                "lifecycle": ["churn_risk"],
                "inactive_days": 30
            }),
            channel_preferences: &["email", "content"],
            strategy: json!({
                "template_id": "reactivation",
                "agent_chain": ["planner", "generator", "selector", "evaluator"],
                "workflow_blueprint": ["identify", "segment", "generate_content", "choose_channel", "send", "wait", "evaluate", "optimize"],
                "message": "自动识别沉默用户并投放再激活内容",
            }),
            status: "draft",
            risk_level: "medium",
            approval_required: false,
            budget_limit: Some(0.0),
            currency: "CNY",
            next_action_hours: 2,
        },
        DefaultJobTemplate {
            goal: "高意向线索升级流",
            target_audience: json!({
                "lifecycle": ["warm", "hot"],
                "lead_score_min": 80
            }),
            channel_preferences: &["email", "wechat", "ads"],
            strategy: json!({
                "template_id": "high_intent_upgrade",
                "agent_chain": ["planner", "generator", "selector", "evaluator"],
                "workflow_blueprint": ["identify", "segment", "generate_content", "choose_channel", "send", "wait", "evaluate", "optimize", "escalate"],
                "message": "对高意向线索执行多渠道升级触达",
            }),
            status: "waiting_approval",
            risk_level: "high",
            approval_required: true,
            budget_limit: Some(5000.0),
            currency: "CNY",
            next_action_hours: 4,
        },
    ]
}

fn default_policy_templates() -> Vec<DefaultPolicyTemplate> {
    vec![
        DefaultPolicyTemplate {
            name: "发送频次限制",
            rule_type: "frequency_guard",
            scope: json!({"channels": ["email", "wechat", "content"]}),
            settings: json!({
                "per_contact_per_day": 1,
                "per_channel_per_day": 3,
                "cooldown_hours": 24,
            }),
        },
        DefaultPolicyTemplate {
            name: "预算阈值控制",
            rule_type: "budget_guard",
            scope: json!({"channels": ["ads", "email", "wechat"]}),
            settings: json!({
                "daily_budget_limit": 5000.0,
                "single_action_budget_limit": 1000.0,
                "require_approval_over": 3000.0,
                "currency": "CNY",
            }),
        },
        DefaultPolicyTemplate {
            name: "敏感内容检查",
            rule_type: "content_guard",
            scope: json!({"content_types": ["email", "article", "social_post"]}),
            settings: json!({
                "blocked_keywords": ["夸大", "保证收益", "违规", "绝对有效"],
                "brand_keywords": ["Gravity"],
                "allowlist_domains": ["gravity.example.com"],
            }),
        },
        DefaultPolicyTemplate {
            name: "异常自动降级",
            rule_type: "anomaly_guard",
            scope: json!({"signals": ["complaint", "conversion_drop", "delivery_failure"]}),
            settings: json!({
                "complaint_rate_threshold": 0.02,
                "conversion_drop_threshold": 0.30,
                "auto_pause": true,
            }),
        },
    ]
}

fn workflow_blueprint(channel_preferences: &[String]) -> Value {
    let mut steps = vec![
        "identify",
        "segment",
        "generate_content",
        "choose_channel",
        "send",
        "wait",
        "evaluate",
        "optimize",
    ];

    if channel_preferences.iter().any(|c| c == "ads") {
        steps.push("budget_review");
    }

    steps.push("escalate");

    json!(steps)
}
