import { useMemo, useState, type FormEvent } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import {
  Activity,
  AlertTriangle,
  ArrowUpRight,
  BadgeCheck,
  BarChart3,
  Bot,
  CheckCircle2,
  ChevronRight,
  Clock3,
  Filter,
  Flame,
  FlaskConical,
  Layers3,
  MessageSquareWarning,
  Pause,
  Play,
  Plus,
  RefreshCw,
  Rocket,
  Shield,
  Sparkles,
  Target,
  ThumbsDown,
  ThumbsUp,
  Wand2,
  Workflow,
  Zap,
} from 'lucide-react'
import { automationService } from '@/services/automation'
import { Button } from '@/components/ui/button'
import type { AutomationJob, AutomationAction, ApprovalRequest, Experiment, PolicyRule, AutomationRun } from '@/types'

type JobStatusFilter = 'all' | AutomationJob['status']

const CHANNEL_OPTIONS = [
  { id: 'email', label: 'Email' },
  { id: 'wechat', label: '微信' },
  { id: 'xiaohongshu', label: '小红书' },
  { id: 'douyin', label: '抖音' },
  { id: 'ads', label: '广告投放' },
  { id: 'sms', label: '短信' },
]

const STATUS_TONES: Record<string, string> = {
  draft: 'bg-slate-100 text-slate-700 border-slate-200',
  waiting_approval: 'bg-amber-100 text-amber-700 border-amber-200',
  active: 'bg-emerald-100 text-emerald-700 border-emerald-200',
  paused: 'bg-orange-100 text-orange-700 border-orange-200',
  completed: 'bg-blue-100 text-blue-700 border-blue-200',
  failed: 'bg-rose-100 text-rose-700 border-rose-200',
  queued: 'bg-slate-100 text-slate-700 border-slate-200',
  running: 'bg-cyan-100 text-cyan-700 border-cyan-200',
  approved: 'bg-emerald-100 text-emerald-700 border-emerald-200',
  rejected: 'bg-rose-100 text-rose-700 border-rose-200',
  pending: 'bg-amber-100 text-amber-700 border-amber-200',
}

const DEFAULT_BLUEPRINT = ['identify', 'segment', 'generate_content', 'choose_channel', 'send', 'measure', 'optimize']

function formatDateTime(value?: string | null) {
  if (!value) return '—'
  return new Date(value).toLocaleString('zh-CN', { hour12: false })
}

function getStrategySteps(strategy: Record<string, unknown>) {
  const raw = strategy.workflow_blueprint
  if (Array.isArray(raw) && raw.every((item) => typeof item === 'string')) {
    return raw as string[]
  }
  return DEFAULT_BLUEPRINT
}

function pickStatus(job: AutomationJob) {
  return STATUS_TONES[job.status] ?? 'bg-slate-100 text-slate-700 border-slate-200'
}

function pickTone(status: string) {
  return STATUS_TONES[status] ?? 'bg-slate-100 text-slate-700 border-slate-200'
}

