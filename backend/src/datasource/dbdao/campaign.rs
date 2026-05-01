use anyhow::Result;
use chrono::NaiveDate;
use sqlx::Row;
use uuid::Uuid;

use super::dao::DBDao;
use super::schema::CampaignRow;

impl DBDao {
    pub async fn create_campaign(
        &self,
        id: Uuid,
        tenant_id: Uuid,
        name: &str,
        campaign_type: &str,
        status: &str,
        description: Option<&str>,
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
        settings: serde_json::Value,
    ) -> Result<CampaignRow> {
        let row = sqlx::query_as::<_, CampaignRow>(
            r#"
            INSERT INTO campaigns (id, tenant_id, name, campaign_type, status, description, start_date, end_date, settings)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, tenant_id, name, campaign_type, status, description, start_date, end_date, settings, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(name)
        .bind(campaign_type)
        .bind(status)
        .bind(description)
        .bind(start_date)
        .bind(end_date)
        .bind(&settings)
        .fetch_one(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn get_campaign_by_id(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Option<CampaignRow>> {
        let row = sqlx::query_as::<_, CampaignRow>(
            "SELECT id, tenant_id, name, campaign_type, status, description, start_date, end_date, settings, created_at, updated_at FROM campaigns WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn list_campaigns(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<CampaignRow>, i64)> {
        let rows = sqlx::query_as::<_, CampaignRow>(
            r#"
            SELECT id, tenant_id, name, campaign_type, status, description, start_date, end_date, settings, created_at, updated_at
            FROM campaigns
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

        let count_row = sqlx::query("SELECT COUNT(*) FROM campaigns WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_one(&self.db)
            .await?;

        let total: i64 = count_row.get(0);

        Ok((rows, total))
    }

    pub async fn update_campaign(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        name: Option<&str>,
        status: Option<&str>,
        description: Option<&str>,
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
    ) -> Result<Option<CampaignRow>> {
        let row = sqlx::query_as::<_, CampaignRow>(
            r#"
            UPDATE campaigns
            SET name = COALESCE($1, name),
                status = COALESCE($2, status),
                description = COALESCE($3, description),
                start_date = COALESCE($4, start_date),
                end_date = COALESCE($5, end_date),
                updated_at = NOW()
            WHERE id = $6 AND tenant_id = $7
            RETURNING id, tenant_id, name, campaign_type, status, description, start_date, end_date, settings, created_at, updated_at
            "#,
        )
        .bind(name)
        .bind(status)
        .bind(description)
        .bind(start_date)
        .bind(end_date)
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn delete_campaign(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM campaigns WHERE id = $1 AND tenant_id = $2")
            .bind(id)
            .bind(tenant_id)
            .execute(&self.db)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn count_campaigns_by_status(&self, tenant_id: Uuid, status: &str) -> Result<i64> {
        let row =
            sqlx::query("SELECT COUNT(*) FROM campaigns WHERE tenant_id = $1 AND status = $2")
                .bind(tenant_id)
                .bind(status)
                .fetch_one(&self.db)
                .await?;

        Ok(row.get(0))
    }
}
