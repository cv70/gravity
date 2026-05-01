use anyhow::Result;
use uuid::Uuid;

use super::schema::AnalyticsDashboard;
use crate::datasource::dbdao::DBDao;

pub struct AnalyticsRepository {
    db_dao: DBDao,
}

impl AnalyticsRepository {
    pub fn new(db_dao: DBDao) -> Self {
        Self { db_dao }
    }

    pub async fn get_dashboard(&self, tenant_id: Uuid) -> Result<AnalyticsDashboard> {
        let total_contacts = self.db_dao.count_contacts(tenant_id, true).await?;
        let active_campaigns = self
            .db_dao
            .count_campaigns_by_status(tenant_id, "active")
            .await?;
        let total_conversions = self.db_dao.count_conversions(tenant_id).await?;

        let conversion_rate = if total_contacts > 0 {
            total_conversions as f64 / total_contacts as f64
        } else {
            0.0
        };

        Ok(AnalyticsDashboard {
            total_contacts,
            active_campaigns,
            total_conversions,
            conversion_rate,
        })
    }

    pub async fn record_event(
        &self,
        tenant_id: Uuid,
        contact_id: Option<Uuid>,
        event: &str,
        properties: serde_json::Value,
    ) -> Result<()> {
        self.db_dao
            .record_event(Uuid::new_v4(), tenant_id, contact_id, event, properties)
            .await
    }

    pub async fn record_conversion(
        &self,
        id: Uuid,
        tenant_id: Uuid,
        contact_id: Uuid,
        goal_id: Option<&str>,
        value: Option<f64>,
        currency: &str,
        properties: serde_json::Value,
    ) -> Result<()> {
        self.db_dao
            .record_conversion(
                id, tenant_id, contact_id, goal_id, value, currency, properties,
            )
            .await
    }
}