export function WorkflowsPage() {
  const queryClient = useQueryClient()
  const [showModal, setShowModal] = useState(false)
  const [selectedJobId, setSelectedJobId] = useState<string | null>(null)
  const [statusFilter, setStatusFilter] = useState<JobStatusFilter>('all')
  const [selectedChannels, setSelectedChannels] = useState<string[]>(['email', 'wechat'])

  const { data, isLoading, isFetching, refetch } = useQuery({
    queryKey: ['automation', 'dashboard'],
    queryFn: automationService.getDashboard,
  })

  const createMutation = useMutation({
    mutationFn: automationService.createJob,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['automation', 'dashboard'] })
      setShowModal(false)
      setSelectedChannels(['email', 'wechat'])
    },
  })

  const bootstrapMutation = useMutation({
    mutationFn: automationService.bootstrapDefaults,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['automation', 'dashboard'] })
    },
  })

  const executeMutation = useMutation({
    mutationFn: ({ id }: { id: string }) => automationService.executeJob(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['automation', 'dashboard'] })
    },
  })

  const approveMutation = useMutation({
    mutationFn: ({ id, approved }: { id: string; approved: boolean }) =>
      automationService.reviewApproval(
        id,
        approved,
        approved ? '已通过自动化动作' : '已驳回高风险动作',
      ),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['automation', 'dashboard'] })
    },
  })

  const jobs = useMemo(() => data?.jobs ?? [], [data?.jobs])
  const filteredJobs = useMemo(() => {
    if (statusFilter === 'all') return jobs
    return jobs.filter((job) => job.status === statusFilter)
  }, [jobs, statusFilter])

  const selectedJob = useMemo(
    () => {
      if (!filteredJobs.length) return null
      return filteredJobs.find((job) => job.id === selectedJobId) ?? filteredJobs[0]
    },
    [filteredJobs, selectedJobId],
  )

  const selectedRuns = useMemo<AutomationRun[]>(() => {
    if (!selectedJob) return []
    return (data?.runs ?? []).filter((run) => run.job_id === selectedJob.id)
  }, [data?.runs, selectedJob])

  const selectedActions = useMemo<AutomationAction[]>(() => {
    if (!selectedJob) return []
    return (data?.actions ?? []).filter((action) =>
      selectedRuns.some((run) => run.id === action.run_id),
    )
  }, [data?.actions, selectedJob, selectedRuns])

  const selectedApprovals = useMemo<ApprovalRequest[]>(() => {
    if (!selectedJob) return data?.approvals ?? []
    const actionIds = new Set(selectedActions.map((action) => action.id))
    return (data?.approvals ?? []).filter((approval) => actionIds.has(approval.action_id))
  }, [data?.approvals, selectedActions, selectedJob])

  const selectedPolicies = useMemo<PolicyRule[]>(() => {
    if (!selectedJob) return data?.policies ?? []
    return (data?.policies ?? []).filter((policy) => {
      const scope = policy.scope as Record<string, unknown> | undefined
      const jobId = scope?.job_id
      return jobId === selectedJob.id || !jobId
    })
  }, [data?.policies, selectedJob])

  const selectedExperiments = useMemo<Experiment[]>(() => {
    if (!selectedJob) return data?.experiments ?? []
    return (data?.experiments ?? []).filter((experiment) => experiment.job_id === selectedJob.id || !experiment.job_id)
  }, [data?.experiments, selectedJob])

  const overviewCards = useMemo(() => {
    if (!data) return []
    return [
      { label: '任务总数', value: data.overview.total_jobs, icon: Workflow, tone: 'from-cyan-500 to-blue-500' },
      { label: '运行中', value: data.overview.runs_in_progress, icon: Activity, tone: 'from-emerald-500 to-teal-500' },
      { label: '待审批', value: data.overview.pending_approvals, icon: Shield, tone: 'from-amber-500 to-orange-500' },
      { label: '风控拦截', value: data.overview.blocked_actions, icon: AlertTriangle, tone: 'from-rose-500 to-pink-500' },
      { label: '已启用策略', value: data.overview.enabled_policies, icon: BadgeCheck, tone: 'from-violet-500 to-fuchsia-500' },
      { label: '自动化覆盖', value: `${data.overview.automation_coverage}%`, icon: BarChart3, tone: 'from-sky-500 to-cyan-500' },
    ]
  }, [data])

  const filterCounts = useMemo(() => {
    const counts: Record<JobStatusFilter, number> = {
      all: jobs.length,
      draft: 0,
      waiting_approval: 0,
      active: 0,
      paused: 0,
      completed: 0,
      failed: 0,
    }
    jobs.forEach((job) => {
      counts[job.status] = (counts[job.status] ?? 0) + 1
    })
    return counts
  }, [jobs])

  const selectedBlueprint = useMemo(() => {
    if (!selectedJob) return DEFAULT_BLUEPRINT
    return getStrategySteps(selectedJob.strategy)
  }, [selectedJob])

  const handleCreate = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    const form = event.currentTarget
    const formData = new FormData(form)

    await createMutation.mutateAsync({
      goal: String(formData.get('goal') || ''),
      target_audience: {
        persona: String(formData.get('persona') || ''),
        lifecycle: String(formData.get('lifecycle') || ''),
        region: String(formData.get('region') || ''),
        intent: String(formData.get('intent') || ''),
        note: String(formData.get('note') || ''),
      },
      channel_preferences: selectedChannels,
      budget_limit: Number(formData.get('budget') || 0) || undefined,
      currency: String(formData.get('currency') || 'CNY'),
      desired_outcome: String(formData.get('outcome') || ''),
      approval_required: formData.get('approval_required') === 'yes',
    })

    form.reset()
  }

  return (
    <div className="space-y-6">
      <div className="relative overflow-hidden rounded-3xl border border-slate-800 bg-slate-950 text-white shadow-2xl">
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_top_right,rgba(56,189,248,0.25),transparent_28%),radial-gradient(circle_at_bottom_left,rgba(14,165,233,0.18),transparent_30%),linear-gradient(135deg,rgba(15,23,42,0.96),rgba(2,6,23,0.98))]" />
        <div className="absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-cyan-400/60 to-transparent" />
        <div className="relative flex flex-col gap-6 p-6 lg:p-8">
          <div className="flex flex-col gap-5 xl:flex-row xl:items-start xl:justify-between">
            <div className="max-w-4xl">
              <div className="inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/8 px-3 py-1 text-xs text-cyan-100 backdrop-blur">
                <Bot className="h-4 w-4" />
                企业级自动化运营中枢
              </div>
              <h1 className="mt-4 text-3xl font-semibold tracking-tight text-white md:text-5xl">
                用一套系统，把运营、审批、执行和复盘全部串起来
              </h1>
              <p className="mt-4 max-w-3xl text-sm leading-7 text-slate-300 md:text-base">
                这里不是“任务列表”，而是可商用的自动化编排中心：系统自动分流、执行、留痕、审批与实验，人工只介入高风险动作。
              </p>
            </div>

            <div className="flex flex-wrap gap-3">
              <Button
                onClick={() => refetch()}
                variant="ghost"
              >
                <RefreshCw className={`h-4 w-4 ${isFetching ? 'animate-spin' : ''}`} />
                刷新看板
              </Button>
              <Button
                onClick={() => setShowModal(true)}
                variant="brand"
              >
                <Plus className="h-4 w-4" />
                新建自动化任务
              </Button>
            </div>
          </div>

          <div className="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-6">
            {overviewCards.map((card) => (
              <div key={card.label} className="rounded-2xl border border-white/10 bg-white/6 p-4 backdrop-blur">
                <div className={`inline-flex rounded-xl bg-gradient-to-br ${card.tone} p-2`}>
                  <card.icon className="h-4 w-4 text-white" />
                </div>
                <p className="mt-4 text-2xl font-semibold text-white">{card.value}</p>
                <p className="mt-1 text-sm text-slate-300">{card.label}</p>
              </div>
            ))}
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-6 xl:grid-cols-[1.7fr_1fr]">
        <div className="space-y-6">
          <div className="rounded-2xl border border-slate-200 bg-white shadow-sm">
            <div className="flex flex-col gap-4 border-b border-slate-200 px-6 py-5 lg:flex-row lg:items-center lg:justify-between">
              <div>
                <div className="flex items-center gap-2 text-sm font-medium text-slate-500">
                  <Filter className="h-4 w-4" />
                  任务过滤器
                </div>
                <h2 className="mt-1 text-lg font-semibold text-slate-900">按状态筛选任务流</h2>
              </div>
              <div className="flex flex-wrap gap-2">
                  {(['all', 'draft', 'waiting_approval', 'active', 'paused', 'completed', 'failed'] as JobStatusFilter[]).map((status) => (
                  <Button
                    key={status}
                    onClick={() => setStatusFilter(status)}
                    variant="secondary"
                    size="sm"
                    className={`rounded-full px-3 py-1.5 ${
                      statusFilter === status
                        ? 'border-slate-900 bg-slate-900 text-white'
                        : 'border-slate-200 bg-white text-slate-600 hover:bg-slate-50'
                    }`}
                  >
                    {status === 'all' ? '全部' : status}
                    <span className={`rounded-full px-2 py-0.5 text-xs ${statusFilter === status ? 'bg-white/12' : 'bg-slate-100'}`}>
                      {filterCounts[status]}
                    </span>
                  </Button>
                ))}
              </div>
            </div>

            <div className="p-6">
              {isLoading ? (
                <div className="flex min-h-[18rem] items-center justify-center text-slate-500">加载自动化中枢...</div>
              ) : filteredJobs.length ? (
                <div className="grid gap-4">
                  {filteredJobs.map((job) => {
                    const isSelected = selectedJob?.id === job.id
                    const steps = getStrategySteps(job.strategy)
                    const currentStep = data?.runs?.find((run) => run.job_id === job.id)?.current_step
                    const currentIndex = currentStep ? steps.indexOf(currentStep) : -1

                    return (
                      <div
                        key={job.id}
                        onClick={() => setSelectedJobId(job.id)}
                        onKeyDown={(event) => {
                          if (event.key === 'Enter' || event.key === ' ') {
                            event.preventDefault()
                            setSelectedJobId(job.id)
                          }
                        }}
                        role="button"
                        tabIndex={0}
                        aria-pressed={isSelected}
                        className={`text-left rounded-2xl border p-5 transition ${
                          isSelected
                            ? 'border-cyan-400 bg-cyan-50 shadow-[0_0_0_1px_rgba(34,211,238,0.18)]'
                            : 'border-slate-200 bg-white hover:border-slate-300 hover:shadow-sm'
                        }`}
                      >
                        <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
                          <div className="space-y-3">
                            <div className="flex flex-wrap items-center gap-2">
                              <span className={`rounded-full border px-2.5 py-1 text-xs font-medium ${pickStatus(job)}`}>
                                {job.status}
                              </span>
                              <span className="inline-flex items-center gap-1 rounded-full border border-slate-200 bg-slate-50 px-2.5 py-1 text-xs text-slate-600">
                                {job.risk_level === 'high' ? <Flame className="h-3.5 w-3.5" /> : <Shield className="h-3.5 w-3.5" />}
                                风险 {job.risk_level}
                              </span>
                              {job.approval_required && (
                                <span className="inline-flex items-center gap-1 rounded-full border border-amber-200 bg-amber-50 px-2.5 py-1 text-xs text-amber-700">
                                  <Shield className="h-3.5 w-3.5" />
                                  强制审批
                                </span>
                              )}
                            </div>

                            <div>
                              <h3 className="text-base font-semibold text-slate-900">{job.goal}</h3>
                              <p className="mt-1 text-sm leading-6 text-slate-600">
                                受众：{String(job.target_audience.persona ?? '未指定')} · 生命周期：{String(job.target_audience.lifecycle ?? '未指定')}
                              </p>
                              <p className="mt-1 text-sm leading-6 text-slate-600">
                                渠道：{job.channel_preferences.join(' / ') || '未设置'} · 预算：{job.budget_limit ?? 0} {job.currency}
                              </p>
                            </div>

                            <div className="flex flex-wrap gap-2">
                              {steps.map((step, index) => {
                                const isCurrent = currentStep === step
                                const isPast = currentIndex >= 0 && index < currentIndex
                                return (
                                  <span
                                    key={step}
                                    className={`rounded-full border px-3 py-1 text-xs ${
                                      isCurrent
                                        ? 'border-cyan-500 bg-cyan-500 text-white'
                                        : isPast
                                          ? 'border-emerald-200 bg-emerald-50 text-emerald-700'
                                          : 'border-slate-200 bg-slate-50 text-slate-500'
                                    }`}
                                  >
                                    {step}
                                  </span>
                                )
                              })}
                            </div>
                          </div>

                          <div className="flex flex-col items-start gap-3 lg:items-end">
                            <div className="flex items-center gap-2 text-sm text-slate-500">
                              <Clock3 className="h-4 w-4" />
                              下次动作 {formatDateTime(job.next_action_at)}
                            </div>
                            <div className="flex flex-wrap gap-2">
                              <Button
                                onClick={(event) => {
                                  event.stopPropagation()
                                  executeMutation.mutate({ id: job.id })
                                }}
                                disabled={executeMutation.isPending}
                                variant="brand"
                                size="sm"
                              >
                                <Play className="h-4 w-4" />
                                执行
                              </Button>
                              {job.status === 'active' ? (
                                <span className="inline-flex items-center gap-1.5 rounded-xl border border-slate-200 px-3 py-2 text-sm text-slate-600">
                                  <Pause className="h-4 w-4" />
                                  运行中
                                </span>
                              ) : (
                                <span className="inline-flex items-center gap-1.5 rounded-xl border border-slate-200 px-3 py-2 text-sm text-slate-600">
                                  <Wand2 className="h-4 w-4" />
                                  可调度
                                </span>
                              )}
                            </div>
                          </div>
                        </div>
                      </div>
                    )
                  })}
                </div>
              ) : (
                <div className="flex min-h-[18rem] flex-col items-center justify-center rounded-2xl border border-dashed border-slate-200 bg-slate-50 px-6 text-center">
                  <Sparkles className="h-10 w-10 text-slate-400" />
                  <p className="mt-4 text-base font-medium text-slate-900">没有符合条件的任务</p>
                  <p className="mt-2 max-w-md text-sm leading-6 text-slate-500">
                    可以先初始化默认模板，或者创建一个面向沉默用户、首购用户的自动化编排。
                  </p>
                  <Button
                    onClick={() => bootstrapMutation.mutate()}
                    disabled={bootstrapMutation.isPending}
                    variant="brand"
                    className="mt-5"
                  >
                    <Rocket className="h-4 w-4" />
                    {bootstrapMutation.isPending ? '初始化中...' : '初始化默认模板'}
                  </Button>
                </div>
              )}
            </div>
          </div>

          <div className="grid gap-6 lg:grid-cols-2">
            <section className="rounded-2xl border border-slate-200 bg-white shadow-sm">
              <div className="flex items-center justify-between border-b border-slate-200 px-6 py-4">
                <div>
                  <p className="text-sm font-medium text-slate-500">运行流</p>
                  <h3 className="text-lg font-semibold text-slate-900">最近执行</h3>
                </div>
                <Activity className="h-5 w-5 text-cyan-600" />
              </div>
              <div className="p-6">
                {selectedRuns.length ? (
                  <div className="space-y-3">
                    {selectedRuns.slice(0, 6).map((run) => (
                      <div key={run.id} className="rounded-2xl border border-slate-200 bg-slate-50 p-4">
                        <div className="flex items-start justify-between gap-4">
                          <div>
                            <p className="font-medium text-slate-900">{run.current_step}</p>
                            <p className="mt-1 text-sm text-slate-600">{run.status}</p>
                          </div>
                          <span className={`rounded-full border px-2.5 py-1 text-xs ${pickTone(run.status)}`}>
                            {run.status}
                          </span>
                        </div>
                        <div className="mt-3 flex items-center justify-between text-xs text-slate-500">
                          <span>{formatDateTime(run.started_at)}</span>
                          <span>{run.id.slice(0, 8)}</span>
                        </div>
                        {run.last_error && (
                          <div className="mt-3 rounded-xl border border-rose-200 bg-rose-50 px-3 py-2 text-sm text-rose-700">
                            {run.last_error}
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                ) : (
                  <div className="rounded-2xl border border-dashed border-slate-200 bg-slate-50 p-6 text-sm text-slate-500">
                    当前任务没有运行记录。
                  </div>
                )}
              </div>
            </section>

            <section className="rounded-2xl border border-slate-200 bg-white shadow-sm">
              <div className="flex items-center justify-between border-b border-slate-200 px-6 py-4">
                <div>
                  <p className="text-sm font-medium text-slate-500">动作流</p>
                  <h3 className="text-lg font-semibold text-slate-900">执行动作</h3>
                </div>
                <Zap className="h-5 w-5 text-amber-600" />
              </div>
              <div className="p-6">
                {selectedActions.length ? (
                  <div className="space-y-3">
                    {selectedActions.slice(0, 6).map((action) => (
                      <div key={action.id} className="rounded-2xl border border-slate-200 bg-slate-50 p-4">
                        <div className="flex items-start justify-between gap-4">
                          <div>
                            <p className="font-medium text-slate-900">{action.action_type}</p>
                            <p className="mt-1 text-sm text-slate-600">{action.channel}</p>
                          </div>
                          <span className={`rounded-full border px-2.5 py-1 text-xs ${pickTone(action.status)}`}>
                            {action.status}
                          </span>
                        </div>
                        <div className="mt-3 flex flex-wrap gap-2 text-xs">
                          <span className="rounded-full border border-slate-200 bg-white px-2.5 py-1 text-slate-600">
                            风险 {action.risk_level}
                          </span>
                          <span className="rounded-full border border-slate-200 bg-white px-2.5 py-1 text-slate-600">
                            需审批 {action.requires_approval ? '是' : '否'}
                          </span>
                        </div>
                      </div>
                    ))}
                  </div>
                ) : (
                  <div className="rounded-2xl border border-dashed border-slate-200 bg-slate-50 p-6 text-sm text-slate-500">
                    当前任务没有生成动作记录。
                  </div>
                )}
              </div>
            </section>
          </div>

          <section className="rounded-2xl border border-slate-200 bg-white shadow-sm">
            <div className="flex items-center justify-between border-b border-slate-200 px-6 py-4">
              <div>
                <p className="text-sm font-medium text-slate-500">审批流</p>
                <h3 className="text-lg font-semibold text-slate-900">待审与已审</h3>
              </div>
              <Shield className="h-5 w-5 text-amber-600" />
            </div>
            <div className="p-6">
              {selectedApprovals.length ? (
                <div className="grid gap-4 xl:grid-cols-2">
                  {selectedApprovals.map((approval) => (
                    <div key={approval.id} className="rounded-2xl border border-slate-200 bg-slate-50 p-4">
                      <div className="flex items-start justify-between gap-4">
                        <div>
                          <p className="font-medium text-slate-900">{approval.title}</p>
                          <p className="mt-1 text-sm leading-6 text-slate-600">{approval.reason}</p>
                        </div>
                        <span className={`rounded-full border px-2.5 py-1 text-xs ${pickTone(approval.status)}`}>
                          {approval.status}
                        </span>
                      </div>
                      <div className="mt-3 flex flex-wrap gap-2">
                        <Button
                          onClick={() => approveMutation.mutate({ id: approval.id, approved: true })}
                          disabled={approveMutation.isPending}
                          variant="brand"
                          size="sm"
                        >
                          <ThumbsUp className="h-4 w-4" />
                          通过
                        </Button>
                        <Button
                          onClick={() => approveMutation.mutate({ id: approval.id, approved: false })}
                          disabled={approveMutation.isPending}
                          variant="destructive"
                          size="sm"
                        >
                          <ThumbsDown className="h-4 w-4" />
                          驳回
                        </Button>
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="rounded-2xl border border-dashed border-slate-200 bg-slate-50 p-6 text-sm text-slate-500">
                  当前没有待处理审批。
                </div>
              )}
            </div>
          </section>

          <div className="grid gap-6 lg:grid-cols-2">
            <section className="rounded-2xl border border-slate-200 bg-white shadow-sm">
              <div className="flex items-center justify-between border-b border-slate-200 px-6 py-4">
                <div>
                  <p className="text-sm font-medium text-slate-500">策略流</p>
                  <h3 className="text-lg font-semibold text-slate-900">政策与护栏</h3>
                </div>
                <Layers3 className="h-5 w-5 text-violet-600" />
              </div>
              <div className="p-6 space-y-3">
                {selectedPolicies.length ? (
                  selectedPolicies.map((policy) => (
                    <div key={policy.id} className="rounded-2xl border border-slate-200 bg-slate-50 p-4">
                      <div className="flex items-start justify-between gap-4">
                        <div>
                          <p className="font-medium text-slate-900">{policy.name}</p>
                          <p className="mt-1 text-sm text-slate-600">{policy.rule_type}</p>
                        </div>
                        <span className={`rounded-full border px-2.5 py-1 text-xs ${policy.enabled ? 'border-emerald-200 bg-emerald-50 text-emerald-700' : 'border-slate-200 bg-slate-100 text-slate-500'}`}>
                          {policy.enabled ? '启用' : '停用'}
                        </span>
                      </div>
                    </div>
                  ))
                ) : (
                  <div className="rounded-2xl border border-dashed border-slate-200 bg-slate-50 p-6 text-sm text-slate-500">
                    没有匹配到该任务的策略规则。
                  </div>
                )}
              </div>
            </section>

            <section className="rounded-2xl border border-slate-200 bg-white shadow-sm">
              <div className="flex items-center justify-between border-b border-slate-200 px-6 py-4">
                <div>
                  <p className="text-sm font-medium text-slate-500">实验流</p>
                  <h3 className="text-lg font-semibold text-slate-900">A/B 测试与增长实验</h3>
                </div>
                <FlaskConical className="h-5 w-5 text-rose-600" />
              </div>
              <div className="p-6 space-y-3">
                {selectedExperiments.length ? (
                  selectedExperiments.map((experiment) => (
                    <div key={experiment.id} className="rounded-2xl border border-slate-200 bg-slate-50 p-4">
                      <div className="flex items-start justify-between gap-4">
                        <div>
                          <p className="font-medium text-slate-900">{experiment.name}</p>
                          <p className="mt-1 text-sm leading-6 text-slate-600">{experiment.hypothesis}</p>
                        </div>
                        <span className={`rounded-full border px-2.5 py-1 text-xs ${pickTone(experiment.status)}`}>
                          {experiment.status}
                        </span>
                      </div>
                      <div className="mt-3 flex items-center justify-between text-xs text-slate-500">
                        <span>指标 {experiment.metric ?? '未设定'}</span>
                        <span>{experiment.winner ?? '未决出'}</span>
                      </div>
                    </div>
                  ))
                ) : (
                  <div className="rounded-2xl border border-dashed border-slate-200 bg-slate-50 p-6 text-sm text-slate-500">
                    没有可展示的实验记录。
                  </div>
                )}
              </div>
            </section>
          </div>
        </div>

        <div className="space-y-6">
          <section className="rounded-2xl border border-slate-200 bg-white shadow-sm">
            <div className="flex items-center justify-between border-b border-slate-200 px-6 py-4">
              <div>
                <p className="text-sm font-medium text-slate-500">选中任务</p>
                <h3 className="text-lg font-semibold text-slate-900">任务详情面板</h3>
              </div>
              <ChevronRight className="h-5 w-5 text-slate-400" />
            </div>
            <div className="p-6">
              {selectedJob ? (
                <div className="space-y-5">
                  <div className="rounded-2xl bg-slate-950 p-5 text-white">
                    <div className="flex items-center justify-between gap-4">
                      <div className="space-y-2">
                        <span className={`inline-flex rounded-full border px-2.5 py-1 text-xs ${pickStatus(selectedJob)}`}>
                          {selectedJob.status}
                        </span>
                        <h4 className="text-xl font-semibold leading-8">{selectedJob.goal}</h4>
                        <p className="text-sm leading-6 text-slate-300">预算 {selectedJob.budget_limit ?? 0} {selectedJob.currency} · 风险 {selectedJob.risk_level}</p>
                      </div>
                      <div className="rounded-2xl bg-white/10 p-3">
                        <Target className="h-6 w-6 text-cyan-300" />
                      </div>
                    </div>
                    <div className="mt-4 grid grid-cols-2 gap-3 text-sm">
                      <div className="rounded-xl bg-white/8 p-3">
                        <p className="text-slate-400">审批</p>
                        <p className="mt-1 font-medium">{selectedJob.approval_required ? '强制审批' : '自动审批'}</p>
                      </div>
                      <div className="rounded-xl bg-white/8 p-3">
                        <p className="text-slate-400">下次动作</p>
                        <p className="mt-1 font-medium">{formatDateTime(selectedJob.next_action_at)}</p>
                      </div>
                    </div>
                  </div>

                  <div className="grid gap-3 sm:grid-cols-2">
                    <div className="rounded-2xl border border-slate-200 bg-slate-50 p-4">
                      <p className="text-sm font-medium text-slate-500">受众画像</p>
                      <p className="mt-2 text-sm leading-6 text-slate-900">
                        {String(selectedJob.target_audience.persona ?? '未指定')} / {String(selectedJob.target_audience.lifecycle ?? '未指定')}
                      </p>
                    </div>
                    <div className="rounded-2xl border border-slate-200 bg-slate-50 p-4">
                      <p className="text-sm font-medium text-slate-500">通道偏好</p>
                      <p className="mt-2 text-sm leading-6 text-slate-900">
                        {selectedJob.channel_preferences.join(' / ') || '未设置'}
                      </p>
                    </div>
                    <div className="rounded-2xl border border-slate-200 bg-slate-50 p-4">
                      <p className="text-sm font-medium text-slate-500">策略蓝图</p>
                      <p className="mt-2 text-sm leading-6 text-slate-900">
                        {selectedBlueprint.join(' → ')}
                      </p>
                    </div>
                    <div className="rounded-2xl border border-slate-200 bg-slate-50 p-4">
                      <p className="text-sm font-medium text-slate-500">目标结果</p>
                      <p className="mt-2 text-sm leading-6 text-slate-900">
                        {String(selectedJob.target_audience.note ?? selectedJob.goal)}
                      </p>
                    </div>
                  </div>

                  <div className="grid grid-cols-2 gap-3">
                    <Button
                      onClick={() => executeMutation.mutate({ id: selectedJob.id })}
                      disabled={executeMutation.isPending}
                      variant="brand"
                    >
                      <Play className="h-4 w-4" />
                      立即执行
                    </Button>
                    <Button
                      onClick={() => setShowModal(true)}
                      variant="secondary"
                    >
                      <Plus className="h-4 w-4" />
                      复制创建
                    </Button>
                  </div>
                </div>
              ) : (
                <div className="rounded-2xl border border-dashed border-slate-200 bg-slate-50 p-6 text-sm leading-6 text-slate-500">
                  选择一个任务后，这里会展示对应的执行记录、审批链路、策略护栏和实验状态。
                </div>
              )}
            </div>
          </section>

          <section className="rounded-2xl border border-slate-200 bg-white shadow-sm">
            <div className="flex items-center justify-between border-b border-slate-200 px-6 py-4">
              <div>
                <p className="text-sm font-medium text-slate-500">策略建议</p>
                <h3 className="text-lg font-semibold text-slate-900">系统优化建议</h3>
              </div>
              <MessageSquareWarning className="h-5 w-5 text-amber-600" />
            </div>
            <div className="space-y-3 p-6">
              {data?.recommendations?.length ? (
                data.recommendations.map((item) => (
                  <div
                    key={item}
                    className="rounded-2xl border border-slate-200 bg-gradient-to-br from-slate-50 to-white p-4 text-sm leading-6 text-slate-700"
                  >
                    <div className="flex gap-3">
                      <ArrowUpRight className="mt-0.5 h-4 w-4 flex-none text-cyan-600" />
                      <span>{item}</span>
                    </div>
                  </div>
                ))
              ) : (
                <div className="rounded-2xl border border-dashed border-slate-200 bg-slate-50 p-6 text-sm text-slate-500">
                  当前没有建议，说明自动化链路暂时稳定。
                </div>
              )}
            </div>
          </section>
        </div>
      </div>

      {showModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/70 p-4 backdrop-blur-sm">
          <div className="max-h-[92vh] w-full max-w-3xl overflow-y-auto rounded-3xl border border-white/10 bg-white shadow-2xl">
            <div className="border-b border-slate-200 bg-slate-950 px-6 py-5 text-white">
              <div className="flex items-center justify-between gap-4">
                <div>
                  <p className="text-sm text-cyan-200">创建自动化任务</p>
                  <h3 className="mt-1 text-xl font-semibold">让系统自动编排增长动作</h3>
                </div>
                <Button
                  onClick={() => setShowModal(false)}
                  variant="ghost"
                  size="sm"
                  className="rounded-xl border border-white/10 bg-white/8 px-3 py-2 text-sm text-white hover:bg-white/12"
                >
                  关闭
                </Button>
              </div>
            </div>

            <form onSubmit={handleCreate} className="space-y-5 p-6">
              <div className="grid gap-4 lg:grid-cols-2">
                <label className="space-y-2">
                  <span className="text-sm font-medium text-slate-700">目标</span>
                  <input
                    name="goal"
                    required
                    placeholder="例如：激活 30 天沉默用户"
                    className="w-full rounded-2xl border border-slate-200 px-4 py-3 outline-none transition focus:border-cyan-500 focus:ring-4 focus:ring-cyan-100"
                  />
                </label>
                <label className="space-y-2">
                  <span className="text-sm font-medium text-slate-700">目标结果</span>
                  <input
                    name="outcome"
                    placeholder="例如：提升复购率 12%"
                    className="w-full rounded-2xl border border-slate-200 px-4 py-3 outline-none transition focus:border-cyan-500 focus:ring-4 focus:ring-cyan-100"
                  />
                </label>
              </div>

              <div className="grid gap-4 lg:grid-cols-3">
                <label className="space-y-2">
                  <span className="text-sm font-medium text-slate-700">受众画像</span>
                  <input
                    name="persona"
                    placeholder="高意向线索 / 老客户 / 新注册"
                    className="w-full rounded-2xl border border-slate-200 px-4 py-3 outline-none transition focus:border-cyan-500 focus:ring-4 focus:ring-cyan-100"
                  />
                </label>
                <label className="space-y-2">
                  <span className="text-sm font-medium text-slate-700">生命周期阶段</span>
                  <input
                    name="lifecycle"
                    placeholder="new / warm / churn"
                    className="w-full rounded-2xl border border-slate-200 px-4 py-3 outline-none transition focus:border-cyan-500 focus:ring-4 focus:ring-cyan-100"
                  />
                </label>
                <label className="space-y-2">
                  <span className="text-sm font-medium text-slate-700">区域</span>
                  <input
                    name="region"
                    placeholder="CN / APAC / Global"
                    className="w-full rounded-2xl border border-slate-200 px-4 py-3 outline-none transition focus:border-cyan-500 focus:ring-4 focus:ring-cyan-100"
                  />
                </label>
              </div>

              <div className="grid gap-4 lg:grid-cols-[1fr_220px_220px]">
                <label className="space-y-2">
                  <span className="text-sm font-medium text-slate-700">意图标签</span>
                  <input
                    name="intent"
                    placeholder="购买意向 / 留存 / 唤醒 / 续费"
                    className="w-full rounded-2xl border border-slate-200 px-4 py-3 outline-none transition focus:border-cyan-500 focus:ring-4 focus:ring-cyan-100"
                  />
                </label>
                <label className="space-y-2">
                  <span className="text-sm font-medium text-slate-700">预算</span>
                  <input
                    name="budget"
                    type="number"
                    min="0"
                    step="0.01"
                    placeholder="0"
                    className="w-full rounded-2xl border border-slate-200 px-4 py-3 outline-none transition focus:border-cyan-500 focus:ring-4 focus:ring-cyan-100"
                  />
                </label>
                <label className="space-y-2">
                  <span className="text-sm font-medium text-slate-700">币种</span>
                  <select
                    name="currency"
                    defaultValue="CNY"
                    className="w-full rounded-2xl border border-slate-200 px-4 py-3 outline-none transition focus:border-cyan-500 focus:ring-4 focus:ring-cyan-100"
                  >
                    <option value="CNY">CNY</option>
                    <option value="USD">USD</option>
                  </select>
                </label>
              </div>

              <div className="space-y-2">
                <span className="text-sm font-medium text-slate-700">渠道偏好</span>
                <div className="flex flex-wrap gap-2">
                  {CHANNEL_OPTIONS.map((channel) => {
                    const active = selectedChannels.includes(channel.id)
                    return (
                      <Button
                        key={channel.id}
                        type="button"
                        onClick={() => {
                          setSelectedChannels((current) =>
                            current.includes(channel.id)
                              ? current.filter((item) => item !== channel.id)
                              : [...current, channel.id],
                          )
                        }}
                        variant="secondary"
                        size="sm"
                        className={`rounded-full px-3 py-1.5 ${
                          active
                            ? 'border-cyan-500 bg-cyan-500 text-white'
                            : 'border-slate-200 bg-white text-slate-600 hover:bg-slate-50'
                        }`}
                      >
                        {channel.label}
                      </Button>
                    )
                  })}
                </div>
              </div>

              <div className="grid gap-4 lg:grid-cols-2">
                <label className="space-y-2">
                  <span className="text-sm font-medium text-slate-700">审批策略</span>
                  <select
                    name="approval_required"
                    defaultValue="yes"
                    className="w-full rounded-2xl border border-slate-200 px-4 py-3 outline-none transition focus:border-cyan-500 focus:ring-4 focus:ring-cyan-100"
                  >
                    <option value="no">自动判断</option>
                    <option value="yes">强制审批</option>
                  </select>
                </label>
                <label className="space-y-2">
                  <span className="text-sm font-medium text-slate-700">备注</span>
                  <input
                    name="note"
                    placeholder="例如：优先低风险动作，避免高触达频次"
                    className="w-full rounded-2xl border border-slate-200 px-4 py-3 outline-none transition focus:border-cyan-500 focus:ring-4 focus:ring-cyan-100"
                  />
                </label>
              </div>

              <div className="flex flex-wrap items-center justify-between gap-3 rounded-2xl bg-slate-50 px-4 py-3">
                <div className="flex items-center gap-2 text-sm text-slate-600">
                  <CheckCircle2 className="h-4 w-4 text-emerald-500" />
                  创建后系统将自动生成任务图、风控规则和实验建议。
                </div>
                <div className="flex items-center gap-2 text-sm text-slate-500">
                  <span className="rounded-full bg-white px-3 py-1">{selectedChannels.length} 个渠道</span>
                  <span className="rounded-full bg-white px-3 py-1">企业模式</span>
                </div>
              </div>

              <div className="flex justify-end gap-3 pt-2">
                <Button
                  type="button"
                  onClick={() => setShowModal(false)}
                  variant="secondary"
                >
                  取消
                </Button>
                <Button
                  type="submit"
                  disabled={createMutation.isPending}
                  variant="brand"
                >
                  <Rocket className="h-4 w-4" />
                  {createMutation.isPending ? '创建中...' : '创建并生成策略'}
                </Button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  )
}
