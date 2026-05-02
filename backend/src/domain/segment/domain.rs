use anyhow::Result;
use uuid::Uuid;

use crate::datasource::dbdao::schema::ContactRow;
use crate::datasource::dbdao::DBDao;

use super::schema::{
    CreateSegmentRequest, Segment, SegmentPreviewContact, SegmentPreviewResponse,
    UpdateSegmentRequest,
};

pub struct SegmentRepository {
    db_dao: DBDao,
}

impl SegmentRepository {
    pub fn new(db_dao: DBDao) -> Self {
        Self { db_dao }
    }

    pub async fn create(&self, tenant_id: Uuid, req: &CreateSegmentRequest) -> Result<Segment> {
        let row = self
            .db_dao
            .create_segment(
                Uuid::new_v4(),
                tenant_id,
                &req.name,
                req.definition.clone(),
                req.is_dynamic.unwrap_or(true),
                req.status.as_deref().unwrap_or("active"),
            )
            .await?;

        Ok(Self::to_segment(row))
    }

    pub async fn list(
        &self,
        tenant_id: Uuid,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<Segment>, i64)> {
        let offset = (page - 1) * limit;
        let (rows, total) = self.db_dao.list_segments(tenant_id, limit, offset).await?;
        Ok((rows.into_iter().map(Self::to_segment).collect(), total))
    }

    pub async fn get_by_id(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Segment>> {
        Ok(self
            .db_dao
            .get_segment_by_id(tenant_id, id)
            .await?
            .map(Self::to_segment))
    }

    pub async fn update(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        req: &UpdateSegmentRequest,
    ) -> Result<Option<Segment>> {
        Ok(self
            .db_dao
            .update_segment(
                tenant_id,
                id,
                req.name.as_deref(),
                req.definition.clone(),
                req.is_dynamic,
                req.status.as_deref(),
            )
            .await?
            .map(Self::to_segment))
    }

    pub async fn delete(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        self.db_dao.delete_segment(tenant_id, id).await
    }

    pub async fn preview(&self, tenant_id: Uuid, definition: &serde_json::Value) -> Result<SegmentPreviewResponse> {
        let (contacts, _) = self.db_dao.list_contacts(tenant_id, 1000, 0, None).await?;
        let matching: Vec<ContactRow> = contacts
            .into_iter()
            .filter(|contact| matches_segment(contact, definition))
            .collect();

        Ok(SegmentPreviewResponse {
            matching_count: matching.len() as i64,
            sample_contacts: matching
                .into_iter()
                .take(10)
                .map(|contact| SegmentPreviewContact {
                    id: contact.id,
                    email: contact.email,
                    name: contact.name,
                    lifecycle_stage: contact
                        .attributes
                        .get("lifecycle_stage")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    tags: contact.tags,
                })
                .collect(),
        })
    }

    fn to_segment(row: crate::datasource::dbdao::schema::SegmentRow) -> Segment {
        Segment {
            id: row.id,
            tenant_id: row.tenant_id,
            name: row.name,
            definition: row.definition,
            is_dynamic: row.is_dynamic,
            status: row.status,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

fn matches_segment(contact: &ContactRow, definition: &serde_json::Value) -> bool {
    let Some(obj) = definition.as_object() else {
        return true;
    };

    if let Some(search) = obj.get("search").and_then(|v| v.as_str()) {
        let needle = search.to_lowercase();
        let haystack = format!("{} {}", contact.name, contact.email).to_lowercase();
        if !haystack.contains(&needle) {
            return false;
        }
    }

    if let Some(stage) = obj.get("lifecycle_stage").and_then(|v| v.as_str()) {
        let current = contact
            .attributes
            .get("lifecycle_stage")
            .and_then(|v| v.as_str())
            .unwrap_or("new");
        if current != stage {
            return false;
        }
    }

    if let Some(subscribed) = obj.get("subscribed").and_then(|v| v.as_bool()) {
        if contact.subscribed != subscribed {
            return false;
        }
    }

    if let Some(tags_any) = obj.get("tags_any").and_then(|v| v.as_array()) {
        let tags: Vec<&str> = tags_any.iter().filter_map(|v| v.as_str()).collect();
        if !tags.is_empty() && !tags.iter().any(|tag| contact.tags.iter().any(|item| item == tag)) {
            return false;
        }
    }

    if let Some(tags_all) = obj.get("tags_all").and_then(|v| v.as_array()) {
        let tags: Vec<&str> = tags_all.iter().filter_map(|v| v.as_str()).collect();
        if !tags.iter().all(|tag| contact.tags.iter().any(|item| item == tag)) {
            return false;
        }
    }

    true
}
