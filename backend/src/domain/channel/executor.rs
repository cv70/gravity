use anyhow::{anyhow, Result};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION};
use serde::Serialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::datasource::dbdao::schema::AutomationActionRow;
use crate::datasource::dbdao::DBDao;

#[derive(Debug, Clone, Serialize)]
pub struct ChannelExecutionResult {
    pub status: String,
    pub success: bool,
    pub failure_reason: Option<String>,
    pub details: Value,
}

#[derive(Clone)]
pub struct ChannelExecutionService {
    db_dao: DBDao,
    client: reqwest::Client,
}

impl ChannelExecutionService {
    pub fn new(db_dao: DBDao) -> Self {
        Self {
            db_dao,
            client: reqwest::Client::new(),
        }
    }

    pub async fn execute_action(
        &self,
        tenant_id: Uuid,
        action: &AutomationActionRow,
    ) -> Result<ChannelExecutionResult> {
        match action.action_type.as_str() {
            "send" => self.execute_send(tenant_id, action).await,
            "publish" => self.execute_publish(tenant_id, action).await,
            "launch" => self.execute_launch(tenant_id, action).await,
            _ => Ok(ChannelExecutionResult {
                status: "executed".to_string(),
                success: true,
                failure_reason: None,
                details: json!({
                    "mode": "internal",
                    "action_type": action.action_type,
                }),
            }),
        }
    }

    async fn execute_send(
        &self,
        tenant_id: Uuid,
        action: &AutomationActionRow,
    ) -> Result<ChannelExecutionResult> {
        let account = self
            .db_dao
            .get_channel_account_by_platform(tenant_id, &action.channel)
            .await?;

        let account = account.ok_or_else(|| {
            anyhow!(
                "No channel account configured for platform {}",
                action.channel
            )
        })?;

        let endpoint = channel_endpoint(&account.settings).ok_or_else(|| {
            anyhow!(
                "Channel account {} is missing endpoint settings",
                account.name
            )
        })?;

        let response = self
            .dispatch_webhook(
                &endpoint,
                &account.settings,
                json!({
                    "tenant_id": tenant_id,
                    "action_id": action.id,
                    "run_id": action.run_id,
                    "action_type": action.action_type,
                    "channel": action.channel,
                    "payload": action.payload,
                    "channel_account": {
                        "id": account.id,
                        "name": account.name,
                        "platform": account.platform,
                    },
                }),
            )
            .await?;

        Ok(ChannelExecutionResult {
            status: "executed".to_string(),
            success: true,
            failure_reason: None,
            details: json!({
                "platform": account.platform,
                "endpoint": endpoint,
                "http_status": response.status().as_u16(),
            }),
        })
    }

    async fn execute_publish(
        &self,
        tenant_id: Uuid,
        action: &AutomationActionRow,
    ) -> Result<ChannelExecutionResult> {
        let content_id = action
            .payload
            .get("content_id")
            .and_then(|value| value.as_str())
            .and_then(|value| Uuid::parse_str(value).ok());

        let content = if let Some(content_id) = content_id {
            let updated = self
                .db_dao
                .update_content_status(tenant_id, content_id, "published")
                .await?;

            updated.ok_or_else(|| anyhow!("Content {} not found", content_id))?
        } else {
            let name = action
                .payload
                .get("content_name")
                .and_then(|value| value.as_str())
                .unwrap_or("Automation content");
            let content_type = action
                .payload
                .get("content_type")
                .and_then(|value| value.as_str())
                .unwrap_or("article");
            let content_value = action
                .payload
                .get("content")
                .cloned()
                .unwrap_or_else(|| action.payload.clone());
            self.db_dao
                .create_content(
                    Uuid::new_v4(),
                    tenant_id,
                    action
                        .payload
                        .get("campaign_id")
                        .and_then(|value| value.as_str())
                        .and_then(|value| Uuid::parse_str(value).ok()),
                    name,
                    content_type,
                    content_value,
                    "published",
                )
                .await?
        };

        let maybe_account = self
            .db_dao
            .get_channel_account_by_platform(tenant_id, &action.channel)
            .await?;

        if let Some(account) = maybe_account {
            if let Some(endpoint) = channel_endpoint(&account.settings) {
                let _ = self
                    .dispatch_webhook(
                        &endpoint,
                        &account.settings,
                        json!({
                            "tenant_id": tenant_id,
                            "action_id": action.id,
                            "run_id": action.run_id,
                            "mode": "publish",
                            "content_id": content.id,
                            "content": content.content,
                            "channel_account": {
                                "id": account.id,
                                "name": account.name,
                                "platform": account.platform,
                            },
                        }),
                    )
                    .await?;
            }
        }

        Ok(ChannelExecutionResult {
            status: "executed".to_string(),
            success: true,
            failure_reason: None,
            details: json!({
                "content_id": content.id,
                "content_status": content.status,
            }),
        })
    }

