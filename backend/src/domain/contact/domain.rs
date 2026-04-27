use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use super::schema::{Contact, CreateContactRequest, UpdateContactRequest};

pub struct ContactRepository {
    pool: PgPool,
}

impl ContactRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, tenant_id: Uuid, req: &CreateContactRequest) -> Result<Contact> {
        let contact = Contact::new(tenant_id, req.email.clone(), req.name.clone());

        let tags = req.tags.clone().unwrap_or_default();
        let attributes = req.attributes.clone().unwrap_or(serde_json::json!({}));

        let result = sqlx::query_as::<_, Contact>(
            r#"
            INSERT INTO contacts (id, tenant_id, email, name, phone, tags, attributes, subscribed)
            VALUES ($1, $2, $3, $4, $5, $6, $7, true)
            RETURNING *
            "#,
        )
        .bind(contact.id)
        .bind(contact.tenant_id)
        .bind(&contact.email)
        .bind(&contact.name)
        .bind(&contact.phone)
        .bind(&tags)
        .bind(&attributes)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_by_id(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Contact>> {
        let result = sqlx::query_as::<_, Contact>(
            "SELECT * FROM contacts WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn list(
        &self,
        tenant_id: Uuid,
        page: i64,
        limit: i64,
        search: Option<&str>,
    ) -> Result<(Vec<Contact>, i64)> {
        let offset = (page - 1) * limit;

        let search_pattern = search.map(|s| format!("%{}%", s));

        let contacts = sqlx::query_as::<_, Contact>(
            r#"
            SELECT * FROM contacts
            WHERE tenant_id = $1
            AND ($2::text IS NULL OR name ILIKE $2 OR email ILIKE $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(tenant_id)
        .bind(&search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM contacts
            WHERE tenant_id = $1
            AND ($2::text IS NULL OR name ILIKE $2 OR email ILIKE $2)
            "#,
        )
        .bind(tenant_id)
        .bind(&search_pattern)
        .fetch_one(&self.pool)
        .await?;

        Ok((contacts, total.0))
    }

    pub async fn update(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        req: &UpdateContactRequest,
    ) -> Result<Option<Contact>> {
        let existing = self.get_by_id(tenant_id, id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();

        let email = req.email.as_ref().unwrap_or(&existing.email);
        let name = req.name.as_ref().unwrap_or(&existing.name);
        let phone = req.phone.clone().or(existing.phone.clone());
        let tags = req.tags.clone().unwrap_or(existing.tags);
        let attributes = req.attributes.clone().unwrap_or(existing.attributes);
        let subscribed = req.subscribed.unwrap_or(existing.subscribed);

        let result = sqlx::query_as::<_, Contact>(
            r#"
            UPDATE contacts
            SET email = $1, name = $2, phone = $3, tags = $4, attributes = $5, subscribed = $6, updated_at = NOW()
            WHERE id = $7 AND tenant_id = $8
            RETURNING *
            "#,
        )
        .bind(email)
        .bind(name)
        .bind(&phone)
        .bind(&tags)
        .bind(&attributes)
        .bind(subscribed)
        .bind(id)
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Some(result))
    }

    pub async fn delete(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM contacts WHERE id = $1 AND tenant_id = $2")
            .bind(id)
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
