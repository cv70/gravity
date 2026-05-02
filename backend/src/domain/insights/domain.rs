use anyhow::Result;
use uuid::Uuid;

use crate::datasource::dbdao::DBDao;
use crate::domain::automation::domain::AutomationRepository;

use super::schema::InsightItem;

pub struct InsightsRepository {
    db_dao: DBDao,
}

impl InsightsRepository {
    pub fn new(db_dao: DBDao) -> Self {
        Self { db_dao }
    }

    pub async fn recommendations(&self, tenant_id: Uuid) -> Result<Vec<InsightItem>> {
        let automation_repo = AutomationRepository::new(self.db_dao.clone());
        let dashboard = automation_repo.dashboard(tenant_id, 5).await?;
        let mut items = vec![];

        if dashboard.overview.pending_approvals > 0 {
            items.push(InsightItem {
                title: "审批队列积压".to_string(),
                severity: "medium".to_string(),
                description: "存在待审批高风险动作，建议尽快处理以恢复自动化吞吐。".to_string(),
                evidence: serde_json::json!({"pending_approvals": dashboard.overview.pending_approvals}),
            });
        }
        if dashboard.overview.human_intervention_rate > 0.3 {
            items.push(InsightItem {
                title: "人工介入偏高".to_string(),
                severity: "high".to_string(),
                description: "风控和审批过于频繁，建议拆分策略或调低高风险阈值。".to_string(),
                evidence: serde_json::json!({"human_intervention_rate": dashboard.overview.human_intervention_rate}),
            });
        }
        if dashboard.overview.automation_coverage < 0.5 {
            items.push(InsightItem {
                title: "自动化覆盖不足".to_string(),
                severity: "medium".to_string(),
                description: "当前自动化覆盖率偏低，适合补充欢迎流、召回流和内容分发流。".to_string(),
                evidence: serde_json::json!({"automation_coverage": dashboard.overview.automation_coverage}),
            });
        }
        if items.is_empty() {
            items.push(InsightItem {
                title: "运营系统健康".to_string(),
                severity: "low".to_string(),
                description: "当前自动化、审批和策略配置处于相对健康状态。".to_string(),
                evidence: serde_json::json!({"active_jobs": dashboard.overview.active_jobs}),
            });
        }

        Ok(items)
    }

    pub async fn anomalies(&self, tenant_id: Uuid) -> Result<Vec<InsightItem>> {
        let automation_repo = AutomationRepository::new(self.db_dao.clone());
        let dashboard = automation_repo.dashboard(tenant_id, 5).await?;
        let mut items = vec![];

        if dashboard.overview.blocked_actions > 0 {
            items.push(InsightItem {
                title: "动作被风控拦截".to_string(),
                severity: "high".to_string(),
                description: "部分自动化动作被拦截，建议检查预算、频控和敏感词策略。".to_string(),
                evidence: serde_json::json!({"blocked_actions": dashboard.overview.blocked_actions}),
            });
        }

        if dashboard.overview.runs_in_progress == 0 && dashboard.overview.total_jobs > 0 {
            items.push(InsightItem {
                title: "任务未在执行".to_string(),
                severity: "low".to_string(),
                description: "存在自动化任务，但当前没有运行中的执行实例。".to_string(),
                evidence: serde_json::json!({"total_jobs": dashboard.overview.total_jobs}),
            });
        }

        Ok(items)
    }

    pub async fn opportunities(&self, tenant_id: Uuid) -> Result<Vec<InsightItem>> {
        let automation_repo = AutomationRepository::new(self.db_dao.clone());
        let dashboard = automation_repo.dashboard(tenant_id, 5).await?;
        let mut items = vec![];

        if dashboard.overview.total_jobs == 0 {
            items.push(InsightItem {
                title: "创建欢迎流".to_string(),
                severity: "medium".to_string(),
                description: "建议从新用户欢迎流开始，快速验证自动化闭环。".to_string(),
                evidence: serde_json::json!({}),
            });
        }

        if dashboard.recommendations.iter().any(|r| r.contains("A/B")) {
            items.push(InsightItem {
                title: "开启实验优化".to_string(),
                severity: "low".to_string(),
                description: "可以基于已有策略创建 A/B 实验，持续提升打开率和转化率。".to_string(),
                evidence: serde_json::json!({"recommendations": dashboard.recommendations}),
            });
        }

        Ok(items)
    }
}
