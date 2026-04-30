use anyhow::Result;
use uuid::Uuid;

// =============================================================================
// ClickHouse Client Wrapper
// =============================================================================

#[derive(Debug, Clone)]
pub struct ClickHouseClient {
    url: String,
}

impl ClickHouseClient {
    pub fn new(host: &str, port: u16, database: &str) -> Self {
        Self {
            url: format!("http://{}:{}/?database={}", host, port, database),
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}

// =============================================================================
// Analytics Events DAO (ClickHouse)
// =============================================================================

#[derive(Debug, Clone)]
pub struct CkAnalyticsDao {
    client: ClickHouseClient,
}

impl CkAnalyticsDao {
    pub fn new(client: ClickHouseClient) -> Self {
        Self { client }
    }

    pub async fn insert_event(
        &self,
        event_id: Uuid,
        tenant_id: Uuid,
        contact_id: Option<Uuid>,
        event_type: &str,
        properties: serde_json::Value,
        occurred_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        // ClickHouse 事件表结构:
        // CREATE TABLE IF NOT EXISTS events (
        //     id UUID,
        //     tenant_id UUID,
        //     contact_id Nullable(UUID),
        //     event String,
        //     properties JSON,
        //     occurred_at DateTime64(3)
        // ) ENGINE = MergeTree()
        // ORDER BY (tenant_id, occurred_at);

        let client = reqwest::Client::new();
        let query = format!(
            r#"INSERT INTO events (id, tenant_id, contact_id, event, properties, occurred_at) VALUES ('{}', '{}', {}, '{}', '{}', '{}')"#,
            event_id,
            tenant_id,
            contact_id.map(|id| format!("'{}'", id)).unwrap_or_else(|| "NULL".to_string()),
            event_type,
            properties.to_string().replace('\'', "\\'"),
            occurred_at.format("%Y-%m-%d %H:%M:%S%.3f"),
        );

        let _ = client
            .post(self.client.url())
            .query(&[("query", query)])
            .send()
            .await?;

        Ok(())
    }

    pub async fn query_events(
        &self,
        tenant_id: Uuid,
        event_type: Option<&str>,
        start_time: Option<chrono::DateTime<chrono::Utc>>,
        end_time: Option<chrono::DateTime<chrono::Utc>>,
        limit: Option<u64>,
    ) -> Result<Vec<serde_json::Value>> {
        let client = reqwest::Client::new();

        let mut conditions = vec![format!("tenant_id = '{}'", tenant_id)];

        if let Some(e_type) = event_type {
            conditions.push(format!("event = '{}'", e_type));
        }

        if let Some(start) = start_time {
            conditions.push(format!("occurred_at >= '{}'", start.format("%Y-%m-%d %H:%M:%S%.3f")));
        }

        if let Some(end) = end_time {
            conditions.push(format!("occurred_at <= '{}'", end.format("%Y-%m-%d %H:%M:%S%.3f")));
        }

        let where_clause = conditions.join(" AND ");
        let limit_clause = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();

        let query = format!(
            "SELECT id, tenant_id, contact_id, event, properties, occurred_at FROM events WHERE {} {}",
            where_clause, limit_clause
        );

        let response = client
            .post(self.client.url())
            .query(&[("query", query)])
            .send()
            .await?;

        let text = response.text().await?;
        let json: serde_json::Value = serde_json::from_str(&text)
            .unwrap_or_else(|_| serde_json::json!({ "data": [], "rows": 0 }));

        let data = json.get("data")
            .and_then(|d| d.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(data)
    }

    pub async fn count_events_by_type(
        &self,
        tenant_id: Uuid,
        event_type: &str,
        start_time: Option<chrono::DateTime<chrono::Utc>>,
        end_time: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<i64> {
        let client = reqwest::Client::new();

        let mut conditions = vec![
            format!("tenant_id = '{}'", tenant_id),
            format!("event = '{}'", event_type),
        ];

        if let Some(start) = start_time {
            conditions.push(format!("occurred_at >= '{}'", start.format("%Y-%m-%d %H:%M:%S%.3f")));
        }

        if let Some(end) = end_time {
            conditions.push(format!("occurred_at <= '{}'", end.format("%Y-%m-%d %H:%M:%S%.3f")));
        }

        let where_clause = conditions.join(" AND ");
        let query = format!("SELECT count() FROM events WHERE {}", where_clause);

        let response = client
            .post(self.client.url())
            .query(&[("query", query)])
            .send()
            .await?;

        let text = response.text().await?;
        let json: serde_json::Value = serde_json::from_str(&text)
            .unwrap_or_else(|_| serde_json::json!({ "data": [[0]] }));

        let count = json.get("data")
            .and_then(|d| d.get(0))
            .and_then(|d| d.get(0))
            .and_then(|d| d.as_i64())
            .unwrap_or(0);

        Ok(count)
    }
}

// =============================================================================
// Funnel Analytics DAO (ClickHouse)
// =============================================================================

#[derive(Debug, Clone)]
pub struct CkFunnelDao {
    client: ClickHouseClient,
}

impl CkFunnelDao {
    pub fn new(client: ClickHouseClient) -> Self {
        Self { client }
    }

    pub async fn get_funnel_metrics(
        &self,
        tenant_id: Uuid,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<serde_json::Value> {
        let client = reqwest::Client::new();

        // 漏斗分析查询 - 计算每个步骤的用户数
        let query = format!(
            r#"
            WITH
                sent AS (
                    SELECT count(DISTINCT contact_id) as cnt
                    FROM events
                    WHERE tenant_id = '{}'
                    AND event = 'message.sent'
                    AND occurred_at BETWEEN '{}' AND '{}'
                ),
                opened AS (
                    SELECT count(DISTINCT contact_id) as cnt
                    FROM events
                    WHERE tenant_id = '{}'
                    AND event = 'message.opened'
                    AND occurred_at BETWEEN '{}' AND '{}'
                ),
                clicked AS (
                    SELECT count(DISTINCT contact_id) as cnt
                    FROM events
                    WHERE tenant_id = '{}'
                    AND event = 'link.clicked'
                    AND occurred_at BETWEEN '{}' AND '{}'
                ),
                converted AS (
                    SELECT count(DISTINCT contact_id) as cnt
                    FROM events
                    WHERE tenant_id = '{}'
                    AND event = 'conversion.recorded'
                    AND occurred_at BETWEEN '{}' AND '{}'
                )
            SELECT 'sent' as step, sent.cnt as count, 0.0 as dropoff_rate
            UNION ALL
            SELECT 'opened' as step, opened.cnt as count, if(sent.cnt > 0, 1.0 - opened.cnt / sent.cnt, 0.0) as dropoff_rate
            UNION ALL
            SELECT 'clicked' as step, clicked.cnt as count, if(opened.cnt > 0, 1.0 - clicked.cnt / opened.cnt, 0.0) as dropoff_rate
            UNION ALL
            SELECT 'converted' as step, converted.cnt as count, if(clicked.cnt > 0, 1.0 - converted.cnt / clicked.cnt, 0.0) as dropoff_rate
            "#,
            tenant_id, start_time.format("%Y-%m-%d %H:%M:%S%.3f"), end_time.format("%Y-%m-%d %H:%M:%S%.3f"),
            tenant_id, start_time.format("%Y-%m-%d %H:%M:%S%.3f"), end_time.format("%Y-%m-%d %H:%M:%S%.3f"),
            tenant_id, start_time.format("%Y-%m-%d %H:%M:%S%.3f"), end_time.format("%Y-%m-%d %H:%M:%S%.3f"),
            tenant_id, start_time.format("%Y-%m-%d %H:%M:%S%.3f"), end_time.format("%Y-%m-%d %H:%M:%S%.3f"),
        );

        let response = client
            .post(self.client.url())
            .query(&[("query", query)])
            .send()
            .await?;

        let text = response.text().await?;
        let json: serde_json::Value = serde_json::from_str(&text)
            .unwrap_or_else(|_| serde_json::json!({ "data": [] }));

        Ok(json)
    }
}