    async fn execute_launch(
        &self,
        tenant_id: Uuid,
        action: &AutomationActionRow,
    ) -> Result<ChannelExecutionResult> {
        let campaign_id = action
            .payload
            .get("campaign_id")
            .and_then(|value| value.as_str())
            .and_then(|value| Uuid::parse_str(value).ok())
            .ok_or_else(|| anyhow!("launch actions require a campaign_id"))?;

        let campaign = self
            .db_dao
            .update_campaign(
                tenant_id,
                campaign_id,
                None,
                Some("active"),
                None,
                None,
                None,
            )
            .await?
            .ok_or_else(|| anyhow!("Campaign {} not found", campaign_id))?;

        if let Some(account) = self
            .db_dao
            .get_channel_account_by_platform(tenant_id, &action.channel)
            .await?
        {
            if let Some(endpoint) = channel_endpoint(&account.settings) {
                let _ = self
                    .dispatch_webhook(
                        &endpoint,
                        &account.settings,
                        json!({
                            "tenant_id": tenant_id,
                            "action_id": action.id,
                            "run_id": action.run_id,
                            "mode": "launch",
                            "campaign": {
                                "id": campaign.id,
                                "name": campaign.name,
                                "status": campaign.status,
                            },
                            "channel_account": {
                                "id": account.id,
                                "name": account.name,
                                "platform": account.platform,
                            },
                        }),
                    )
                    .await?;
            }
        }

        Ok(ChannelExecutionResult {
            status: "executed".to_string(),
            success: true,
            failure_reason: None,
            details: json!({
                "campaign_id": campaign.id,
                "campaign_status": campaign.status,
            }),
        })
    }

    async fn dispatch_webhook(
        &self,
        endpoint: &str,
        settings: &Value,
        payload: Value,
    ) -> Result<reqwest::Response> {
        let mut request = self.client.post(endpoint).json(&payload);
        request = request.headers(build_headers(settings)?);

        let response = request.send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Channel webhook {} failed: {} {}",
                endpoint,
                status,
                body
            ));
        }

        Ok(response)
    }
}

fn channel_endpoint(settings: &Value) -> Option<String> {
    settings
        .get("endpoint")
        .or_else(|| settings.get("webhook_url"))
        .or_else(|| settings.get("url"))
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
}

fn build_headers(settings: &Value) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();

    if let Some(token) = settings
        .get("bearer_token")
        .and_then(|value| value.as_str())
    {
        let value = HeaderValue::from_str(&format!("Bearer {}", token))?;
        headers.insert(AUTHORIZATION, value);
    } else if let Some(token) = settings.get("auth_token").and_then(|value| value.as_str()) {
        let value = HeaderValue::from_str(&format!("Bearer {}", token))?;
        headers.insert(AUTHORIZATION, value);
    }

    if let Some(api_key) = settings.get("api_key").and_then(|value| value.as_str()) {
        headers.insert(
            HeaderName::from_static("x-api-key"),
            HeaderValue::from_str(api_key)?,
        );
    }

    if let Some(extra_headers) = settings.get("headers").and_then(|value| value.as_object()) {
        for (key, value) in extra_headers {
            if let Some(value) = value.as_str() {
                headers.insert(
                    HeaderName::from_bytes(key.as_bytes())?,
                    HeaderValue::from_str(value)?,
                );
            }
        }
    }

    Ok(headers)
}
