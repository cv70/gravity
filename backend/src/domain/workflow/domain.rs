use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use serde_json::json;
use uuid::Uuid;

use crate::datasource::dbdao::DBDao;

use super::schema::{
    CreateWorkflowRequest, TestWorkflowRequest, UpdateWorkflowRequest, Workflow,
    WorkflowExecution, WorkflowSchedule, WorkflowTestResponse,
};

pub struct WorkflowRepository {
    db_dao: DBDao,
}

impl WorkflowRepository {
    pub fn new(db_dao: DBDao) -> Self {
        Self { db_dao }
    }

    pub async fn create(&self, tenant_id: Uuid, req: &CreateWorkflowRequest) -> Result<Workflow> {
        let row = self
            .db_dao
            .create_workflow(
                Uuid::new_v4(),
                tenant_id,
                &req.name,
                req.description.as_deref(),
                req.version.unwrap_or(1),
                &req.trigger_type,
                req.trigger_config.clone(),
                req.steps.clone(),
                req.status.as_deref().unwrap_or("draft"),
            )
            .await?;

        Ok(Self::to_workflow(row))
    }

    pub async fn list(
        &self,
        tenant_id: Uuid,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<Workflow>, i64)> {
        let offset = (page - 1) * limit;
        let (rows, total) = self.db_dao.list_workflows(tenant_id, limit, offset).await?;
        Ok((rows.into_iter().map(Self::to_workflow).collect(), total))
    }

    pub async fn get_by_id(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Workflow>> {
        Ok(self.db_dao.get_workflow_by_id(tenant_id, id).await?.map(Self::to_workflow))
    }

    pub async fn update(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        req: &UpdateWorkflowRequest,
    ) -> Result<Option<Workflow>> {
        Ok(self
            .db_dao
            .update_workflow(
                tenant_id,
                id,
                req.name.as_deref(),
                req.description.as_deref(),
                req.version,
                req.trigger_type.as_deref(),
                req.trigger_config.clone(),
                req.steps.clone(),
                req.status.as_deref(),
            )
            .await?
            .map(Self::to_workflow))
    }

    pub async fn delete(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        self.db_dao.delete_workflow(tenant_id, id).await
    }

    pub async fn activate(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Workflow>> {
        self.update(
            tenant_id,
            id,
            &UpdateWorkflowRequest {
                name: None,
                description: None,
                version: None,
                trigger_type: None,
                trigger_config: None,
                steps: None,
                status: Some("active".to_string()),
            },
        )
        .await
    }

    pub async fn deactivate(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Workflow>> {
        self.update(
            tenant_id,
            id,
            &UpdateWorkflowRequest {
                name: None,
                description: None,
                version: None,
                trigger_type: None,
                trigger_config: None,
                steps: None,
                status: Some("inactive".to_string()),
            },
        )
        .await
    }

    pub async fn list_executions(
        &self,
        tenant_id: Uuid,
        workflow_id: Uuid,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<WorkflowExecution>, i64)> {
        let offset = (page - 1) * limit;
        let (rows, total) = self
            .db_dao
            .list_workflow_executions(tenant_id, Some(workflow_id), limit, offset)
            .await?;
        Ok((rows.into_iter().map(Self::to_execution).collect(), total))
    }

    pub async fn get_execution(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Option<WorkflowExecution>> {
        Ok(self
            .db_dao
            .get_workflow_execution_by_id(tenant_id, id)
            .await?
            .map(Self::to_execution))
    }

    pub async fn test(
        &self,
        tenant_id: Uuid,
        workflow_id: Uuid,
        req: &TestWorkflowRequest,
    ) -> Result<WorkflowTestResponse> {
        let workflow = self
            .get_by_id(tenant_id, workflow_id)
            .await?
            .ok_or_else(|| anyhow!("Workflow not found"))?;

        let mut status = "completed".to_string();
        let mut current_step_index = 0;
        let mut schedules = Vec::new();
        let mut output = json!({
            "workflow_id": workflow.id,
            "step_count": workflow.steps.as_array().map(|steps| steps.len()).unwrap_or(0),
        });
        let execution_id = Uuid::new_v4();

        if let Some(steps) = workflow.steps.as_array() {
            for (index, step) in steps.iter().enumerate() {
                current_step_index = index as i32;
                let step_type = step.get("type").and_then(|v| v.as_str()).unwrap_or("action");
                if step_type == "wait" {
                    let delay_hours = step
                        .get("config")
                        .and_then(|config| config.get("delay_hours"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(1);
                    let schedule = WorkflowSchedule {
                        id: Uuid::new_v4(),
                        tenant_id,
                        workflow_execution_id: execution_id,
                        resume_at: Utc::now() + Duration::hours(delay_hours),
                        payload: json!({
                            "contact_id": req.contact_id,
                            "step_index": index,
                            "reason": "workflow wait step",
                        }),
                        status: "pending".to_string(),
                        created_at: Utc::now(),
                    };
                    schedules.push(schedule);
                    status = "waiting".to_string();
                    output = json!({
                        "message": "Workflow paused on wait step",
                        "wait_hours": delay_hours,
                        "current_step_index": index,
                    });
                    break;
                }
                if step_type == "approval" {
                    status = "waiting_approval".to_string();
                    output = json!({
                        "message": "Workflow paused for approval",
                        "current_step_index": index,
                    });
                    break;
                }
            }
        }

        let execution = WorkflowExecution {
            id: execution_id,
            tenant_id,
            workflow_id,
            contact_id: req.contact_id,
            status,
            current_step_index,
            context: req.context.clone().unwrap_or_else(|| json!({})),
            started_at: Utc::now(),
            completed_at: if output.get("message").is_some() {
                None
            } else {
                Some(Utc::now())
            },
        };

        Ok(WorkflowTestResponse {
            execution,
            schedules,
            output,
        })
    }

    fn to_workflow(row: crate::datasource::dbdao::schema::WorkflowRow) -> Workflow {
        Workflow {
            id: row.id,
            tenant_id: row.tenant_id,
            name: row.name,
            description: row.description,
            version: row.version,
            trigger_type: row.trigger_type,
            trigger_config: row.trigger_config,
            steps: row.steps,
            status: row.status,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }

    fn to_execution(row: crate::datasource::dbdao::schema::WorkflowExecutionRow) -> WorkflowExecution {
        WorkflowExecution {
            id: row.id,
            tenant_id: row.tenant_id,
            workflow_id: row.workflow_id,
            contact_id: row.contact_id,
            status: row.status,
            current_step_index: row.current_step_index,
            context: row.context,
            started_at: row.started_at,
            completed_at: row.completed_at,
        }
    }
}
