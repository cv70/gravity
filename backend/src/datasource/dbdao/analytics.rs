use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;

use super::dao::DBDao;

impl DBDao {
    pub async fn count_contacts(&self, tenant_id: Uuid, subscribed_only: bool) -> Result<i64> {
        let query = if subscribed_only {
            "SELECT COUNT(*) FROM contacts WHERE tenant_id = $1 AND subscribed = true"
        } else {
            "SELECT COUNT(*) FROM contacts WHERE tenant_id = $1"
        };

        let row = sqlx::query(query)
            .bind(tenant_id)
            .fetch_one(&self.db)
            .await?;

        Ok(row.get(0))
    }

    pub async fn count_conversions(&self, tenant_id: Uuid) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) FROM conversions WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_one(&self.db)
            .await?;

        Ok(row.get(0))
    }

    pub async fn record_event(
        &self,
        id: Uuid,
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
        .bind(id)
        .bind(tenant_id)
        .bind(contact_id)
        .bind(event)
        .bind(&properties)
        .execute(&self.db)
        .await?;

        Ok(())
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
        sqlx::query(
            r#"
            INSERT INTO conversions (id, tenant_id, contact_id, goal_id, value, currency, properties)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(contact_id)
        .bind(goal_id)
        .bind(value)
        .bind(currency)
        .bind(&properties)
        .execute(&self.db)
        .await?;

        Ok(())
    }
}
