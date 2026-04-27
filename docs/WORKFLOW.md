# 工作流引擎设计

## 核心概念

### Workflow (工作流定义)
工作流是一个有向无环图 (DAG)，定义了自动化流程的结构。

```json
{
  "id": "wf_01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "name": "新用户欢迎流程",
  "trigger_type": "contact.created",
  "trigger_config": {},
  "steps": [
    {
      "id": "step_1",
      "type": "send_message",
      "config": {
        "channel": "email",
        "template_id": "welcome",
        "subject": "欢迎加入！"
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
    },
    {
      "id": "step_4",
      "type": "send_message",
      "config": {
        "channel": "wechat",
        "template_id": "followup"
      }
    },
    {
      "id": "step_5",
      "type": "update_contact",
      "config": {
        "tags": ["needs_followup"]
      }
    }
  ]
}
```

### Trigger (触发器)

触发工作流执行的事件来源：

| 类型 | 配置 | 说明 |
|------|------|------|
| `contact.created` | - | 新联系人创建时触发 |
| `contact.tag_added` | `{"tag": "vip"}` | 添加指定标签时触发 |
| `contact.segment_entered` | `{"segment_id": "..."}` | 进入分群时触发 |
| `message.clicked` | - | 消息被点击时触发 |
| `schedule.cron` | `{"cron": "0 9 * * *"}` | 定时触发 |
| `campaign.launched` | - | 活动启动时触发 |
| `conversion.recorded` | - | 转化发生时触发 |

### Step (执行步骤)

| 类型 | 配置 | 说明 |
|------|------|------|
| `send_message` | `channel`, `template_id`, `content` | 发送消息 |
| `wait` | `delay_hours` 或 `delay_until` | 延时等待 |
| `condition` | `field`, `operator`, `value` | 条件分支 |
| `update_contact` | `tags`, `attributes` | 更新联系人 |
| `add_to_list` | `list_id` | 加入分群 |
| `remove_from_list` | `list_id` | 移出分群 |
| `webhook` | `url`, `method`, `headers`, `body` | 调用外部 API |
| `track_event` | `event`, `properties` | 记录自定义事件 |
| `ai_generate` | `prompt_template`, `output_field` | AI 生成内容 |

## 引擎执行流程

```
1. 触发事件到达
       │
       ▼
2. 创建 WorkflowExecution (status: pending)
       │
       ▼
3. Engine 加载 Workflow DAG
       │
       ▼
4. 按拓扑序执行 Step
       │
       ├──→ Step 类型判断
       │         │
       │         ├──→ Send Message → 调用 ChannelAdapter
       │         ├──→ Wait → 挂起，存入 scheduler
       │         ├──→ Condition → 评估规则，选择分支
       │         └──→ 其他类型 → 执行对应逻辑
       │
       ▼
5. 产生领域事件 (workflow.step_executed)
       │
       ▼
6. 更新 Execution 状态
       │
       ├──→ 继续 → 回第 3 步
       │
       └──→ 完成/Failed → 结束
```

## Wait 步骤的调度

Wait 步骤需要暂停执行并在延时后恢复，引擎通过 scheduler 模块管理：

```rust
// Wait 执行时挂起
async fn execute_wait_step(execution: &mut WorkflowExecution, step: &Step) {
    let delay_hours = step.config.delay_hours;
    let resume_at = Utc::now() + Duration::hours(delay_hours);

    // 写入待调度表
    scheduler::schedule_resume(execution.id, step.id, resume_at).await?;

    // 更新执行状态为 waiting
    execution.status = ExecutionStatus::Waiting;
    execution.current_step_index += 1;
}
```

Scheduler 后台任务定期扫描待恢复的执行：

```rust
async fn process_scheduled_resumes() {
    let now = Utc::now();
    let pending = scheduler::fetch_due_executions(now).await?;

    for (execution_id, step_id) in pending {
        if let Some(mut execution) = workflow.load_execution(execution_id).await? {
            execution.status = ExecutionStatus::Running;
            engine.resume_from_step(&mut execution, step_id).await?;
        }
    }
}
```

## 状态机

```
                    ┌──────────────┐
                    │   Pending    │
                    └──────┬───────┘
                           │ engine.start()
                           ▼
                    ┌──────────────┐
              ┌─────│   Running    │─────┐
              │     └──────────────┘     │
              │                          │
   step.type == "wait"         step completed normally
              │                          │
              ▼                          ▼
     ┌──────────────┐           ┌──────────────┐
     │   Waiting    │           │   Running    │
     └──────┬───────┘           └──────────────┘
            │ scheduler.resume()
            │
            ▼
     ┌──────────────┐
     │   Running    │ ──────────────────► (继续执行下一步)
     └──────────────┘
              │
              │ condition branch selected
              ▼
     ┌──────────────┐
     │   Running    │
     └──────────────┘
              │
              ├──── step failed ────┐
              │                     │
              ▼                     ▼
     ┌──────────────┐     ┌──────────────┐
     │   Failed     │     │  Completed   │
     └──────────────┘     └──────────────┘
```

## 并发控制

- 每个 Contact 同一时间只能有一个运行的 Workflow Execution
- 防止重复触发：Contact 已在某 Workflow 运行中时，新触发事件会被忽略或排队

```rust
async fn try_start_workflow(contact_id: Uuid, workflow_id: Uuid) -> Result<ExecutionId> {
    // 使用 Redis SETNX 实现分布式锁
    let lock_key = format!("workflow:running:{}:{}", contact_id, workflow_id);
    if !redis::set_nx(&lock_key, EXECUTION_LOCK_TTL).await? {
        return Err(WorkflowError::AlreadyRunning);
    }

    let execution = create_execution(contact_id, workflow_id).await?;
    Ok(execution.id)
}
```

## 事件驱动集成

Workflow 通过 NATS 订阅领域事件来触发：

```rust
async fn subscribe_to_triggers(nats: &NatsContext) {
    // 监听 contact.created 事件
    nats.subscribe("contact.created", |event| {
        let contact_id = event.contact_id;
        let workflows = find_triggered_workflows("contact.created").await?;
        for workflow in workflows {
            engine.start(workflow.id, contact_id).await?;
        }
    }).await;
}
```

## 可视化编辑器数据结构

前端 ReactFlow 使用的节点/边结构：

```typescript
interface WorkflowNode {
  id: string
  type: 'trigger' | 'action' | 'condition' | 'delay'
  position: { x: number; y: number }
  data: {
    stepType: string
    config: Record<string, unknown>
    label: string
  }
}

interface WorkflowEdge {
  id: string
  source: string
  target: string
  sourceHandle?: 'true' | 'false' // condition 的分支
  label?: string
}
```
