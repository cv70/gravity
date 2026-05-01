use anyhow::Result;
use uuid::Uuid;

use crate::datasource::dbdao::schema::ChannelAccountRow;
use crate::datasource::dbdao::DBDao;

use super::schema::{
    ChannelAccount, ChannelAccountListResponse, CreateChannelAccountRequest,
    UpdateChannelAccountRequest,
};

pub struct ChannelRepository {
    db_dao: DBDao,
}

impl ChannelRepository {
    pub fn new(db_dao: DBDao) -> Self {
        Self { db_dao }
    }

    pub async fn create(
        &self,
        tenant_id: Uuid,
        req: &CreateChannelAccountRequest,
    ) -> Result<ChannelAccount> {
        let row = self
            .db_dao
            .create_channel_account(
                Uuid::new_v4(),
                tenant_id,
                &req.platform,
                &req.name,
                &req.credentials_encrypted,
                req.settings.clone(),
                req.status.as_deref().unwrap_or("connected"),
            )
            .await?;

        Ok(Self::to_account(row))
    }

    pub async fn list(
        &self,
        tenant_id: Uuid,
        page: i64,
        limit: i64,
    ) -> Result<ChannelAccountListResponse> {
        let offset = (page - 1) * limit;
        let (rows, total) = self
            .db_dao
            .list_channel_accounts(tenant_id, limit, offset)
            .await?;

        Ok(ChannelAccountListResponse {
            data: rows.into_iter().map(Self::to_account).collect(),
            total,
        })
    }

    pub async fn update(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        req: &UpdateChannelAccountRequest,
    ) -> Result<Option<ChannelAccount>> {
        let row = self
            .db_dao
            .update_channel_account(
                tenant_id,
                id,
                req.name.as_deref(),
                req.status.as_deref(),
                req.settings.clone(),
                req.last_sync_at,
            )
            .await?;

        Ok(row.map(Self::to_account))
    }

    pub async fn delete(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        self.db_dao.delete_channel_account(tenant_id, id).await
    }

    fn to_account(row: ChannelAccountRow) -> ChannelAccount {
        ChannelAccount {
            id: row.id,
            tenant_id: row.tenant_id,
            platform: row.platform,
            name: row.name,
            credentials_encrypted: row.credentials_encrypted,
            settings: row.settings,
            status: row.status,
            last_sync_at: row.last_sync_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
