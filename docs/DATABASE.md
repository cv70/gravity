# 数据库设计

## 多租户隔离

所有业务表包含 `tenant_id` 字段，通过 PostgreSQL Row-Level Security (RLS) 强制隔离：

```sql
ALTER TABLE contacts ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON contacts USING (tenant_id = current_setting('app.tenant_id')::uuid);
```

## 核心表结构

### organizations (组织)
```sql
CREATE TABLE organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    plan VARCHAR(50) DEFAULT 'free',
    settings JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

### users (用户)
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES organizations(id),
    email VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'viewer',
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, email)
);
```

### contacts (联系人)
```sql
CREATE TABLE contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES organizations(id),
    email VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    phone VARCHAR(50),
    tags TEXT[] DEFAULT '{}',
    attributes JSONB DEFAULT '{}',
    subscribed BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, email)
);
```

### campaigns (营销活动)
```sql
CREATE TABLE campaigns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES organizations(id),
    name VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL, -- social, email, content, ads
    status VARCHAR(50) DEFAULT 'draft',
    description TEXT,
    start_date DATE,
    end_date DATE,
    settings JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

### contents (内容素材)
```sql
CREATE TABLE contents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES organizations(id),
    campaign_id UUID REFERENCES campaigns(id),
    name VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL, -- email_template, social_post, ad_copy
    content JSONB NOT NULL,
    status VARCHAR(50) DEFAULT 'draft',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

### workflows (自动化工作流)
```sql
CREATE TABLE workflows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES organizations(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    trigger_type VARCHAR(100) NOT NULL,
    trigger_config JSONB DEFAULT '{}',
    steps JSONB NOT NULL, -- DAG 定义
    status VARCHAR(50) DEFAULT 'draft',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

### workflow_executions (工作流执行实例)
```sql
CREATE TABLE workflow_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID NOT NULL REFERENCES workflows(id),
    contact_id UUID NOT NULL REFERENCES contacts(id),
    status VARCHAR(50) NOT NULL, -- pending, running, completed, failed
    current_step_index INT DEFAULT 0,
    context JSONB DEFAULT '{}',
    started_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);
```

### channel_accounts (渠道账号)
```sql
CREATE TABLE channel_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES organizations(id),
    platform VARCHAR(50) NOT NULL, -- wechat, xiaohongshu, douyin, email, ads
    name VARCHAR(255) NOT NULL,
    credentials_encrypted TEXT NOT NULL, -- AES-256-GCM 加密
    settings JSONB DEFAULT '{}',
    status VARCHAR(50) DEFAULT 'disconnected',
    last_sync_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

### conversion_goals (转化目标)
```sql
CREATE TABLE conversion_goals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES organizations(id),
    campaign_id UUID REFERENCES campaigns(id),
    name VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL, -- purchase, signup, form_submit
    value DECIMAL(10,2),
    currency VARCHAR(10) DEFAULT 'CNY',
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### conversions (转化记录)
```sql
CREATE TABLE conversions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES organizations(id),
    contact_id UUID NOT NULL REFERENCES contacts(id),
    goal_id UUID NOT NULL REFERENCES conversion_goals(id),
    value DECIMAL(10,2),
    currency VARCHAR(10) DEFAULT 'CNY',
    properties JSONB DEFAULT '{}',
    attributed_to UUID[], -- 归因的活动/触点
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### events (行为事件)
```sql
CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES organizations(id),
    contact_id UUID REFERENCES contacts(id),
    event VARCHAR(255) NOT NULL,
    properties JSONB DEFAULT '{}',
    occurred_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 同步到 ClickHouse 的物化视图
CREATE MATERIALIZED VIEW events_analytics
TO analytics_events AS
SELECT * FROM events;
```

## ClickHouse 表

### analytics_events (行为事件明细)
```sql
CREATE TABLE analytics_events (
    id UUID,
    tenant_id UUID,
    contact_id UUID,
    event VARCHAR(255),
    properties JSONB,
    occurred_at TIMESTAMP
) ENGINE = MergeTree()
PARTITION BY toYYYYMMDD(occurred_at)
ORDER BY (tenant_id, occurred_at, contact_id);
```

### analytics_funnel_steps (漏斗步骤)
```sql
CREATE TABLE analytics_funnel_steps (
    id UUID,
    tenant_id UUID,
    campaign_id UUID,
    step_name VARCHAR(255),
    step_order INT,
    count INT64,
    date DATE
) ENGINE = SummingMergeTree()
ORDER BY (tenant_id, campaign_id, date);
```

## 索引策略

```sql
-- contacts 查询优化
CREATE INDEX idx_contacts_tenant_email ON contacts(tenant_id, email);
CREATE INDEX idx_contacts_tenant_tags ON contacts USING GIN(tenant_id, tags);

-- events 时间范围查询
CREATE INDEX idx_events_tenant_occurred ON events(tenant_id, occurred_at DESC);
CREATE INDEX idx_events_contact ON events(contact_id, occurred_at DESC);

-- campaigns 状态筛选
CREATE INDEX idx_campaigns_tenant_status ON campaigns(tenant_id, status);
```
