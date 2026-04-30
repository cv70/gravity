use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;

use super::dao::DBDao;
use super::schema::ContactRow;

impl DBDao {
    pub async fn create_contact(
        &self,
        id: Uuid,
        tenant_id: Uuid,
        email: &str,
        name: &str,
        phone: Option<&str>,
        tags: Vec<String>,
        attributes: serde_json::Value,
    ) -> Result<ContactRow> {
        let row = sqlx::query_as::<_, ContactRow>(
            r#"
            INSERT INTO contacts (id, tenant_id, email, name, phone, tags, attributes, subscribed)
            VALUES ($1, $2, $3, $4, $5, $6, $7, true)
            RETURNING id, tenant_id, email, name, phone, tags, attributes, subscribed, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(email)
        .bind(name)
        .bind(phone)
        .bind(&tags)
        .bind(&attributes)
        .fetch_one(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn get_contact_by_id(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<ContactRow>> {
        let row = sqlx::query_as::<_, ContactRow>(
            "SELECT id, tenant_id, email, name, phone, tags, attributes, subscribed, created_at, updated_at FROM contacts WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn list_contacts(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
        search_pattern: Option<&str>,
    ) -> Result<(Vec<ContactRow>, i64)> {
        let rows = sqlx::query_as::<_, ContactRow>(
            r#"
            SELECT id, tenant_id, email, name, phone, tags, attributes, subscribed, created_at, updated_at
            FROM contacts
            WHERE tenant_id = $1
            AND ($2::text IS NULL OR name ILIKE $2 OR email ILIKE $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(tenant_id)
        .bind(search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let count_row = sqlx::query(
            r#"
            SELECT COUNT(*) FROM contacts
            WHERE tenant_id = $1
            AND ($2::text IS NULL OR name ILIKE $2 OR email ILIKE $2)
            "#,
        )
        .bind(tenant_id)
        .bind(search_pattern)
        .fetch_one(&self.db)
        .await?;

        let total: i64 = count_row.get(0);

        Ok((rows, total))
    }

    pub async fn update_contact(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        email: Option<&str>,
        name: Option<&str>,
        phone: Option<&str>,
        tags: Option<Vec<String>>,
        attributes: Option<serde_json::Value>,
        subscribed: Option<bool>,
    ) -> Result<Option<ContactRow>> {
        let row = sqlx::query_as::<_, ContactRow>(
            r#"
            UPDATE contacts
            SET email = COALESCE($1, email),
                name = COALESCE($2, name),
                phone = COALESCE($3, phone),
                tags = COALESCE($4, tags),
                attributes = COALESCE($5, attributes),
                subscribed = COALESCE($6, subscribed),
                updated_at = NOW()
            WHERE id = $7 AND tenant_id = $8
            RETURNING id, tenant_id, email, name, phone, tags, attributes, subscribed, created_at, updated_at
            "#,
        )
        .bind(email)
        .bind(name)
        .bind(phone)
        .bind(&tags)
        .bind(&attributes)
        .bind(subscribed)
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn delete_contact(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM contacts WHERE id = $1 AND tenant_id = $2")
            .bind(id)
            .bind(tenant_id)
            .execute(&self.db)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
