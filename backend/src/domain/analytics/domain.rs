use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use super::schema::AnalyticsDashboard;

pub struct AnalyticsRepository {
    pool: PgPool,
}

impl AnalyticsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_dashboard(&self, tenant_id: Uuid) -> Result<AnalyticsDashboard> {
        let total_contacts: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM contacts WHERE tenant_id = $1 AND subscribed = true",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let active_campaigns: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM campaigns WHERE tenant_id = $1 AND status = 'Active'",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let total_conversions: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM conversions WHERE tenant_id = $1",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let conversion_rate = if total_contacts.0 > 0 {
            total_conversions.0 as f64 / total_contacts.0 as f64
        } else {
            0.0
        };

        Ok(AnalyticsDashboard {
            total_contacts: total_contacts.0,
            active_campaigns: active_campaigns.0,
            total_conversions: total_conversions.0,
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
        sqlx::query(
            r#"
            INSERT INTO events (id, tenant_id, contact_id, event, properties, occurred_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(tenant_id)
        .bind(contact_id)
        .bind(event)
        .bind(&properties)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
