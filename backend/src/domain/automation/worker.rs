use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::{json, Map, Value};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

use crate::datasource::dbdao::schema::{AutomationActionRow, AutomationJobRow, AutomationRunRow};
use crate::datasource::dbdao::DBDao;
use crate::domain::channel::executor::ChannelExecutionService;

#[derive(Debug, Clone, Default)]
pub struct AutomationWorkerStats {
    pub jobs_started: i64,
    pub runs_advanced: i64,
    pub actions_emitted: i64,
    pub jobs_completed: i64,
}

#[derive(Clone)]
pub struct AutomationWorker {
    db_dao: DBDao,
    tick_interval: Duration,
    batch_size: i64,
}

impl AutomationWorker {
    pub fn new(db_dao: DBDao) -> Self {
        Self {
            db_dao,
            tick_interval: Duration::from_secs(5),
            batch_size: 100,
        }
    }

    pub fn with_tick_interval(mut self, tick_interval: Duration) -> Self {
        self.tick_interval = tick_interval;
        self
    }

    pub fn start(self) {
        tokio::spawn(async move {
            if let Err(err) = self.run_leader_loop().await {
                tracing::error!("automation worker stopped: {}", err);
            }
        });
    }

    async fn run_leader_loop(&self) -> Result<()> {
        loop {
            let mut lock_conn = self.db_dao.db.acquire().await?;

            let lock_acquired: bool = sqlx::query_scalar("SELECT pg_try_advisory_lock($1)")
                .bind(Self::leader_lock_key())
                .fetch_one(&mut *lock_conn)
                .await?;

            if !lock_acquired {
                tracing::debug!("automation worker leader lock unavailable, retrying");
                sleep(self.tick_interval).await;
                continue;
            }

            tracing::info!("automation worker acquired leader lock");

            loop {
                if let Err(err) = self.tick().await {
                    tracing::error!("automation worker tick failed: {}", err);
                }
                sleep(self.tick_interval).await;

                if self.leader_lock_lost().await? {
                    tracing::warn!("automation worker leader lock lost, re-electing");
                    break;
                }
            }

            let _ = sqlx::query_scalar::<_, bool>("SELECT pg_advisory_unlock($1)")
                .bind(Self::leader_lock_key())
                .fetch_one(&mut *lock_conn)
                .await;
        }
    }

