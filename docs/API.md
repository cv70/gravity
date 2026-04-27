# API 文档

## 认证

### 登录
```
POST /api/v1/auth/login
Content-Type: application/json

{
  "email": "admin@example.com",
  "password": "password"
}

Response 200:
{
  "user": {
    "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "email": "admin@example.com",
    "name": "Admin",
    "role": "organization_owner"
  },
  "access_token": "eyJ...",
  "refresh_token": "eyJ..."
}
```

### 刷新 Token
```
POST /api/v1/auth/refresh
Content-Type: application/json

{
  "refresh_token": "eyJ..."
}

Response 200:
{
  "access_token": "eyJ..."
}
```

### 登出
```
DELETE /api/v1/auth/logout
Authorization: Bearer <access_token>
```

---

## 联系人

### 列表
```
GET /api/v1/contacts
Authorization: Bearer <access_token>

Query: ?page=1&limit=20&search=keyword

Response 200:
{
  "data": [
    {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "email": "user@example.com",
      "name": "张三",
      "phone": "13800138000",
      "tags": ["vip", "意向客户"],
      "attributes": {"company": "示例公司"},
      "created_at": "2024-01-15T10:30:00Z",
      "updated_at": "2024-01-15T10:30:00Z"
    }
  ],
  "total": 100,
  "page": 1,
  "limit": 20
}
```

### 创建
```
POST /api/v1/contacts
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "email": "user@example.com",
  "name": "张三",
  "phone": "13800138000",
  "tags": ["vip"],
  "attributes": {"company": "示例公司"}
}

Response 201: <contact object>
```

### 批量创建
```
POST /api/v1/contacts/batch
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "contacts": [
    {"email": "user1@example.com", "name": "用户1"},
    {"email": "user2@example.com", "name": "用户2"}
  ]
}

Response 201: {
  "created": 2,
  "errors": []
}
```

### 导入 CSV
```
POST /api/v1/contacts/import
Authorization: Bearer <access_token>
Content-Type: multipart/form-data

file: <csv file>

Response 200:
{
  "imported": 100,
  "errors": 5
}
```

### 更新
```
PATCH /api/v1/contacts/:id
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "name": "新名字",
  "tags": ["vip", "重要"]
}

Response 200: <contact object>
```

### 删除
```
DELETE /api/v1/contacts/:id
Authorization: Bearer <access_token>

Response 204: No Content
```

---

## 营销活动

### 列表
```
GET /api/v1/campaigns
Authorization: Bearer <access_token>

Response 200:
{
  "data": [
    {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "name": "春季促销",
      "type": "email",
      "status": "active",
      "description": "春季促销活动",
      "start_date": "2024-03-01",
      "end_date": "2024-03-31",
      "metrics": {
        "sent": 10000,
        "opened": 3500,
        "clicked": 800,
        "converted": 120
      },
      "created_at": "2024-02-15T10:30:00Z",
      "updated_at": "2024-03-01T10:30:00Z"
    }
  ]
}
```

### 创建
```
POST /api/v1/campaigns
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "name": "春季促销",
  "type": "email",
  "description": "春季促销活动",
  "start_date": "2024-03-01",
  "end_date": "2024-03-31"
}

Response 201: <campaign object>
```

### 启动
```
POST /api/v1/campaigns/:id/launch
Authorization: Bearer <access_token>

Response 200: <campaign object (status: "active")>
```

### 暂停
```
POST /api/v1/campaigns/:id/pause
Authorization: Bearer <access_token>

Response 200: <campaign object (status: "paused")>
```

---

## 工作流

### 列表
```
GET /api/v1/workflows
Authorization: Bearer <access_token>

Response 200:
{
  "data": [
    {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "name": "新用户欢迎流程",
      "status": "active",
      "trigger_type": "contact.created",
      "steps": [...],
      "created_at": "2024-01-15T10:30:00Z"
    }
  ]
}
```

