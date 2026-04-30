use anyhow::Result;
use uuid::Uuid;

use super::schema::{Contact, ContactListResponse, CreateContactRequest, UpdateContactRequest};
use crate::datasource::dbdao::DBDao;

pub struct ContactRepository {
    db_dao: DBDao,
}

impl ContactRepository {
    pub fn new(db_dao: DBDao) -> Self {
        Self { db_dao }
    }

    pub async fn create(&self, tenant_id: Uuid, req: &CreateContactRequest) -> Result<Contact> {
        let id = Uuid::new_v4();
        let tags = req.tags.clone().unwrap_or_default();
        let attributes = req.attributes.clone().unwrap_or(serde_json::json!({}));

        let row = self.db_dao.create_contact(
            id,
            tenant_id,
            &req.email,
            &req.name,
            req.phone.as_deref(),
            tags,
            attributes,
        ).await?;

        Ok(Contact {
            id: row.id,
            tenant_id: row.tenant_id,
            email: row.email,
            name: row.name,
            phone: row.phone,
            tags: row.tags,
            attributes: row.attributes,
            subscribed: row.subscribed,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    pub async fn get_by_id(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Contact>> {
        let row = self.db_dao.get_contact_by_id(tenant_id, id).await?;

        Ok(row.map(|r| Contact {
            id: r.id,
            tenant_id: r.tenant_id,
            email: r.email,
            name: r.name,
            phone: r.phone,
            tags: r.tags,
            attributes: r.attributes,
            subscribed: r.subscribed,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    pub async fn list(
        &self,
        tenant_id: Uuid,
        page: i64,
        limit: i64,
        search: Option<&str>,
    ) -> Result<ContactListResponse> {
        let offset = (page - 1) * limit;

        let search_pattern = search.map(|s| {
            let escaped = s.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
            format!("%{}%", escaped)
        });

        let (rows, total) = self.db_dao.list_contacts(
            tenant_id,
            limit,
            offset,
            search_pattern.as_deref(),
        ).await?;

        let contacts: Vec<Contact> = rows
            .into_iter()
            .map(|r| Contact {
                id: r.id,
                tenant_id: r.tenant_id,
                email: r.email,
                name: r.name,
                phone: r.phone,
                tags: r.tags,
                attributes: r.attributes,
                subscribed: r.subscribed,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok(ContactListResponse {
            data: contacts,
            total,
            page,
            limit,
        })
    }

    pub async fn update(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        req: &UpdateContactRequest,
    ) -> Result<Option<Contact>> {
        let row = self.db_dao.update_contact(
            tenant_id,
            id,
            req.email.as_deref(),
            req.name.as_deref(),
            req.phone.as_deref(),
            req.tags.clone(),
            req.attributes.clone(),
            req.subscribed,
        ).await?;

        Ok(row.map(|r| Contact {
            id: r.id,
            tenant_id: r.tenant_id,
            email: r.email,
            name: r.name,
            phone: r.phone,
            tags: r.tags,
            attributes: r.attributes,
            subscribed: r.subscribed,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    pub async fn delete(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        self.db_dao.delete_contact(tenant_id, id).await
    }
}