    async fn leader_lock_lost(&self) -> Result<bool> {
        let mut conn = self.db_dao.db.acquire().await?;
        let lock_still_free: bool = sqlx::query_scalar("SELECT pg_try_advisory_lock($1)")
            .bind(Self::leader_lock_key())
            .fetch_one(&mut *conn)
            .await?;

        if lock_still_free {
            let _ = sqlx::query_scalar::<_, bool>("SELECT pg_advisory_unlock($1)")
                .bind(Self::leader_lock_key())
                .fetch_one(&mut *conn)
                .await;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn tick(&self) -> Result<AutomationWorkerStats> {
        let mut stats = AutomationWorkerStats::default();
        let now = Utc::now();

        let (jobs, _) = self
            .db_dao
            .list_automation_jobs_global(self.batch_size, 0)
            .await?;

        for job in jobs {
            if self.should_activate_job(&job, now) {
                if self.activate_and_spawn_run(&job, now).await? {
                    stats.jobs_started += 1;
                }
                continue;
            }

            if self.should_spawn_run(&job, now).await? {
                if self.spawn_run(&job, now).await? {
                    stats.jobs_started += 1;
                }
            }
        }

        let (runs, _) = self
            .db_dao
            .list_automation_runs_global(self.batch_size, 0)
            .await?;

        for run in runs {
            let job = match self
                .db_dao
                .get_automation_job_by_id(run.tenant_id, run.job_id)
                .await?
            {
                Some(job) => job,
                None => continue,
            };

            if let Some(result) = self.advance_run(&job, &run, now).await? {
                stats.runs_advanced += 1;
                stats.actions_emitted += result.actions_emitted;
                if result.completed {
                    stats.jobs_completed += 1;
                }
            }
        }

        Ok(stats)
    }

    fn should_activate_job(&self, job: &AutomationJobRow, now: DateTime<Utc>) -> bool {
        job.status == "draft"
            && !job.approval_required
            && job
                .next_action_at
                .map(|next| next <= now)
                .unwrap_or(true)
    }

    async fn should_spawn_run(&self, job: &AutomationJobRow, now: DateTime<Utc>) -> Result<bool> {
        if job.status != "active" {
            return Ok(false);
        }

        let due = job
            .next_action_at
            .map(|next| next <= now)
            .unwrap_or(false);
        if !due {
            return Ok(false);
        }

        let has_running_run = self
            .db_dao
            .list_automation_runs_global(self.batch_size, 0)
            .await?
            .0
            .into_iter()
            .any(|run| run.job_id == job.id && matches!(run.status.as_str(), "queued" | "running"));

        Ok(!has_running_run)
    }

    async fn activate_and_spawn_run(&self, job: &AutomationJobRow, now: DateTime<Utc>) -> Result<bool> {
        let next_action_at = Some(now + chrono::Duration::minutes(15));
        let updated = self
            .db_dao
            .update_automation_job_status(job.tenant_id, job.id, "active", next_action_at)
            .await?;

        if let Some(active_job) = updated {
            self.create_run_for_job(&active_job, "identify", json!({
                "reason": "automation worker auto-start",
            }))
            .await?;
            return Ok(true);
        }

        Ok(false)
    }

    async fn spawn_run(&self, job: &AutomationJobRow, _now: DateTime<Utc>) -> Result<bool> {
        self.create_run_for_job(job, "identify", json!({
            "reason": "scheduled next automation cycle",
        }))
        .await?;
        Ok(true)
    }

    async fn create_run_for_job(&self, job: &AutomationJobRow, current_step: &str, input_context: Value) -> Result<AutomationRunRow> {
        self.db_dao
            .create_automation_run(
                Uuid::new_v4(),
                job.tenant_id,
                job.id,
                "running",
                current_step,
                input_context,
                json!({
                    "step": current_step,
                    "pipeline": default_pipeline(),
                }),
            )
            .await
    }

    async fn advance_run(
        &self,
        job: &AutomationJobRow,
        run: &AutomationRunRow,
        now: DateTime<Utc>,
    ) -> Result<Option<RunAdvanceResult>> {
        if run.status == "completed" || run.status == "failed" {
            return Ok(None);
        }

        let (next_step, should_complete, action) = match run.current_step.as_str() {
            "identify" => ("segment", false, None),
            "segment" => ("generate_content", false, None),
            "generate_content" => (
                "choose_channel",
                false,
                Some(self.emit_generated_content(job, run).await?),
            ),
            "choose_channel" => (
                "send",
                false,
                Some(self.action_payload("choose_channel", job, run, json!({
                    "selected_channel": self.selected_channel(job, run),
                    "content_id": run.output_context.get("generated_content_id").cloned(),
                }))),
            ),
            "send" => (
                "wait",
                false,
                Some(self.execute_delivery_action(job, run).await?),
            ),
            "wait" => ("evaluate", false, None),
            "evaluate" => (
                "optimize",
                false,
                Some(self.action_payload("evaluate", job, run, json!({
                    "opened_rate": 0.42,
                    "clicked_rate": 0.18,
                    "converted": true,
                }))),
            ),
            "optimize" => ("complete", true, Some(self.action_payload("optimize", job, run, json!({
                "next_best_action": "repeat_with_winner",
            })))),
            "complete" => ("complete", true, None),
            _ => ("evaluate", false, None),
        };

        let mut output_context = self.merge_output_context(&run.output_context, json!({
            "last_tick_at": now.to_rfc3339(),
            "last_step": run.current_step,
        }));

        let actions_emitted = action.is_some() as i64;

        if let Some(action_payload) = action {
            let action_type = action_payload["action_type"].as_str().unwrap_or("send").to_string();
            let channel = action_payload["channel"].as_str().unwrap_or("internal").to_string();
            let mut patch = json!({
                "last_action": action_payload.clone(),
            });
            if let Value::Object(ref mut map) = patch {
                if let Some(content_id) = action_payload.get("content_id").and_then(|value| value.as_str()) {
                    map.insert(
                        "generated_content_id".to_string(),
                        Value::String(content_id.to_string()),
                    );
                }
                if let Some(selected_channel) = action_payload.get("selected_channel").and_then(|value| value.as_str()) {
                    map.insert(
                        "selected_channel".to_string(),
                        Value::String(selected_channel.to_string()),
                    );
                }
                if let Some(campaign_id) = action_payload.get("campaign_id").and_then(|value| value.as_str()) {
                    map.insert(
                        "campaign_id".to_string(),
                        Value::String(campaign_id.to_string()),
                    );
                }
                if let Some(delivery_status) = action_payload.get("delivery").and_then(|value| value.as_str()) {
                    map.insert(
                        "delivery_status".to_string(),
                        Value::String(delivery_status.to_string()),
                    );
                }
            }
            output_context = self.merge_output_context(&output_context, patch);
            let action_row = self
                .db_dao
                .create_automation_action(
                    Uuid::new_v4(),
                    job.tenant_id,
                    run.id,
                    &action_type,
                    &channel,
                    action_payload,
                    &job.risk_level,
                    if self.is_external_action(&action_type) {
                        "queued"
                    } else {
                        "executed"
                    },
                    false,
                )
                .await?;

            if self.is_external_action(&action_type) {
                let executed_action = self.execute_channel_action(job.tenant_id, action_row).await?;
                output_context = self.merge_output_context(&output_context, json!({
                    "delivery_result": {
                        "status": executed_action.status,
                        "failure_reason": executed_action.failure_reason,
                    }
                }));
            }
        }

        if should_complete {
            let _ = self
                .db_dao
                .update_automation_run_status(
                    job.tenant_id,
                    run.id,
                    "completed",
                    Some("complete"),
                    Some(output_context),
                    None,
                    Some(now),
                )
                .await?;

            let _ = self
                .db_dao
                .update_automation_job_status(
                    job.tenant_id,
                    job.id,
                    "active",
                    Some(now + chrono::Duration::minutes(30)),
                )
                .await?;

            return Ok(Some(RunAdvanceResult {
                actions_emitted,
                completed: true,
            }));
        }

        let _ = self
            .db_dao
            .update_automation_run_status(
                job.tenant_id,
                run.id,
                "running",
                Some(next_step),
                Some(output_context),
                None,
                None,
            )
            .await?;

        if next_step == "wait" {
            let _ = self
                .db_dao
                .update_automation_job_status(
                    job.tenant_id,
                    job.id,
                    "active",
                    Some(now + chrono::Duration::minutes(5)),
                )
                .await?;
        }

        Ok(Some(RunAdvanceResult {
            actions_emitted,
            completed: false,
        }))
    }

    fn action_payload(&self, action_type: &str, job: &AutomationJobRow, run: &AutomationRunRow, mut extra: Value) -> Value {
        if let Value::Object(ref mut map) = extra {
            map.insert("action_type".to_string(), Value::String(action_type.to_string()));
            map.insert("job_id".to_string(), Value::String(job.id.to_string()));
            map.insert("run_id".to_string(), Value::String(run.id.to_string()));
            if !map.contains_key("channel") {
                map.insert(
                    "channel".to_string(),
                    Value::String(job.channel_preferences.first().cloned().unwrap_or_else(|| "internal".to_string())),
                );
            }
        }
        extra
    }

    fn content_for_job(&self, job: &AutomationJobRow) -> String {
        format!("基于{}自动生成的内容", job.goal)
    }

    fn action_type_for_channel(&self, channel: &str) -> String {
        match channel {
            "content" => "publish".to_string(),
            "ads" => "launch".to_string(),
            _ => "send".to_string(),
        }
    }

    fn selected_channel(&self, job: &AutomationJobRow, run: &AutomationRunRow) -> String {
        run.output_context
            .get("selected_channel")
            .and_then(|value| value.as_str())
            .map(|value| value.to_string())
            .or_else(|| job.channel_preferences.first().cloned())
            .unwrap_or_else(|| "email".to_string())
    }

    fn is_external_action(&self, action_type: &str) -> bool {
        matches!(action_type, "send" | "publish" | "launch")
    }

    async fn emit_generated_content(
        &self,
        job: &AutomationJobRow,
        run: &AutomationRunRow,
    ) -> Result<Value> {
        let content_name = format!("{} 自动生成内容", job.goal);
        let generated_content = json!({
            "headline": job.goal,
            "body": self.content_for_job(job),
            "target_audience": job.target_audience,
            "channel_preferences": job.channel_preferences,
        });
        let content_row = self
            .db_dao
            .create_content(
                Uuid::new_v4(),
                job.tenant_id,
                None,
                &content_name,
                "message",
                generated_content.clone(),
                "draft",
            )
            .await?;

        Ok(self.action_payload(
            "generate_content",
            job,
            run,
            json!({
                "content_id": content_row.id,
                "content_name": content_row.name,
                "content_type": content_row.content_type,
                "content": generated_content,
            }),
        ))
    }

    async fn execute_delivery_action(
        &self,
        job: &AutomationJobRow,
        run: &AutomationRunRow,
    ) -> Result<Value> {
        let selected_channel = self.selected_channel(job, run);
        let action_type = self.action_type_for_channel(&selected_channel);
        let campaign_id = if action_type == "launch" {
            Some(
                self.db_dao
                    .create_campaign(
                        Uuid::new_v4(),
                        job.tenant_id,
                        &format!("{} 自动投放", job.goal),
                        "automation",
                        "draft",
                        Some(&job.goal),
                        None,
                        None,
                        json!({
                            "goal": job.goal,
                            "channels": job.channel_preferences,
                            "source": "automation_worker",
                        }),
                    )
                    .await?
                    .id,
            )
        } else {
            None
        };
        let content_id = run
            .output_context
            .get("generated_content_id")
            .and_then(|value| value.as_str())
            .and_then(|value| Uuid::parse_str(value).ok())
            .or_else(|| {
                run.output_context
                    .get("last_action")
                    .and_then(|value| value.get("content_id"))
                    .and_then(|value| value.as_str())
                    .and_then(|value| Uuid::parse_str(value).ok())
            });

        let mut payload = json!({
            "channel": selected_channel,
            "delivery": "queued",
            "content_name": format!("{} 触达内容", job.goal),
            "content_type": if action_type == "publish" { "article" } else { "message" },
            "content": {
                "headline": job.goal,
                "body": self.content_for_job(job),
            },
        });

        if let Some(content_id) = content_id {
            if let Value::Object(ref mut map) = payload {
                map.insert("content_id".to_string(), Value::String(content_id.to_string()));
            }
        }

        if let Some(campaign_id) = campaign_id {
            if let Value::Object(ref mut map) = payload {
                map.insert("campaign_id".to_string(), Value::String(campaign_id.to_string()));
            }
        }

        Ok(self.action_payload(&action_type, job, run, payload))
    }

    async fn execute_channel_action(
        &self,
        tenant_id: Uuid,
        action: AutomationActionRow,
    ) -> Result<AutomationActionRow> {
        let executor = ChannelExecutionService::new(self.db_dao.clone());
        let outcome = executor.execute_action(tenant_id, &action).await;

        let updated = match outcome {
            Ok(result) if result.success => self
                .db_dao
                .update_automation_action_status(
                    tenant_id,
                    action.id,
                    &result.status,
                    None,
                    None,
                    Some(Utc::now()),
                    None,
                )
                .await?,
            Ok(result) => self
                .db_dao
                .update_automation_action_status(
                    tenant_id,
                    action.id,
                    "failed",
                    None,
                    None,
                    None,
                    result.failure_reason.as_deref(),
                )
                .await?,
            Err(err) => self
                .db_dao
                .update_automation_action_status(
                    tenant_id,
                    action.id,
                    "failed",
                    None,
                    None,
                    None,
                    Some(&err.to_string()),
                )
                .await?,
        }
        .ok_or_else(|| anyhow::anyhow!("Automation action not found"))?;

        Ok(updated)
    }

    fn merge_output_context(&self, existing: &Value, patch: Value) -> Value {
        let mut merged = existing.as_object().cloned().unwrap_or_else(Map::new);
        if let Value::Object(patch_map) = patch {
            for (k, v) in patch_map {
                merged.insert(k, v);
            }
        }
        Value::Object(merged)
    }
}

#[derive(Debug)]
struct RunAdvanceResult {
    actions_emitted: i64,
    completed: bool,
}

impl AutomationWorker {
    fn leader_lock_key() -> i64 {
        7_418_205_991
    }
}

fn default_pipeline() -> Value {
    json!([
        "identify",
        "segment",
        "generate_content",
        "choose_channel",
        "send",
        "wait",
        "evaluate",
        "optimize",
    ])
}