### 创建
```
POST /api/v1/workflows
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "name": "新用户欢迎流程",
  "trigger_type": "contact.created",
  "steps": [
    {
      "type": "send_message",
      "config": {
        "channel": "email",
        "template_id": "welcome"
      }
    },
    {
      "type": "wait",
      "config": {"delay_hours": 24}
    },
    {
      "type": "condition",
      "config": {
        "field": "opened_email",
        "operator": "equals",
        "value": true
      }
    }
  ]
}

Response 201: <workflow object>
```

### 激活
```
POST /api/v1/workflows/:id/activate
Authorization: Bearer <access_token>

Response 200: <workflow object (status: "active")>
```

### 执行记录
```
GET /api/v1/workflows/:id/executions
Authorization: Bearer <access_token>

Response 200:
{
  "data": [
    {
      "id": "exec_01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "workflow_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "contact_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "status": "completed",
      "started_at": "2024-03-15T10:30:00Z",
      "completed_at": "2024-03-15T10:30:05Z"
    }
  ]
}
```

---

## 渠道

### 列表
```
GET /api/v1/channels
Authorization: Bearer <access_token>

Response 200:
{
  "data": [
    {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "platform": "wechat",
      "name": "微信公众号",
      "status": "connected",
      "account_id": "wx_123456",
      "last_sync_at": "2024-03-15T10:30:00Z"
    }
  ]
}
```

### 连接渠道
```
POST /api/v1/channels/:platform/connect
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "name": "微信公众号",
  "app_id": "wx_xxxxx",
  "app_secret": "xxxxx"
}

Response 201: <channel object>
```

### 状态检查
```
GET /api/v1/channels/:id/status
Authorization: Bearer <access_token>

Response 200:
{
  "connected": true,
  "last_check_at": "2024-03-15T10:30:00Z",
  "error": null
}
```

---

## 分析

### 仪表盘
```
GET /api/v1/analytics/dashboard
Authorization: Bearer <access_token>

Response 200:
{
  "total_contacts": 10500,
  "active_campaigns": 8,
  "total_conversions": 1250,
  "conversion_rate": 0.119,
  "recent_events": [...],
  "campaign_performance": [
    {
      "campaign_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "campaign_name": "春季促销",
      "sent": 10000,
      "opened": 3500,
      "clicked": 800,
      "converted": 120
    }
  ]
}
```

### 漏斗分析
```
GET /api/v1/analytics/funnel
Authorization: Bearer <access_token>
Query: ?campaign_id=01ARZ3NDEKTSV4RRFFQ69G5FAV

Response 200:
{
  "steps": [
    {"step": "发送", "count": 10000, "dropoff_rate": 0},
    {"step": "打开", "count": 3500, "dropoff_rate": 0.65},
    {"step": "点击", "count": 800, "dropoff_rate": 0.77},
    {"step": "转化", "count": 120, "dropoff_rate": 0.85}
  ]
}
```

---

## 埋点

### 事件追踪
```
POST /api/v1/track/event
Content-Type: application/json

{
  "event": "message.opened",
  "contact_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "properties": {
    "campaign_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "message_id": "msg_123"
  },
  "timestamp": "2024-03-15T10:30:00Z"
}
```

### Identify
```
POST /api/v1/track/identify
Content-Type: application/json

{
  "contact_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "traits": {
    "name": "张三",
    "company": "示例公司"
  }
}
```

### 页面浏览
```
POST /api/v1/track/page
Content-Type: application/json

{
  "contact_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "name": "/pricing",
  "url": "https://example.com/pricing",
  "referrer": "https://example.com/"
}
```

### 转化追踪
```
POST /api/v1/track/conversion
Content-Type: application/json

{
  "contact_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "goal_id": "goal_01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "value": 99.00,
  "currency": "CNY",
  "properties": {
    "order_id": "order_123"
  }
}
```

---

## 错误响应

所有错误返回统一格式：

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid email format",
    "details": {
      "field": "email",
      "reason": "格式不正确"
    }
  }
}
```

常见错误码：
- `400 VALIDATION_ERROR` - 请求参数验证失败
- `401 UNAUTHORIZED` - 未认证或 Token 过期
- `403 FORBIDDEN` - 无权限
- `404 NOT_FOUND` - 资源不存在
- `429 RATE_LIMITED` - 请求过于频繁
- `500 INTERNAL_ERROR` - 服务器内部错误
