import { useMemo, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { BadgeCheck, Filter, ShieldCheck, XCircle } from 'lucide-react'

import { governanceService } from '@/services/governance'
import { Button } from '@/components/ui/button'
import type { Approval } from '@/types'

type StatusFilter = 'all' | 'pending' | 'approved' | 'rejected'

const NOTE_TEMPLATES = [
  '符合当前策略，可继续执行。',
  '请补充执行说明后再审批。',
  '本次动作风险可控，允许放行。',
  '超出当前风控阈值，建议调整后重提。',
]

function tone(status: string) {
  switch (status) {
    case 'approved':
      return 'bg-emerald-100 text-emerald-700'
    case 'rejected':
      return 'bg-rose-100 text-rose-700'
    default:
      return 'bg-amber-100 text-amber-700'
  }
}

export function ApprovalsPage() {
  const queryClient = useQueryClient()
  const [status, setStatus] = useState<StatusFilter>('all')
  const [objectType, setObjectType] = useState('')
  const [selectedIds, setSelectedIds] = useState<string[]>([])
  const [reviewNote, setReviewNote] = useState(NOTE_TEMPLATES[0])

  const { data, isLoading } = useQuery({
    queryKey: ['governance', 'approvals', status, objectType],
    queryFn: () =>
      governanceService.approvals({
        page: 1,
        limit: 100,
        status: status === 'all' ? undefined : status,
        object_type: objectType || undefined,
      }),
  })

  const reviewMutation = useMutation({
    mutationFn: ({ id, approved, note }: { id: string; approved: boolean; note?: string }) =>
      governanceService.reviewApproval(id, approved, note),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['governance', 'approvals'] })
    },
  })

  const approvals = useMemo(() => data?.data ?? [], [data?.data])
  const selectedCount = selectedIds.length

  const stats = useMemo(() => {
    const items = approvals
    return [
      { label: '总审批', value: items.length },
      { label: '待处理', value: items.filter((item) => item.status === 'pending').length },
      { label: '已通过', value: items.filter((item) => item.status === 'approved').length },
      { label: '已驳回', value: items.filter((item) => item.status === 'rejected').length },
    ]
  }, [approvals])

  const handleReview = async (approval: Approval, approved: boolean) => {
    const note = reviewNote.trim() || NOTE_TEMPLATES[0]
    await reviewMutation.mutateAsync({ id: approval.id, approved, note })
  }

  const toggleSelected = (id: string) => {
    setSelectedIds((current) =>
      current.includes(id) ? current.filter((item) => item !== id) : [...current, id],
    )
  }

  const toggleSelectAll = () => {
    if (selectedIds.length === approvals.length) {
      setSelectedIds([])
      return
    }
    setSelectedIds(approvals.map((item) => item.id))
  }

  const batchReview = async (approved: boolean) => {
    if (selectedIds.length === 0) return
    const note = reviewNote.trim() || NOTE_TEMPLATES[0]
    await Promise.allSettled(
      selectedIds.map((id) => reviewMutation.mutateAsync({ id, approved, note })),
    )
    setSelectedIds([])
  }

  const handleStatusChange = (value: StatusFilter) => {
    setStatus(value)
    setSelectedIds([])
  }

  const handleObjectTypeChange = (value: string) => {
    setObjectType(value)
    setSelectedIds([])
  }

  const clearSelection = () => setSelectedIds([])

  return (
    <div className="space-y-6">
      <div className="relative overflow-hidden rounded-3xl bg-slate-950 text-white shadow-xl border border-slate-800">
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_top_right,rgba(34,211,238,0.22),transparent_30%),radial-gradient(circle_at_bottom_left,rgba(59,130,246,0.18),transparent_28%)]" />
        <div className="relative p-6 lg:p-8">
          <div className="inline-flex items-center gap-2 rounded-full bg-white/10 px-3 py-1 text-xs text-cyan-100">
            <ShieldCheck className="h-4 w-4" />
            审批中心
          </div>
          <h1 className="mt-4 text-3xl font-bold tracking-tight">统一处理高风险动作的放行与驳回</h1>
          <p className="mt-3 max-w-3xl text-sm lg:text-base text-slate-300">
            把审批从总览里拆出来后，运营、风控和管理层可以更清楚地分工协作。
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-4">
        {stats.map((stat) => (
          <div key={stat.label} className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
            <p className="text-3xl font-bold text-slate-900">{stat.value}</p>
            <p className="mt-1 text-sm text-slate-500">{stat.label}</p>
          </div>
        ))}
      </div>

      <div className="rounded-2xl border border-slate-200 bg-white shadow-sm">
        <div className="flex flex-col gap-3 border-b px-6 py-4 lg:flex-row lg:items-center lg:justify-between">
          <div>
            <h2 className="text-lg font-semibold text-slate-900">审批列表</h2>
            <p className="text-sm text-slate-500">审批项通常来自自动化工作流中的高风险动作</p>
          </div>
          <div className="grid gap-3 xl:grid-cols-[1.05fr_1fr_auto] xl:items-start">
            <div className="space-y-3">
              <div className="flex flex-wrap items-center gap-3">
                <div className="inline-flex items-center gap-2 rounded-xl border border-slate-200 bg-slate-50 px-3 py-2">
                  <Filter className="h-4 w-4 text-slate-400" />
                  <select
                    value={status}
                    onChange={(e) => handleStatusChange(e.target.value as StatusFilter)}
                    className="bg-transparent text-sm outline-none"
                  >
                    <option value="all">全部状态</option>
                    <option value="pending">待处理</option>
                    <option value="approved">已通过</option>
                    <option value="rejected">已驳回</option>
                  </select>
                </div>
                <input
                  value={objectType}
                  onChange={(e) => handleObjectTypeChange(e.target.value)}
                  placeholder="按对象类型筛选，例如 action"
                  className="min-w-0 flex-1 rounded-xl border border-slate-200 bg-white px-3 py-2.5 text-sm outline-none transition focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                />
              </div>
              <div className="flex flex-wrap gap-2">
                <Button
                  onClick={clearSelection}
                  disabled={selectedCount === 0}
                  variant="secondary"
                >
                  清除选择
                </Button>
                <Button
                  onClick={toggleSelectAll}
                  variant="secondary"
                >
                  {selectedCount > 0 && selectedCount === approvals.length ? '取消全选' : '全选当前页'}
                </Button>
                <span className="inline-flex items-center rounded-full bg-slate-100 px-3 py-2 text-xs font-medium text-slate-600">
                  已选 {selectedCount} / {approvals.length}
                </span>
              </div>
            </div>

            <div className="rounded-2xl border border-slate-200 bg-slate-50 p-3">
              <label className="mb-2 block text-xs font-semibold uppercase tracking-wide text-slate-500">
                审批备注
              </label>
              <textarea
                value={reviewNote}
                onChange={(e) => setReviewNote(e.target.value)}
                rows={2}
                className="w-full rounded-lg border border-slate-200 bg-white px-3 py-2 text-sm outline-none transition focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                placeholder="填写通过或驳回的统一说明"
              />
              <div className="mt-3 flex flex-wrap gap-2">
                {NOTE_TEMPLATES.map((template) => (
                  <Button
                    key={template}
                    onClick={() => setReviewNote(template)}
                    variant="secondary"
                    size="sm"
                    className={`rounded-full px-3 py-2 text-xs font-medium transition ${
                      reviewNote === template
                        ? 'bg-slate-950 text-white'
                        : 'border border-slate-200 bg-white text-slate-600 hover:bg-slate-50'
                    }`}
                  >
                    {template}
                  </Button>
                ))}
              </div>
              <p className="mt-2 text-xs leading-5 text-slate-500">
                备注会同步用于单条审批和批量审批，减少重复输入。
              </p>
            </div>

            <div className="flex flex-row gap-3 xl:flex-col xl:justify-start">
              <Button
                onClick={() => batchReview(true)}
                disabled={selectedCount === 0}
                variant="brand"
              >
                <BadgeCheck className="h-4 w-4" />
                批量通过 {selectedCount > 0 ? `(${selectedCount})` : ''}
              </Button>
              <Button
                onClick={() => batchReview(false)}
                disabled={selectedCount === 0}
                variant="destructive"
              >
                <XCircle className="h-4 w-4" />
                批量驳回 {selectedCount > 0 ? `(${selectedCount})` : ''}
              </Button>
            </div>
          </div>
        </div>

        <div className="space-y-4 p-6">
          {selectedCount > 0 && (
            <div className="rounded-2xl border border-cyan-100 bg-cyan-50 px-4 py-3 text-sm text-cyan-900">
              已选择 {selectedCount} 个审批，可使用当前备注统一处理。
            </div>
          )}
          {isLoading ? (
            <div className="py-10 text-center text-slate-500">加载中...</div>
          ) : approvals.length > 0 ? (
            approvals.map((approval) => (
              <div key={approval.id} className="rounded-2xl border border-slate-200 bg-slate-50 p-5">
                <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
                  <label className="flex items-start gap-3">
                    <input
                      type="checkbox"
                      checked={selectedIds.includes(approval.id)}
                      onChange={() => toggleSelected(approval.id)}
                      className="mt-1 h-4 w-4 rounded border-slate-300 text-cyan-600 focus:ring-cyan-500"
                    />
                    <div className="space-y-2">
                      <div className="flex items-center gap-2">
                        <p className="font-semibold text-slate-900">{approval.object_type}</p>
                        <span className={`rounded-full px-2.5 py-1 text-xs font-medium ${tone(approval.status)}`}>
                          {approval.status}
                        </span>
                      </div>
                      <p className="text-sm text-slate-500">对象: {approval.object_id}</p>
                      <p className="text-sm leading-6 text-slate-600">{approval.reason || '暂无原因描述'}</p>
                    </div>
                  </label>
                  {approval.status === 'pending' ? (
                    <div className="flex flex-col gap-2 sm:flex-row lg:flex-col">
                      <Button
                        onClick={() => handleReview(approval, true)}
                        variant="brand"
                        size="sm"
                      >
                        <BadgeCheck className="h-4 w-4" />
                        通过
                      </Button>
                      <Button
                        onClick={() => handleReview(approval, false)}
                        variant="destructive"
                        size="sm"
                      >
                        <XCircle className="h-4 w-4" />
                        驳回
                      </Button>
                    </div>
                  ) : (
                    <div className="rounded-xl bg-white px-4 py-3 text-sm text-slate-500">
                      已由 {approval.approved_by || '系统'} 处理
                    </div>
                  )}
                </div>
              </div>
            ))
          ) : (
            <div className="rounded-2xl bg-slate-50 p-10 text-center text-slate-500">暂无审批项。</div>
          )}
        </div>
      </div>
    </div>
  )
}
