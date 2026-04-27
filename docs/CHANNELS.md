# 渠道集成设计

## ChannelAdapter Trait

所有渠道适配器实现统一的 `ChannelAdapter` trait：

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub app_id: String,
    pub app_secret: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<ChronoDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: ChronoDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundMessage {
    pub contact_id: String,
    pub channel: String,
    pub msg_type: String, // text, image, article, template
    pub content: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResult {
    pub message_id: String,
    pub status: MessageStatus,
    pub sent_at: ChronoDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageStatus {
    Sent,
    Delivered,
    Failed { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundEvent {
    pub event_id: String,
    pub event_type: String,
    pub contact_id: String,
    pub data: Value,
    pub occurred_at: ChronoDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMetrics {
    pub sent: i64,
    pub delivered: i64,
    pub opened: i64,
    pub clicked: i64,
    pub bounced: i64,
    pub complained: i64,
}

#[async_trait]
pub trait ChannelAdapter: Send + Sync {
    /// OAuth 认证或 API Key 认证
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthToken, ChannelError>;

    /// 发送出站消息
    async fn send_message(&self, msg: &OutboundMessage) -> Result<MessageResult, ChannelError>;

    /// 拉取入站事件（Webhooks 推送的场景可为空实现）
    async fn receive_events(&self) -> Result<Vec<InboundEvent>, ChannelError>;

    /// 获取渠道指标
    async fn get_metrics(&self, range: &TimeRange) -> Result<ChannelMetrics, ChannelError>;

    /// 验证连接状态
    async fn validate_connection(&self) -> Result<bool, ChannelError>;
}
```

## 渠道注册表

通过注册表统一管理所有渠道：

```rust
pub struct ChannelRegistry {
    adapters: HashMap<String, Box<dyn ChannelAdapter>>,
}

impl ChannelRegistry {
    pub fn new() -> Self {
        let mut registry = Self { adapters: HashMap::new() };
        registry.register("email", Box::new(EmailAdapter::new()));
        registry.register("wechat", Box::new(WechatAdapter::new()));
        registry.register("xiaohongshu", Box::new(XiaohongshuAdapter::new()));
        registry.register("douyin", Box::new(DouyinAdapter::new()));
        registry.register("google_ads", Box::new(GoogleAdsAdapter::new()));
        registry
    }

    pub fn register(&mut self, platform: &str, adapter: Box<dyn ChannelAdapter>) {
        self.adapters.insert(platform.to_string(), adapter);
    }

    pub fn get(&self, platform: &str) -> Option<&dyn ChannelAdapter> {
        self.adapters.get(platform).map(|b| b.as_ref())
    }
}
```

## 邮件渠道 (SMTP)

```rust
pub struct EmailAdapter {
    smtp_config: SmtpConfig,
}

#[async_trait]
impl ChannelAdapter for EmailAdapter {
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthToken> {
        // SMTP 不需要 OAuth，使用连接测试代替
        Ok(AuthToken {
            access_token: "smtp".to_string(),
            refresh_token: None,
            expires_at: Utc::now() + Duration::days(365),
        })
    }

    async fn send_message(&self, msg: &OutboundMessage) -> Result<MessageResult> {
        let content = &msg.content;
        let email = Email::parse(content)?;

        let mut mailer = SmtpClient::new(&self.smtp_config)
            .await
            .map_err(|e| ChannelError::SendFailed(e.to_string()))?;

        mailer
            .send(&email)
            .await
            .map_err(|e| ChannelError::SendFailed(e.to_string()))?;

        Ok(MessageResult {
            message_id: ulid::Ulid::new().to_string(),
            status: MessageStatus::Sent,
            sent_at: Utc::now(),
        })
    }

    async fn validate_connection(&self) -> Result<bool> {
        // 连接测试
        Ok(SmtpClient::new(&self.smtp_config).await.is_ok())
    }
}
```

## 微信公众号

```rust
pub struct WechatAdapter {
    http_client: reqwest::Client,
    api_base: String,
}

#[async_trait]
impl ChannelAdapter for WechatAdapter {
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthToken> {
        // 获取 Access Token
        let url = format!(
            "{}/cgi-bin/token?grant_type=client_credential&appid={}&secret={}",
            self.api_base, credentials.app_id, credentials.app_secret
        );
        let resp: Value = self.http_client.get(&url).send().await?.json().await?;

        let access_token = resp["access_token"].as_str().unwrap();
        let expires_in = resp["expires_in"].as_i64().unwrap();

        Ok(AuthToken {
            access_token: access_token.to_string(),
            refresh_token: None,
            expires_at: Utc::now() + Duration::seconds(expires_in - 300),
        })
    }

    async fn send_message(&self, msg: &OutboundMessage) -> Result<MessageResult> {
        let token = self.get_valid_token().await?;
        let url = format!(
            "{}/cgi-bin/message/custom/send?access_token={}",
            self.api_base, token
        );

        let payload = json!({
            "touser": msg.contact_id,
            "msgtype": msg.msg_type,
            msg.msg_type: msg.content
        });

        let resp: Value = self.http_client.post(&url).json(&payload).send().await?.json().await?;

        if resp["errcode"].as_i64().unwrap_or(0) == 0 {
            Ok(MessageResult {
                message_id: resp["msgid"].as_str().unwrap().to_string(),
                status: MessageStatus::Sent,
                sent_at: Utc::now(),
            })
        } else {
            Err(ChannelError::SendFailed(resp["errmsg"].as_str().unwrap()))
        }
    }
}
```

## 凭证安全

渠道凭证使用 AES-256-GCM 加密存储：

```rust
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine};

pub fn encrypt(plaintext: &str, key: &[u8; 32]) -> String {
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let nonce = Aes256Gcm::generate_nonce(&mut rand::thread_rng());

    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .expect("encryption failed");

    // nonce || ciphertext (base64)
    let mut combined = nonce.to_vec();
    combined.extend(ciphertext);
    STANDARD.encode(combined)
}

pub fn decrypt(encrypted: &str, key: &[u8; 32]) -> String {
    let combined = STANDARD.decode(encrypted).unwrap();
    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .expect("decryption failed");

    String::from_utf8(plaintext).unwrap()
}
```

## Webhook 事件接收

各渠道通过 Webhook 接收入站事件：

```rust
// API 路由处理微信回调
async fn wechat_webhook(
    Path((platform,)): Path<(String,)>,
    TypedHeader(headers): TypedHeader<Headers>,
    body: String,
    signature: Query<WxSignature>,
) -> String {
    // 验证签名
    let channel = get_channel_account(&platform).await?;
    let token = decrypt(&channel.credentials_encrypted, &channel_encryption_key)
        .await?;

    let msg_signature = signature.msg_signature;
    let timestamp = signature.timestamp;
    let nonce = signature.nonce;

    if !verify_wechat_signature(&token, timestamp, nonce, &body, msg_signature) {
        return "签名验证失败".to_string();
    }

    // 处理消息
    let msg: WxMessage = serde_xml_rs::from_str(&body)?;
    let event = convert_to_inbound_event(msg)?;

    // 发布到 NATS
    nats::publish("channel.event", &event).await?;

    "success".to_string()
}
```
