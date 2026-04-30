use anyhow::Result;
use uuid::Uuid;

use super::dao::DBDao;
use super::schema::OrganizationRow;

impl DBDao {
    pub async fn create_organization(
        &self,
        id: Uuid,
        name: &str,
        plan: &str,
        settings: serde_json::Value,
    ) -> Result<OrganizationRow> {
        let row = sqlx::query_as::<_, OrganizationRow>(
            "INSERT INTO organizations (id, name, plan, settings) VALUES ($1, $2, $3, $4) RETURNING id, name, plan, settings, created_at, updated_at",
        )
        .bind(id)
        .bind(name)
        .bind(plan)
        .bind(&settings)
        .fetch_one(&self.db)
        .await?;

        Ok(row)
    }

    pub async fn get_organization_by_name(&self, name: &str) -> Result<Option<OrganizationRow>> {
        let row = sqlx::query_as::<_, OrganizationRow>(
            "SELECT id, name, plan, settings, created_at, updated_at FROM organizations WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.db)
        .await?;

        Ok(row)
    }
}
