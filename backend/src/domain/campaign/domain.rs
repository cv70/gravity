use anyhow::Result;
use uuid::Uuid;

use super::schema::{Campaign, CampaignListResponse, CreateCampaignRequest, UpdateCampaignRequest};
use crate::datasource::dbdao::DBDao;
use crate::datasource::dbdao::schema::CampaignRow;

pub struct CampaignRepository {
    db_dao: DBDao,
}

impl CampaignRepository {
    pub fn new(db_dao: DBDao) -> Self {
        Self { db_dao }
    }

    pub async fn create(&self, tenant_id: Uuid, req: &CreateCampaignRequest) -> Result<Campaign> {
        let id = Uuid::new_v4();
        let campaign = Campaign::new(tenant_id, req.name.clone(), req.campaign_type.clone());

        let row = self.db_dao.create_campaign(
            id,
            tenant_id,
            &campaign.name,
            &campaign.campaign_type,
            &campaign.status,
            req.description.as_deref(),
            req.start_date,
            req.end_date,
            campaign.settings.clone(),
        ).await?;

        Ok(Campaign {
            id: row.id,
            tenant_id: row.tenant_id,
            name: row.name,
            campaign_type: row.campaign_type,
            status: row.status,
            description: row.description,
            start_date: row.start_date,
            end_date: row.end_date,
            settings: row.settings,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    pub async fn get_by_id(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Campaign>> {
        let row = self.db_dao.get_campaign_by_id(tenant_id, id).await?;

        Ok(row.map(|r| Campaign {
            id: r.id,
            tenant_id: r.tenant_id,
            name: r.name,
            campaign_type: r.campaign_type,
            status: r.status,
            description: r.description,
            start_date: r.start_date,
            end_date: r.end_date,
            settings: r.settings,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    pub async fn list(&self, tenant_id: Uuid, page: i64, limit: i64) -> Result<(Vec<Campaign>, i64)> {
        let offset = (page - 1) * limit;

        let (rows, total) = self.db_dao.list_campaigns(tenant_id, limit, offset).await?;

        let campaigns: Vec<Campaign> = rows
            .into_iter()
            .map(|r| Campaign {
                id: r.id,
                tenant_id: r.tenant_id,
                name: r.name,
                campaign_type: r.campaign_type,
                status: r.status,
                description: r.description,
                start_date: r.start_date,
                end_date: r.end_date,
                settings: r.settings,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok((campaigns, total))
    }

    pub async fn update(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        req: &UpdateCampaignRequest,
    ) -> Result<Option<Campaign>> {
        let row = self.db_dao.update_campaign(
            tenant_id,
            id,
            req.name.as_deref(),
            req.status.as_ref().map(|s| s.to_string()).as_deref(),
            req.description.as_deref(),
            req.start_date,
            req.end_date,
        ).await?;

        Ok(row.map(|r| Campaign {
            id: r.id,
            tenant_id: r.tenant_id,
            name: r.name,
            campaign_type: r.campaign_type,
            status: r.status,
            description: r.description,
            start_date: r.start_date,
            end_date: r.end_date,
            settings: r.settings,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    pub async fn delete(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        self.db_dao.delete_campaign(tenant_id, id).await
    }

    pub async fn count_by_status(&self, tenant_id: Uuid) -> Result<(i64,)> {
        let count = self.db_dao.count_campaigns_by_status(tenant_id, "active").await?;
        Ok((count,))
    }
}
