# 渠道集成设计

## 1. 设计目标

渠道层负责把 Gravity 的统一决策转成真实世界中的触达、回执和指标回流。它不是简单的 SDK 封装，而是系统执行闭环的重要一环。

渠道层必须做到：

- 统一接入多渠道
- 统一消息与事件模型
- 统一状态回写和指标采集
- 统一凭证管理和安全控制
- 统一失败处理和重试策略

## 2. 统一抽象

所有渠道都实现同一个 `ChannelAdapter` 接口。

```rust
#[async_trait]
pub trait ChannelAdapter: Send + Sync {
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthToken, ChannelError>;
    async fn send_message(&self, msg: &OutboundMessage) -> Result<MessageResult, ChannelError>;
    async fn receive_events(&self) -> Result<Vec<InboundEvent>, ChannelError>;
    async fn get_metrics(&self, range: &TimeRange) -> Result<ChannelMetrics, ChannelError>;
    async fn validate_connection(&self) -> Result<bool, ChannelError>;
}
```

### 2.1 关键对象

- `Credentials`：渠道凭证和授权信息
- `AuthToken`：临时访问令牌
- `OutboundMessage`：出站消息
- `InboundEvent`：入站事件和回执
- `MessageResult`：发送结果
- `ChannelMetrics`：渠道健康和转化指标

## 3. 支持范围

Gravity 的渠道层面向的是“运营闭环”，不是单纯发消息。

| 渠道 | 作用 |
|------|------|
| 邮件 | 欢迎、培育、通知、召回、复购 |
| 微信 / 企微 | 私域触达、标签同步、会话运营 |
| 短信 | 强提醒、催付、验证码、重要通知 |
| 内容平台 | 内容分发、引流、互动回收 |
| 广告平台 | 投放同步、转化回传、消耗对账 |
| 落地页 / 表单 | 转化承接、埋点、线索收集 |

## 4. 渠道注册表

通过注册表管理所有适配器，方便策略层和工作流层按名称调用。

```rust
pub struct ChannelRegistry {
    adapters: HashMap<String, Box<dyn ChannelAdapter>>,
}
```

## 5. 统一执行语义

### 5.1 出站

出站消息必须支持：

- 模板化发送
- 批量发送
- 频控与退订检查
- 失败重试和幂等去重
- 发送结果回传

### 5.2 入站

入站事件必须支持：

- 打开、点击、回复、订阅、退订、投诉
- 表单提交、加微、预约、下单、成交等业务事件
- Webhook 推送和主动拉取两种模式

### 5.3 指标

所有渠道都应统一回传：

- sent / delivered / opened / clicked
- bounced / complained / unsubscribed
- conversion / revenue / cost
- latency / retry_count / failure_reason

## 6. 安全与凭证

渠道凭证必须加密存储，且支持轮换和失效处理。

```rust
pub fn encrypt(plaintext: &str, key: &[u8; 32]) -> String { /* ... */ }
pub fn decrypt(encrypted: &str, key: &[u8; 32]) -> String { /* ... */ }
```

安全要求：

- 凭证只存加密值，不存明文
- 最小权限原则
- 支持密钥轮换
- 记录渠道连接与授权审计
- 任何日志都不能泄露 token、secret 或签名参数

## 7. 失败处理

- 连接失败：标记渠道不可用并告警
- 发送失败：按策略重试或切换备用渠道
- 回执延迟：允许补偿同步
- 渠道限流：自动降频或排队
- 配置失效：回退到安全默认策略

## 8. 设计约束

- 不允许各渠道各自定义一套业务语义
- 不允许渠道 SDK 直接侵入业务层
- 不允许绕过审计直接发送高风险内容
- 不允许凭证明文进入日志或前端
