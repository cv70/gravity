# Gravity - 自动化推广营销转换平台

> 全渠道营销自动化 SaaS 平台，支持社交媒体、邮件/私域、内容营销、付费广告的整合管理。

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端 | TypeScript + React + Vite |
| 后端 | Rust (Axum) |
| 数据库 | PostgreSQL + ClickHouse + Redis |
| 消息队列 | NATS |
| 部署 | Docker Compose / Kubernetes |

## 核心功能

- **联系人管理** - 统一管理所有渠道的客户数据，支持标签、分群、导入导出
- **营销活动** - 创建、执行全渠道营销活动，跟踪效果
- **工作流自动化** - 可视化拖拽配置营销自动化流程
- **数据分析** - 实时看板、漏斗分析、归因模型
- **渠道集成** - 微信/小红书/抖音/邮件/广告平台的统一接入

## 项目结构

```
gravity/
├── crates/               # Rust 后端 (模块化单体)
│   ├── gravity-api/      # HTTP API 层
│   ├── gravity-core/     # 核心领域模型
│   ├── gravity-db/       # 数据库访问层
│   ├── gravity-channels/# 第三方渠道集成
│   ├── gravity-workflow/# 工作流引擎
│   ├── gravity-analytics/# 分析引擎
│   └── gravity-common/  # 共享工具
├── frontend/            # React 前端
├── docker-compose.yml   # 本地开发环境
└── Cargo.toml           # Rust workspace
```

## 快速开始

```bash
# 启动本地开发环境
docker compose up -d

# 后端开发
cd crates/gravity-api && cargo run

# 前端开发
cd frontend && npm install && npm run dev
```

## 架构设计

详见 [ARCHITECTURE.md](./ARCHITECTURE.md)
