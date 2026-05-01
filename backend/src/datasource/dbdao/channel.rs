use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;

use super::dao::DBDao;
use super::schema::ChannelAccountRow;

impl DBDao {
    pub async fn create_channel_account(
        &self,
        id: Uuid,
        tenant_id: Uuid,
        platform: &str,
        name: &str,
        credentials_encrypted: &str,
        settings: serde_json::Value,
        status: &str,
    ) -> Result<ChannelAccountRow> {
        let row = sqlx::query_as::<_, ChannelAccountRow>(
            r#"
            INSERT INTO channel_accounts (
                id, tenant_id, platform, name, credentials_encrypted, settings, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, tenant_id, platform, name, credentials_encrypted, settings, status, last_sync_at, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(platform)
        .bind(name)
        .bind(credentials_encrypted)
        .bind(settings)
        .bind(status)
        .fetch_one(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn get_channel_account_by_platform(
        &self,
        tenant_id: Uuid,
        platform: &str,
    ) -> Result<Option<ChannelAccountRow>> {
        let row = sqlx::query_as::<_, ChannelAccountRow>(
            r#"
            SELECT id, tenant_id, platform, name, credentials_encrypted, settings, status, last_sync_at, created_at, updated_at
            FROM channel_accounts
            WHERE tenant_id = $1 AND platform = $2
            ORDER BY status DESC, updated_at DESC
            LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .bind(platform)
        .fetch_optional(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn list_channel_accounts(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ChannelAccountRow>, i64)> {
        let rows = sqlx::query_as::<_, ChannelAccountRow>(
            r#"
            SELECT id, tenant_id, platform, name, credentials_encrypted, settings, status, last_sync_at, created_at, updated_at
            FROM channel_accounts
            WHERE tenant_id = $1
            ORDER BY updated_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let count_row = sqlx::query("SELECT COUNT(*) FROM channel_accounts WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_one(&self.db)
            .await?;

        Ok((rows, count_row.get::<i64, _>(0)))
    }

    pub async fn update_channel_account(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        name: Option<&str>,
        status: Option<&str>,
        settings: Option<serde_json::Value>,
        last_sync_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Option<ChannelAccountRow>> {
        let row = sqlx::query_as::<_, ChannelAccountRow>(
            r#"
            UPDATE channel_accounts
            SET name = COALESCE($1, name),
                status = COALESCE($2, status),
                settings = COALESCE($3, settings),
                last_sync_at = COALESCE($4, last_sync_at),
                updated_at = NOW()
            WHERE id = $5 AND tenant_id = $6
            RETURNING id, tenant_id, platform, name, credentials_encrypted, settings, status, last_sync_at, created_at, updated_at
            "#,
        )
        .bind(name)
        .bind(status)
        .bind(settings)
        .bind(last_sync_at)
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn delete_channel_account(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM channel_accounts WHERE id = $1 AND tenant_id = $2")
            .bind(id)
            .bind(tenant_id)
            .execute(&self.db)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
