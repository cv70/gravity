# 分阶段实施计划

## Phase 1: 基础架构 + 核心流程 (4-6 周)

**目标**：跑通"联系人管理 → 创建活动 → 发送邮件 → 追踪行为 → 查看数据"端到端流程

### 任务清单

| 优先级 | 任务 | 交付物 |
|--------|------|--------|
| P0 | Cargo workspace + crate 结构 | 所有 Cargo.toml + 目录结构 |
| P0 | 数据库 Schema 迁移 | SQLx migrations + RLS policies |
| P0 | gravity-common (ID、配置、加密) | `lib.rs` 工具模块 |
| P0 | gravity-core (领域模型) | Contact, Campaign, Content, Conversion 实体 |
| P0 | gravity-db (Repository) | ContactRepository, CampaignRepository 等 |
| P0 | gravity-api (Axum 服务) | main.rs + 认证中间件 |
| P0 | CRUD API | /contacts, /campaigns, /content 路由 |
| P0 | React 脚手架 | Vite + React Router + Tailwind + Zustand |
| P1 | 前端页面 | Dashboard, Contact List, Campaign List |
| P1 | 邮件 ChannelAdapter | SMTP 发送 |
| P1 | 埋点 API | /track/event, /track/identify |
| P1 | ClickHouse 事件写入 | analytics_events 表写入 |
| P2 | 前端简易 Dashboard | 数据显示 |

### 验收标准
- [ ] `docker compose up` 启动所有服务
- [ ] 用户注册/登录，获取 JWT
- [ ] 创建联系人，列表展示
- [ ] 创建邮件活动，启动发送
- [ ] 埋点记录 `message.opened` 事件
- [ ] Dashboard 显示活动数据

---

## Phase 2: 工作流引擎 + 社交渠道 (4-6 周)

**目标**：实现营销自动化工作流，接入微信/小红书

### 任务清单

| 优先级 | 任务 | 交付物 |
|--------|------|--------|
| P0 | gravity-workflow 引擎 | State machine interpreter |
| P0 | DAG 解析与执行 | Step executor |
| P0 | 触发器系统 | Event triggers, schedule triggers |
| P0 | Wait 步骤 + Scheduler | 延时调度恢复 |
| P0 | Workflow API | /workflows CRUD, /activate |
| P1 | ReactFlow 工作流编辑器 | 可视化拖拽界面 |
| P1 | 微信公众号 Adapter | OAuth + 发送消息 |
| P1 | 小红书 Adapter | OAuth + 发布笔记 |
| P1 | NATS 事件总线 | Domain event pub/sub |
| P1 | 联系人分群 | 基于规则的动态分群 |
| P2 | 渠道管理页面 | 渠道连接/断开 UI |
| P2 | 工作流管理页面 | 创建/编辑/激活工作流 |

### 验收标准
- [ ] 创建工作流：Trigger + Send Email + Wait + Condition + Send WeChat
- [ ] 新联系人创建时自动触发工作流
- [ ] Wait 步骤正确延时后恢复执行
- [ ] ReactFlow 可视化编辑工作流
- [ ] 微信公众号消息发送成功
- [ ] 小红书内容发布成功

---

## Phase 3: 数据分析 + 付费广告 (4-6 周)

**目标**：完整分析能力，漏斗归因，广告渠道

### 任务清单

| 优先级 | 任务 | 交付物 |
|--------|------|--------|
| P0 | 漏斗分析查询 | ClickHouse 多步骤漏斗 SQL |
| P0 | 归因模型 | First-touch, Last-touch, Linear |
| P0 | Campaign Performance API | /analytics/campaigns/:id/performance |
| P0 | 前端漏斗图 | ECharts 漏斗可视化 |
| P0 | 前端归因报表 | 归因可视化 |
| P1 | 实时看板 | WebSocket 推送 + 实时聚合 |
| P1 | 抖音 Adapter | OAuth + 视频/直播消息 |
| P1 | 广告平台 Adapter | 巨量引擎 / Google Ads |
| P1 | A/B 测试 | 活动多版本支持 |
| P2 | ROI 分析 | 广告投入产出计算 |
| P2 | 数据导出 | CSV/Excel 导出 |

### 验收标准
- [ ] 漏斗图展示：发送 → 打开 → 点击 → 转化
- [ ] 归因报表显示各触点贡献
- [ ] 实时数据更新 (WebSocket)
- [ ] 抖音消息发送
- [ ] 广告平台数据同步

---

## Phase 4: AI 增强 + 企业级特性 (持续迭代)

### 任务清单

| 优先级 | 任务 | 交付物 |
|--------|------|--------|
| P1 | AI 文案生成 | LLM 接入 + 模板生成 |
| P1 | 智能分群 | 行为聚类 + 自动分群 |
| P1 | 最佳发送时间 | 预测模型 |
| P2 | 完整 RBAC | 组织 Owner/Admin/Editor/Viewer |
| P2 | 审计日志 | 操作审计 |
| P2 | 开放 API | Webhook + SDK |
| P3 | 多语言支持 | i18n |
| P3 | 白标定制 | 主题定制 |

---

## 技术债务与重构点

1. **Event Bus 选型**：Phase 1 先用进程内 `tokio::broadcast`，Phase 2 切换到 NATS
2. **前端状态管理**：Phase 1 简单 Zustand，Phase 2 可考虑切换到 TanStack Query
3. **测试覆盖**：每 crate 至少 80% 单元测试，E2E 测试覆盖核心流程
4. **性能优化**：ClickHouse 聚合查询缓存，Redis 热点数据缓存

---

## 里程碑

| 阶段 | 预计周期 | 关键里程碑 |
|------|----------|------------|
| Phase 1 | 4-6 周 | 端到端邮件营销流程跑通 |
| Phase 2 | 4-6 周 | 自动化工作流上线 |
| Phase 3 | 4-6 周 | 完整数据分析能力 |
| Phase 4 | 持续 | AI + 企业级功能 |
