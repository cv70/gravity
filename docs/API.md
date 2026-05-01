# API 文档

## 1. 设计目标

Gravity API 是统一编排入口，不只是 CRUD 接口集合。它负责将前端、渠道、自动化工作流和分析系统连接起来，并对外提供一致的业务语义。

## 2. 约定

- Base Path: `/api/v1`
- 认证方式：JWT Access Token + Refresh Token
- 响应风格：统一 JSON
- 多租户：所有业务请求都绑定 `tenant_id`
- 高风险操作：需要审批或审计
- 幂等写入：重要写接口应支持幂等键

## 3. 认证

```http
POST /api/v1/auth/login
POST /api/v1/auth/refresh
DELETE /api/v1/auth/logout
```

登录返回用户身份、角色、访问令牌和刷新令牌。

## 4. 核心资源

### 4.1 Contacts

```http
GET    /api/v1/contacts
POST   /api/v1/contacts
PATCH  /api/v1/contacts/:id
DELETE /api/v1/contacts/:id
POST   /api/v1/contacts/batch
POST   /api/v1/contacts/import
```

用途：管理联系人、线索、标签、属性和生命周期状态。

### 4.2 Segments

```http
GET    /api/v1/segments
POST   /api/v1/segments
PATCH  /api/v1/segments/:id
DELETE /api/v1/segments/:id
POST   /api/v1/segments/:id/preview
```

用途：管理动态分群和人群包。

### 4.3 Campaigns

```http
GET    /api/v1/campaigns
POST   /api/v1/campaigns
PATCH  /api/v1/campaigns/:id
POST   /api/v1/campaigns/:id/launch
POST   /api/v1/campaigns/:id/pause
```

用途：管理营销、运营和转化活动。

### 4.4 Contents

```http
GET    /api/v1/contents
POST   /api/v1/contents
PATCH  /api/v1/contents/:id
POST   /api/v1/contents/:id/generate
POST   /api/v1/contents/:id/approve
```

用途：管理内容资产、模板和 AI 生成结果。

### 4.5 Workflows

```http
GET    /api/v1/workflows
POST   /api/v1/workflows
PATCH  /api/v1/workflows/:id
POST   /api/v1/workflows/:id/activate
POST   /api/v1/workflows/:id/deactivate
GET    /api/v1/workflows/:id/executions
GET    /api/v1/workflows/:id/executions/:execution_id
POST   /api/v1/workflows/:id/test
```

用途：管理自动化流程和执行实例。

### 4.6 Channels

```http
GET    /api/v1/channels
POST   /api/v1/channels/connect
POST   /api/v1/channels/:id/disconnect
POST   /api/v1/channels/:id/validate
GET    /api/v1/channels/:id/metrics
```

用途：管理渠道连接、授权和健康状态。

### 4.7 Analytics

```http
GET /api/v1/analytics/dashboard
GET /api/v1/analytics/funnels
GET /api/v1/analytics/attribution
GET /api/v1/analytics/campaigns/:id/performance
GET /api/v1/analytics/segments/:id/performance
```

用途：查看漏斗、归因、ROI、转化和运营健康度。

### 4.8 Approvals

```http
GET  /api/v1/approvals
POST /api/v1/approvals
PATCH /api/v1/approvals/:id
```

用途：处理高风险动作审批。

### 4.9 Audit

```http
GET /api/v1/audit-logs
```

用途：查看关键操作审计记录。

### 4.10 Tracking

```http
POST /api/v1/track/event
POST /api/v1/track/identify
POST /api/v1/track/conversion
POST /api/v1/track/webhook/:source
```

用途：接收用户行为、身份识别和转化事件。

### 4.11 Insights

```http
GET /api/v1/insights/recommendations
GET /api/v1/insights/anomalies
GET /api/v1/insights/opportunities
```

用途：返回策略建议、异常告警和增长机会。

## 5. 典型返回

```json
{
  "data": [],
  "total": 0,
  "page": 1,
  "limit": 20
}
```

错误返回统一包含：

```json
{
  "error": {
    "code": "invalid_request",
    "message": "...",
    "request_id": "..."
  }
}
```

## 6. 关键语义

### 6.1 自动化执行

API 调用工作流时，应返回：

- 是否成功启动
- 执行实例 ID
- 当前状态
- 是否需要审批

### 6.2 审批流

对高风险操作，API 不直接执行，而是创建审批对象，等待放行后再进入执行链路。

### 6.3 可追踪性

每一次写操作都应产生：

- 审计记录
- 事件记录
- 可选的分析事件

### 6.4 幂等与重试

对创建、触达、触发和回调类接口，服务端应支持幂等键和重复请求去重，避免多次执行同一动作。

## 7. 示例场景

### 7.1 启动工作流

```http
POST /api/v1/workflows/:id/activate
```

### 7.2 记录转化

```http
POST /api/v1/track/conversion
```

### 7.3 获取活动表现

```http
GET /api/v1/analytics/campaigns/:id/performance
```

## 8. 设计约束

- 不要把业务规则塞进 API 控制层
- 不要让前端直接依赖数据库语义
- 不要用不一致的资源命名
- 不要让高风险接口绕过审批
