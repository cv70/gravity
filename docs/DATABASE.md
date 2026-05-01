# 数据库设计

## 1. 设计目标

Gravity 的数据库设计服务于一个最终态目标：让运营、策略、执行、分析和治理形成统一闭环。数据模型需要同时支持事务型业务、事件型分析、流程状态和审计追踪。

## 2. 多租户隔离

所有业务表默认包含 `tenant_id` 字段，通过 PostgreSQL Row-Level Security (RLS) 强制隔离：

```sql
ALTER TABLE contacts ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON contacts
USING (tenant_id = current_setting('app.tenant_id')::uuid);
```

## 3. 核心数据域

### 3.1 组织与权限

```sql
CREATE TABLE organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    plan VARCHAR(50) DEFAULT 'enterprise',
    settings JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

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

### 3.2 联系人与画像

```sql
CREATE TABLE contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES organizations(id),
    external_id VARCHAR(255),
    email VARCHAR(255),
    phone VARCHAR(50),
    name VARCHAR(255),
    lifecycle_stage VARCHAR(50) DEFAULT 'new',
    tags TEXT[] DEFAULT '{}',
    attributes JSONB DEFAULT '{}',
    consent_state VARCHAR(50) DEFAULT 'unknown',
    subscribed BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, email)
);

CREATE TABLE segments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES organizations(id),
    name VARCHAR(255) NOT NULL,
    definition JSONB NOT NULL,
    is_dynamic BOOLEAN DEFAULT true,
    status VARCHAR(50) DEFAULT 'active',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

### 3.3 活动与内容

```sql
CREATE TABLE campaigns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES organizations(id),
    name VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL,
    status VARCHAR(50) DEFAULT 'draft',
    description TEXT,
    start_date DATE,
    end_date DATE,
    settings JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE contents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES organizations(id),
    campaign_id UUID REFERENCES campaigns(id),
    name VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL,
    content JSONB NOT NULL,
    status VARCHAR(50) DEFAULT 'draft',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

### 3.4 工作流与执行

```sql
CREATE TABLE workflows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES organizations(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    version INT NOT NULL DEFAULT 1,
    trigger_type VARCHAR(100) NOT NULL,
    trigger_config JSONB DEFAULT '{}',
    steps JSONB NOT NULL,
    status VARCHAR(50) DEFAULT 'draft',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE workflow_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES organizations(id),
    workflow_id UUID NOT NULL REFERENCES workflows(id),
    contact_id UUID NOT NULL REFERENCES contacts(id),
    status VARCHAR(50) NOT NULL,
    current_step_index INT DEFAULT 0,
    context JSONB DEFAULT '{}',
    started_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE TABLE workflow_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES organizations(id),
    workflow_execution_id UUID NOT NULL REFERENCES workflow_executions(id),
    resume_at TIMESTAMPTZ NOT NULL,
    payload JSONB DEFAULT '{}',
    status VARCHAR(50) DEFAULT 'pending',
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### 3.5 渠道与凭证

```sql
CREATE TABLE channel_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES organizations(id),
    platform VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    credentials_encrypted TEXT NOT NULL,
    settings JSONB DEFAULT '{}',
    status VARCHAR(50) DEFAULT 'disconnected',
    last_sync_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

### 3.6 事件与转化

```sql
CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES organizations(id),
    contact_id UUID REFERENCES contacts(id),
    source VARCHAR(50) NOT NULL DEFAULT 'system',
    event VARCHAR(255) NOT NULL,
    properties JSONB DEFAULT '{}',
    occurred_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE conversion_goals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES organizations(id),
    campaign_id UUID REFERENCES campaigns(id),
    name VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL,
    value DECIMAL(10,2),
    currency VARCHAR(10) DEFAULT 'CNY',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE conversions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES organizations(id),
    contact_id UUID NOT NULL REFERENCES contacts(id),
    goal_id UUID NOT NULL REFERENCES conversion_goals(id),
    value DECIMAL(10,2),
    currency VARCHAR(10) DEFAULT 'CNY',
    properties JSONB DEFAULT '{}',
    attributed_to UUID[],
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### 3.7 实验、审批与审计

```sql
CREATE TABLE experiments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES organizations(id),
    name VARCHAR(255) NOT NULL,
    hypothesis TEXT,
    status VARCHAR(50) DEFAULT 'draft',
    config JSONB DEFAULT '{}',
    results JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES organizations(id),
    object_type VARCHAR(50) NOT NULL,
    object_id UUID NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    requested_by UUID NOT NULL REFERENCES users(id),
    approved_by UUID REFERENCES users(id),
    reason TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    decided_at TIMESTAMPTZ
);

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES organizations(id),
    user_id UUID REFERENCES users(id),
    action VARCHAR(255) NOT NULL,
    target_type VARCHAR(50),
    target_id UUID,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

## 4. 分析存储

### 4.1 ClickHouse 事件明细

```sql
CREATE TABLE analytics_events (
    id UUID,
    tenant_id UUID,
    contact_id UUID,
    source VARCHAR(50),
    event VARCHAR(255),
    properties String,
    occurred_at DateTime
) ENGINE = MergeTree()
PARTITION BY toYYYYMMDD(occurred_at)
ORDER BY (tenant_id, occurred_at, contact_id);
```

### 4.2 漏斗与聚合

```sql
CREATE TABLE analytics_funnel_steps (
    id UUID,
    tenant_id UUID,
    campaign_id UUID,
    step_name VARCHAR(255),
    step_order Int32,
    count Int64,
    date Date
) ENGINE = SummingMergeTree()
ORDER BY (tenant_id, campaign_id, date);
```

## 5. 索引策略

```sql
CREATE INDEX idx_contacts_tenant_email ON contacts(tenant_id, email);
CREATE INDEX idx_contacts_tags_gin ON contacts USING GIN (tags);
CREATE INDEX idx_events_tenant_occurred ON events(tenant_id, occurred_at DESC);
CREATE INDEX idx_events_contact ON events(contact_id, occurred_at DESC);
CREATE INDEX idx_campaigns_tenant_status ON campaigns(tenant_id, status);
CREATE INDEX idx_workflow_exec_tenant_status ON workflow_executions(tenant_id, status);
```

## 6. 数据设计原则

- 主业务、执行状态和分析事件分层存储
- 工作流定义与执行实例分离，便于版本化和恢复
- 所有关键动作都要留下审计痕迹
- 转化、审批和实验结果要能和用户行为关联
- 数据模型尽量为多渠道和多业务场景保留扩展空间
- 事件表优先追加写入，避免把分析负载打到事务表上
