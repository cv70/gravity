use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use super::schema::{Campaign, CampaignStatus, CreateCampaignRequest, UpdateCampaignRequest};

pub struct CampaignRepository {
    pool: PgPool,
}

impl CampaignRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, tenant_id: Uuid, req: &CreateCampaignRequest) -> Result<Campaign> {
        let campaign = Campaign::new(tenant_id, req.name.clone(), req.campaign_type.clone());

        let result = sqlx::query_as::<_, Campaign>(
            r#"
            INSERT INTO campaigns (id, tenant_id, name, campaign_type, status, description, start_date, end_date, settings)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(campaign.id)
        .bind(campaign.tenant_id)
        .bind(&campaign.name)
        .bind(&campaign.campaign_type)
        .bind(&campaign.status)
        .bind(&campaign.description)
        .bind(&campaign.start_date)
        .bind(&campaign.end_date)
        .bind(&campaign.settings)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_by_id(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Campaign>> {
        let result = sqlx::query_as::<_, Campaign>(
            "SELECT * FROM campaigns WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn list(&self, tenant_id: Uuid) -> Result<Vec<Campaign>> {
        let campaigns = sqlx::query_as::<_, Campaign>(
            "SELECT * FROM campaigns WHERE tenant_id = $1 ORDER BY created_at DESC",
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(campaigns)
    }

    pub async fn update(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        req: &UpdateCampaignRequest,
    ) -> Result<Option<Campaign>> {
        let existing = self.get_by_id(tenant_id, id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();

        let name = req.name.as_ref().unwrap_or(&existing.name);
        let status = req.status.as_ref().map(|s| s.to_string()).unwrap_or(existing.status);
        let description = req.description.clone().or(existing.description.clone());
        let start_date = req.start_date.or(existing.start_date);
        let end_date = req.end_date.or(existing.end_date);

        let result = sqlx::query_as::<_, Campaign>(
            r#"
            UPDATE campaigns
            SET name = $1, status = $2, description = $3, start_date = $4, end_date = $5, updated_at = NOW()
            WHERE id = $6 AND tenant_id = $7
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(&status)
        .bind(&description)
        .bind(start_date)
        .bind(end_date)
        .bind(id)
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Some(result))
    }

    pub async fn delete(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM campaigns WHERE id = $1 AND tenant_id = $2")
            .bind(id)
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn count_by_status(&self, tenant_id: Uuid) -> Result<(i64,)> {
        let result = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM campaigns WHERE tenant_id = $1 AND status = 'active'",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }
}
