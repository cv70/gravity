use anyhow::Result;
use uuid::Uuid;

use super::dao::DBDao;
use super::schema::UserRow;

impl DBDao {
    pub async fn create_user(
        &self,
        id: Uuid,
        tenant_id: Uuid,
        email: &str,
        password_hash: &str,
        name: &str,
        role: &str,
    ) -> Result<UserRow> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            INSERT INTO users (id, tenant_id, email, password_hash, name, role)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, tenant_id, email, password_hash, name, role, last_login_at, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(email)
        .bind(password_hash)
        .bind(name)
        .bind(role)
        .fetch_one(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn get_user_by_email(&self, email: &str, tenant_id: Uuid) -> Result<Option<UserRow>> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, tenant_id, email, password_hash, name, role, last_login_at, created_at, updated_at FROM users WHERE email = $1 AND tenant_id = $2",
        )
        .bind(email)
        .bind(tenant_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn get_user_by_id(&self, user_id: Uuid, tenant_id: Uuid) -> Result<Option<UserRow>> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, tenant_id, email, password_hash, name, role, last_login_at, created_at, updated_at FROM users WHERE id = $1 AND tenant_id = $2",
        )
        .bind(user_id)
        .bind(tenant_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn update_last_login(&self, user_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = $1")
            .bind(user_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }
}
