# 工作流引擎设计

## 1. 设计目标

Gravity 的工作流引擎不是简单的自动化脚本执行器，而是运营动作的统一编排层。它负责把策略、规则、内容、渠道和反馈串成可恢复、可审计、可扩展的执行链路。

工作流引擎必须支持：

- 自动触发、自动分支、自动等待、自动补偿、自动重试
- 多渠道触达与状态回写
- 人工审批插入关键节点
- 失败恢复与幂等执行
- 与实验、风控、归因、画像联动

## 2. 核心概念

### 2.1 Workflow

Workflow 是一套有向图或状态机定义，描述一个完整的运营动作流程。它需要版本化、可回放、可暂停、可恢复。

```json
{
  "id": "wf_01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "name": "新用户自动培育流程",
  "version": 3,
  "trigger_type": "contact.created",
  "trigger_config": {},
  "steps": [
    {
      "id": "step_1",
      "type": "send_message",
      "config": {
        "channel": "email",
        "template_id": "welcome",
        "subject": "欢迎加入"
      }
    },
    {
      "id": "step_2",
      "type": "wait",
      "config": {
        "delay_hours": 24
      }
    },
    {
      "id": "step_3",
      "type": "condition",
      "config": {
        "field": "email_opened",
        "operator": "equals",
        "value": true
      },
      "branches": {
        "true": ["step_4"],
        "false": ["step_5"]
      }
    }
  ]
}
```

### 2.2 Trigger

触发器决定工作流何时启动。

| 类型 | 说明 |
|------|------|
| `contact.created` | 新联系人或线索进入系统时触发 |
| `contact.tag_added` | 标签变化触发 |
| `contact.segment_entered` | 用户进入某个分群时触发 |
| `message.opened` | 消息被打开时触发 |
| `message.clicked` | 消息被点击时触发 |
| `conversion.recorded` | 发生转化时触发 |
| `schedule.cron` | 定时触发 |
| `manual.start` | 人工触发 |

### 2.3 Step

步骤是工作流中的执行节点，必须具备明确的输入、输出和失败语义。

| 类型 | 作用 |
|------|------|
| `send_message` | 调用渠道发送消息或内容 |
| `wait` | 延时等待或等待某个条件满足 |
| `condition` | 条件分支 |
| `update_contact` | 更新标签、属性和生命周期状态 |
| `add_to_segment` | 加入分群 |
| `remove_from_segment` | 移出分群 |
| `webhook` | 调用外部系统 |
| `track_event` | 写入自定义事件 |
| `ai_generate` | 生成内容或策略建议 |
| `approval` | 进入人工审批 |
| `alert` | 发送告警或升级处理 |

## 3. 执行模型

工作流执行采用“状态机 + 事件驱动 + 延迟恢复”的组合模式。

### 3.1 执行流程

1. 触发事件进入系统
2. 匹配可执行的 Workflow
3. 创建 WorkflowExecution
4. 逐步执行节点
5. 遇到 `wait` 节点时挂起并登记恢复时间
6. 遇到 `condition` 节点时按规则选择分支
7. 遇到 `approval` 节点时暂停，等待人工放行
8. 触发渠道执行并接收回执
9. 更新画像、任务状态和分析数据
10. 继续下一节点或结束执行

### 3.2 状态

- `pending`：等待开始
- `running`：执行中
- `waiting`：等待恢复或等待审批
- `completed`：成功结束
- `failed`：执行失败
- `cancelled`：被取消或失效

### 3.3 执行实例

执行实例应保存至少三类信息：

- 启动时使用的工作流版本
- 当前执行到的步骤与上下文
- 触达、等待、审批、重试和回写结果

## 4. 并发与幂等

- 同一联系人同一工作流同时只允许一个有效执行实例
- 同一节点的重复投递必须具备幂等保护
- 触发事件需要去重，避免重复创建执行实例
- 恢复任务需要可重放，避免调度器故障导致流程丢失

```rust
async fn try_start_workflow(contact_id: Uuid, workflow_id: Uuid) -> Result<ExecutionId> {
    let lock_key = format!("workflow:running:{}:{}", contact_id, workflow_id);
    if !redis::set_nx(&lock_key, EXECUTION_LOCK_TTL).await? {
        return Err(WorkflowError::AlreadyRunning);
    }

    let execution = create_execution(contact_id, workflow_id).await?;
    Ok(execution.id)
}
```

## 5. 调度与恢复

`wait` 节点不会阻塞线程，而是写入调度队列或待恢复表，由后台调度器按时间恢复。

恢复规则：

- 到点后自动恢复
- 如果依赖条件未满足，可重新排队或进入补偿分支
- 如果工作流定义已变更，恢复时需按版本策略处理
- 如果工作流被停用，后续恢复应进入安全终止路径

## 6. 版本与变更

- 工作流定义必须版本化，历史执行只绑定启动时版本
- 新版本发布后，不影响已运行中的执行实例
- 如需迁移，必须通过明确的迁移任务或人工批准
- 版本差异应可审计、可回放、可解释

## 7. 事件联动

工作流会订阅和发布领域事件，形成闭环：

- 输入：`contact.*`、`message.*`、`conversion.*`、`segment.*`
- 输出：`workflow.started`、`workflow.step_executed`、`workflow.completed`、`workflow.failed`
- 联动：画像更新、策略重算、分析入库、任务派发、审批流转

## 8. 可视化编辑器

前端编辑器使用节点/边结构表达工作流，允许运营和产品在可视化界面中配置流程。

```typescript
interface WorkflowNode {
  id: string
  type: 'trigger' | 'action' | 'condition' | 'delay' | 'approval'
  position: { x: number; y: number }
  data: {
    stepType: string
    config: Record<string, unknown>
    label: string
  }
}
```

## 9. 设计约束

- 工作流必须可解释，不能只依赖黑盒决策
- 任何自动化动作都必须可追踪到来源策略和触发条件
- 高风险动作必须可插入审批
- 工作流应尽量配置化，不依赖硬编码业务分支
