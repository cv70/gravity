import { useMemo, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { automationService } from '@/services/automation'
import { Bot, Plus, Play, ThumbsUp, ThumbsDown, ShieldAlert, RefreshCw, Workflow } from 'lucide-react'

export function WorkflowsPage() {
  const queryClient = useQueryClient()
  const [showModal, setShowModal] = useState(false)

  const { data, isLoading } = useQuery({
    queryKey: ['automation', 'dashboard'],
    queryFn: automationService.getDashboard,
  })

  const createMutation = useMutation({
    mutationFn: automationService.createJob,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['automation', 'dashboard'] })
      setShowModal(false)
    },
  })

  const bootstrapMutation = useMutation({
    mutationFn: automationService.bootstrapDefaults,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['automation', 'dashboard'] }),
  })

  const executeMutation = useMutation({
    mutationFn: ({ id }: { id: string }) => automationService.executeJob(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['automation', 'dashboard'] }),
  })

  const approveMutation = useMutation({
    mutationFn: ({ id, approved }: { id: string; approved: boolean }) => automationService.reviewApproval(id, approved, approved ? '已通过自动化动作' : '已驳回高风险动作'),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['automation', 'dashboard'] }),
  })

  const overviewCards = useMemo(() => (data ? [
    { label: '任务总数', value: data.overview.total_jobs, icon: Workflow, tone: 'from-sky-500 to-cyan-500' },
    { label: '活跃任务', value: data.overview.active_jobs, icon: Bot, tone: 'from-emerald-500 to-teal-500' },
    { label: '待审批', value: data.overview.pending_approvals, icon: ShieldAlert, tone: 'from-amber-500 to-orange-500' },
    { label: '风控拦截', value: data.overview.blocked_actions, icon: RefreshCw, tone: 'from-rose-500 to-pink-500' },
  ] : []), [data])

  const runsByJobId = useMemo(() => {
    const map = new Map<string, string>()
    data?.runs?.forEach((run) => {
      map.set(run.job_id, run.current_step)
    })
    return map
  }, [data?.runs])

  const getWorkflowBlueprint = (strategy: Record<string, unknown>) => {
    const raw = strategy.workflow_blueprint
    if (Array.isArray(raw)) {
      return raw.filter((step): step is string => typeof step === 'string')
    }
    return ['identify', 'segment', 'generate_content', 'choose_channel', 'send', 'wait', 'evaluate', 'optimize']
  }

  const handleCreate = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const form = e.currentTarget
    const formData = new FormData(form)
    const channels = String(formData.get('channels') || 'email').split(',').map((item) => item.trim()).filter(Boolean)

    await createMutation.mutateAsync({
      goal: formData.get('goal') as string,
      target_audience: {
        persona: formData.get('persona') as string,
        lifecycle: formData.get('lifecycle') as string,
      },
      channel_preferences: channels,
      budget_limit: Number(formData.get('budget') || 0) || undefined,
      currency: 'CNY',
      desired_outcome: formData.get('outcome') as string,
      approval_required: formData.get('approval') === 'yes',
    })

    form.reset()
  }

  return (
    <div className="space-y-6">
      <div className="relative overflow-hidden rounded-3xl bg-slate-950 text-white shadow-xl border border-slate-800">
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_top_right,rgba(34,211,238,0.25),transparent_35%),radial-gradient(circle_at_bottom_left,rgba(59,130,246,0.2),transparent_30%)]" />
        <div className="relative flex flex-col gap-4 p-6 lg:p-8">
          <div className="flex items-start justify-between gap-4">
            <div>
              <div className="inline-flex items-center gap-2 rounded-full bg-white/10 px-3 py-1 text-xs text-cyan-100">
                <Bot className="h-4 w-4" />
                AI 代理主导的自动化中枢
              </div>
              <h1 className="mt-4 text-3xl font-bold tracking-tight">工作流不再手搭，策略可以直接起飞</h1>
              <p className="mt-3 max-w-3xl text-sm lg:text-base text-slate-300">
                从目标、渠道、预算到审批风控，系统会自动生成运营任务图，只有高风险动作才进入人工审批。
              </p>
            </div>
            <button
              onClick={() => setShowModal(true)}
              className="inline-flex items-center gap-2 rounded-xl bg-cyan-400 px-4 py-2 text-sm font-semibold text-slate-950 shadow-lg shadow-cyan-500/20 transition hover:bg-cyan-300"
            >
              <Plus className="h-4 w-4" />
              新建任务
            </button>
          </div>

          <div className="grid grid-cols-1 gap-3 md:grid-cols-4">
            {overviewCards.map((card) => (
              <div key={card.label} className="rounded-2xl border border-white/10 bg-white/5 p-4 backdrop-blur">
                <div className={`inline-flex rounded-xl bg-gradient-to-br ${card.tone} p-2`}>
                  <card.icon className="h-4 w-4 text-white" />
                </div>
                <p className="mt-4 text-2xl font-bold">{card.value}</p>
                <p className="mt-1 text-sm text-slate-300">{card.label}</p>
              </div>
            ))}
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 xl:grid-cols-3 gap-6">
        <div className="xl:col-span-2 bg-white rounded-2xl border border-gray-200 shadow-sm overflow-hidden">
          <div className="px-6 py-4 border-b flex items-center justify-between">
            <div>
              <h2 className="text-lg font-semibold text-gray-900">任务图</h2>
              <p className="text-sm text-gray-500">生成、执行、审批和优化的闭环状态</p>
            </div>
            <Workflow className="h-5 w-5 text-brand-600" />
          </div>

          <div className="p-6">
            {isLoading ? (
              <div className="py-8 text-center text-gray-500">加载中...</div>
            ) : data?.jobs?.length ? (
              <div className="space-y-4">
                {data.jobs.map((job) => (
                  <div key={job.id} className="rounded-2xl border border-gray-200 p-4 hover:shadow-sm transition-shadow">
                    <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
                      <div>
                        <p className="font-semibold text-gray-900">{job.goal}</p>
                        <p className="mt-1 text-sm text-gray-500">
                          渠道: {job.channel_preferences.join(' / ') || '未设置'} · 预算: {job.budget_limit ?? 0} {job.currency} · 风险: {job.risk_level}
                        </p>
                        <p className="mt-2 text-xs text-gray-400">
                          下次动作: {job.next_action_at ? new Date(job.next_action_at).toLocaleString('zh-CN') : '待调度'}
                        </p>
                        <div className="mt-3 flex flex-wrap gap-2">
                          {getWorkflowBlueprint(job.strategy).map((step) => {
                            const currentStep = runsByJobId.get(job.id)
                            const isCurrent = currentStep === step
                            const isPast = currentStep ? getWorkflowBlueprint(job.strategy).indexOf(step) < getWorkflowBlueprint(job.strategy).indexOf(currentStep) : false
                            return (
                              <span
                                key={step}
                                className={`rounded-full px-2.5 py-1 text-xs border ${
                                  isCurrent ? 'bg-brand-600 text-white border-brand-600' :
                                  isPast ? 'bg-emerald-50 text-emerald-700 border-emerald-200' :
                                  'bg-gray-50 text-gray-500 border-gray-200'
                                }`}
                              >
                                {step}
                              </span>
                            )
                          })}
                        </div>
                      </div>
                      <div className="flex flex-wrap items-center gap-2">
                        <span className={`px-2 py-1 text-xs rounded-full ${
                          job.status === 'active' ? 'bg-green-100 text-green-700' :
                          job.status === 'waiting_approval' ? 'bg-amber-100 text-amber-700' :
                          job.status === 'completed' ? 'bg-blue-100 text-blue-700' :
                          'bg-gray-100 text-gray-700'
                        }`}>
                          {job.status}
                        </span>
                        <button
                          onClick={() => executeMutation.mutate({ id: job.id })}
                          disabled={executeMutation.isPending}
                          className="inline-flex items-center gap-1 rounded-lg border border-gray-200 px-3 py-1.5 text-sm text-gray-700 transition hover:bg-gray-50 disabled:opacity-50"
                        >
                          <Play className="h-4 w-4" />
                          执行
                        </button>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="rounded-2xl bg-gray-50 p-8 text-center text-gray-500 space-y-4">
                <p>暂无自动化任务，先初始化默认模板，或者手动创建一个欢迎流。</p>
                <button
                  onClick={() => bootstrapMutation.mutate()}
                  disabled={bootstrapMutation.isPending}
                  className="inline-flex items-center gap-2 rounded-xl bg-brand-600 px-4 py-2.5 text-sm font-medium text-white hover:bg-brand-700 disabled:opacity-50"
                >
                  <RefreshCw className="h-4 w-4" />
                  {bootstrapMutation.isPending ? '初始化中...' : '初始化默认模板'}
                </button>
              </div>
            )}
          </div>
        </div>

        <div className="bg-white rounded-2xl border border-gray-200 shadow-sm overflow-hidden">
          <div className="px-6 py-4 border-b">
            <h2 className="text-lg font-semibold text-gray-900">审批队列</h2>
          </div>
          <div className="p-6">
            {data?.approvals?.length ? (
              <div className="space-y-4">
                {data.approvals.map((approval) => (
                  <div key={approval.id} className="rounded-2xl border border-gray-200 p-4">
                    <div className="flex items-start justify-between gap-3">
                      <div>
                        <p className="font-medium text-gray-900">{approval.title}</p>
                        <p className="mt-1 text-sm text-gray-500">{approval.reason}</p>
                      </div>
                      <span className="rounded-full bg-amber-100 px-2 py-1 text-xs text-amber-700">{approval.status}</span>
                    </div>
                    <div className="mt-4 flex gap-2">
                      <button
                        onClick={() => approveMutation.mutate({ id: approval.id, approved: true })}
                        disabled={approveMutation.isPending}
                        className="inline-flex items-center gap-1 rounded-lg bg-emerald-600 px-3 py-2 text-sm text-white transition hover:bg-emerald-500 disabled:opacity-50"
                      >
                        <ThumbsUp className="h-4 w-4" />
                        通过
                      </button>
                      <button
                        onClick={() => approveMutation.mutate({ id: approval.id, approved: false })}
                        disabled={approveMutation.isPending}
                        className="inline-flex items-center gap-1 rounded-lg border border-gray-200 px-3 py-2 text-sm text-gray-700 transition hover:bg-gray-50 disabled:opacity-50"
                      >
                        <ThumbsDown className="h-4 w-4" />
                        驳回
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="rounded-2xl bg-gray-50 p-8 text-center text-gray-500 space-y-4">
                <p>没有待审批动作，自动化链路正在自我执行。</p>
                <button
                  onClick={() => bootstrapMutation.mutate()}
                  disabled={bootstrapMutation.isPending}
                  className="inline-flex items-center gap-2 rounded-xl border border-gray-200 px-4 py-2.5 text-sm font-medium text-gray-700 hover:bg-white disabled:opacity-50"
                >
                  <RefreshCw className="h-4 w-4" />
                  补齐默认风控模板
                </button>
              </div>
            )}
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="bg-white rounded-2xl border border-gray-200 shadow-sm overflow-hidden">
          <div className="px-6 py-4 border-b">
            <h2 className="text-lg font-semibold text-gray-900">最近运行</h2>
          </div>
          <div className="p-6">
            {data?.runs?.length ? (
              <div className="space-y-3">
                {data.runs.map((run) => (
                  <div key={run.id} className="flex items-center justify-between rounded-xl bg-gray-50 px-4 py-3">
                    <div>
                      <p className="font-medium text-gray-900">{run.current_step}</p>
                      <p className="text-sm text-gray-500">{run.status} · {new Date(run.started_at).toLocaleString('zh-CN')}</p>
                    </div>
                    <span className="text-xs text-gray-500">{run.job_id.slice(0, 8)}</span>
                  </div>
                ))}
              </div>
            ) : (
              <p className="py-8 text-center text-gray-500">暂无运行记录</p>
            )}
          </div>
        </div>

        <div className="bg-white rounded-2xl border border-gray-200 shadow-sm overflow-hidden">
          <div className="px-6 py-4 border-b">
            <h2 className="text-lg font-semibold text-gray-900">优化建议</h2>
          </div>
          <div className="p-6 space-y-3">
            {data?.recommendations?.length ? data.recommendations.map((item) => (
              <div key={item} className="rounded-xl bg-slate-950 px-4 py-3 text-sm leading-6 text-slate-200">
                {item}
              </div>
            )) : (
              <div className="rounded-xl bg-gray-50 p-6 text-sm text-gray-500">
                建议从规则、实验和渠道组合开始优化。
              </div>
            )}
          </div>
        </div>
      </div>

      {showModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
          <div className="w-full max-w-xl rounded-2xl bg-white shadow-2xl">
            <div className="border-b px-6 py-4">
              <h3 className="text-lg font-semibold text-gray-900">创建自动化任务</h3>
              <p className="text-sm text-gray-500">描述目标后，系统会自动生成策略和风控等级。</p>
            </div>
            <form onSubmit={handleCreate} className="space-y-4 p-6">
              <div>
                <label className="mb-1 block text-sm font-medium text-gray-700">目标</label>
                <input name="goal" required className="w-full rounded-xl border border-gray-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100" placeholder="例如：激活沉默 30 天用户" />
              </div>
              <div className="grid gap-4 md:grid-cols-2">
                <div>
                  <label className="mb-1 block text-sm font-medium text-gray-700">受众标签</label>
                  <input name="persona" className="w-full rounded-xl border border-gray-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100" placeholder="例如：高意向线索" />
                </div>
                <div>
                  <label className="mb-1 block text-sm font-medium text-gray-700">生命周期阶段</label>
                  <input name="lifecycle" className="w-full rounded-xl border border-gray-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100" placeholder="例如：new / warm / churn" />
                </div>
              </div>
              <div className="grid gap-4 md:grid-cols-2">
                <div>
                  <label className="mb-1 block text-sm font-medium text-gray-700">渠道（逗号分隔）</label>
                  <input name="channels" defaultValue="email,wechat" className="w-full rounded-xl border border-gray-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100" />
                </div>
                <div>
                  <label className="mb-1 block text-sm font-medium text-gray-700">预算</label>
                  <input name="budget" type="number" min="0" step="0.01" className="w-full rounded-xl border border-gray-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100" placeholder="0" />
                </div>
              </div>
              <div>
                <label className="mb-1 block text-sm font-medium text-gray-700">目标结果</label>
                <input name="outcome" className="w-full rounded-xl border border-gray-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100" placeholder="例如：30 天内提升转化 15%" />
              </div>
              <div>
                <label className="mb-1 block text-sm font-medium text-gray-700">是否强制审批</label>
                <select name="approval" defaultValue="no" className="w-full rounded-xl border border-gray-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100">
                  <option value="no">否，交给系统自动判断</option>
                  <option value="yes">是，高风险动作必须审批</option>
                </select>
              </div>
              <div className="flex justify-end gap-3 pt-2">
                <button type="button" onClick={() => setShowModal(false)} className="rounded-xl border border-gray-200 px-4 py-2.5 text-sm font-medium text-gray-700 hover:bg-gray-50">
                  取消
                </button>
                <button type="submit" className="rounded-xl bg-brand-600 px-4 py-2.5 text-sm font-medium text-white hover:bg-brand-700">
                  创建并生成策略
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  )
}
