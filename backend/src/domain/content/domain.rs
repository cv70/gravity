use anyhow::Result;
use uuid::Uuid;

use super::schema::{Content, ContentListResponse, CreateContentRequest};
use crate::datasource::dbdao::DBDao;

pub struct ContentRepository {
    db_dao: DBDao,
}

impl ContentRepository {
    pub fn new(db_dao: DBDao) -> Self {
        Self { db_dao }
    }

    pub async fn create(&self, tenant_id: Uuid, req: &CreateContentRequest) -> Result<Content> {
        let id = Uuid::new_v4();
        let content = Content::new(tenant_id, req.name.clone(), req.content_type.clone(), req.content.clone(), req.campaign_id);

        let row = self.db_dao.create_content(
            id,
            tenant_id,
            req.campaign_id,
            &content.name,
            &content.content_type,
            content.content.clone(),
            &content.status,
        ).await?;

        Ok(Content {
            id: row.id,
            tenant_id: row.tenant_id,
            campaign_id: row.campaign_id,
            name: row.name,
            content_type: row.content_type,
            content: row.content,
            status: row.status,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    pub async fn list(&self, tenant_id: Uuid, page: i64, limit: i64) -> Result<ContentListResponse> {
        let offset = (page - 1) * limit;

        let (rows, total) = self.db_dao.list_contents(tenant_id, limit, offset).await?;

        let contents: Vec<Content> = rows
            .into_iter()
            .map(|r| Content {
                id: r.id,
                tenant_id: r.tenant_id,
                campaign_id: r.campaign_id,
                name: r.name,
                content_type: r.content_type,
                content: r.content,
                status: r.status,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok(ContentListResponse {
            data: contents,
            total,
        })
    }

    pub async fn get_by_id(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Content>> {
        let row = self.db_dao.get_content_by_id(tenant_id, id).await?;

        Ok(row.map(|r| Content {
            id: r.id,
            tenant_id: r.tenant_id,
            campaign_id: r.campaign_id,
            name: r.name,
            content_type: r.content_type,
            content: r.content,
            status: r.status,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    pub async fn delete(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        self.db_dao.delete_content(tenant_id, id).await
    }
}
