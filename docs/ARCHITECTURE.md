# Gravity 架构设计

## 1. 高层架构

采用**模块化单体 (Modular Monolith)** 模式：单一 Rust 部署单元，内部按领域边界划分为 crate，通过事件总线解耦。

```
┌─────────────────────────────────────────────────────┐
│                    Gravity Platform                   │
│                                                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐           │
│  │ React SPA│  │  Admin   │  │  Public  │           │
│  │(Dashboard)│  │  Portal  │  │  Pages   │           │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘           │
│       │              │             │                   │
│       └──────────────┼─────────────┘                   │
│                      │ REST                             │
│  ┌───────────────────▼────────────────────────────┐   │
│  │            API Gateway (Axum)                    │   │
│  │     Auth · Tenant · Rate Limit · Routing        │   │
│  └───────────────────┬────────────────────────────┘   │
│                      │                                 │
│  ┌───────┬───────┬───┴───┬────────┬──────────┐       │
│  │Contact│Content│Channel│Workflow│Analytics  │       │
│  │ Crate │ Crate │ Crate │ Crate │ Crate     │       │
│  └───┬───┴───┬───┴───┬───┴───┬────┴─────┬────┘       │
│      │       │       │       │          │             │
│  ┌───▼───────▼───────▼───────▼──────────▼─────┐      │
│  │           Event Bus (NATS)                   │      │
│  └───┬───────────┬───────────┬────────────────┘      │
│      │           │           │                        │
│  ┌───▼──┐  ┌─────▼────┐  ┌──▼──────────┐            │
│  │PostgreSQL│  │  Redis   │  │ ClickHouse  │            │
│  │(OLTP)   │  │ (Cache)  │  │  (OLAP)     │            │
│  └────────┘  └──────────┘  └─────────────┘            │
└─────────────────────────────────────────────────────┘
```

## 2. Rust Crate 职责

| Crate | 职责 |
|-------|------|
| `gravity-api` | HTTP 服务入口，路由，中间件（认证、租户、限流） |
| `gravity-core` | 纯 Rust 领域模型，业务规则，无 IO 依赖 |
| `gravity-db` | SQLx Repository 实现，数据库迁移 |
| `gravity-channels` | ChannelAdapter trait，第三方渠道集成 |
| `gravity-workflow` | 状态机引擎，触发器，调度器 |
| `gravity-analytics` | ClickHouse 事件摄入，漏斗分析，归因 |
| `gravity-common` | ULID、配置、加密等共享工具 |

## 3. 领域模型

```
Organization ──1:N──> Team ──1:N──> User
     │
     ├──1:N──> Campaign (营销活动)
     │              ├── type: social | email | content | ads
     │              ├──1:N──> Workflow
     │              └──1:N──> Content
     │
     ├──1:N──> Contact (联系人)
     │              ├──1:N──> Event
     │              └──1:N──> Conversion
     │
     └──1:N──> ChannelAccount
```

## 4. 数据库

| 数据库 | 用途 |
|--------|------|
| PostgreSQL | 主业务数据，RLS 多租户隔离 |
| ClickHouse | 分析事件，按日分区 |
| Redis | 缓存、限流、会话 |

## 5. 事件流

```
用户行为 → Event Bus (NATS)
    ├─→ Analytics (ClickHouse)
    ├─→ Workflow (触发自动化)
    ├─→ Persistence
    └─→ WebSocket (实时推送)
```

## 6. 工作流引擎

基于状态机的 DAG 执行器：

- **触发器**：事件触发、定时触发、条件触发
- **步骤类型**：Send Message、Wait、Condition、Update Contact、Webhook
- **调度**：Wait 步骤由定时器恢复执行

## 7. 渠道集成

统一 `ChannelAdapter` trait：

```rust
#[async_trait]
pub trait ChannelAdapter: Send + Sync {
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthToken>;
    async fn send_message(&self, msg: &OutboundMessage) -> Result<MessageResult>;
    async fn receive_events(&self) -> Result<Vec<InboundEvent>>;
    async fn get_metrics(&self, range: &TimeRange) -> Result<ChannelMetrics>;
}
```

初期支持：邮件(SMTP)、微信公众号、小红书、抖音、广告平台。

## 8. 认证与多租户

- JWT (RS256)：access_token 15min + refresh_token 7d
- PostgreSQL RLS：所有表含 `tenant_id`，RLS Policy 强制隔离
- RBAC：organization_owner, admin, editor, viewer

## 9. API 设计

REST API，`/api/v1/` 前缀：

- `/auth/*` - 认证
- `/contacts/*` - 联系人管理
- `/campaigns/*` - 营销活动
- `/workflows/*` - 工作流
- `/channels/*` - 渠道管理
- `/analytics/*` - 数据分析
- `/track/*` - 埋点端点

## 10. 部署

- 开发：Docker Compose（postgres + redis + clickhouse + nats + server）
- 生产：Kubernetes（Deployment + StatefulSet）
