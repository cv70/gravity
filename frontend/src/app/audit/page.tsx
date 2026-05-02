import { useMemo, useState } from 'react'
import { Link } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { Archive, Clock3, Download, Filter, ChevronLeft, ChevronRight } from 'lucide-react'

import { governanceService } from '@/services/governance'
import { Button } from '@/components/ui/button'

export function AuditPage() {
  const [page, setPage] = useState(1)
  const limit = 20
  const [action, setAction] = useState('')
  const [targetType, setTargetType] = useState('')
  const [startAt, setStartAt] = useState('')
  const [endAt, setEndAt] = useState('')

  const { data, isLoading } = useQuery({
    queryKey: ['governance', 'audit', page, action, targetType, startAt, endAt],
    queryFn: () =>
      governanceService.auditLogs({
        page,
        limit,
        action: action || undefined,
        target_type: targetType || undefined,
        start_at: startAt ? new Date(startAt).toISOString() : undefined,
        end_at: endAt ? new Date(endAt).toISOString() : undefined,
      }),
  })

  const stats = useMemo(() => {
    const items = data?.data ?? []
    return [
      { label: '日志条数', value: items.length },
      { label: '最近操作', value: items[0] ? new Date(items[0].created_at).toLocaleDateString('zh-CN') : '—' },
      { label: '动作类型', value: new Set(items.map((item) => item.action)).size },
      { label: '对象类型', value: new Set(items.map((item) => item.target_type).filter(Boolean)).size },
    ]
  }, [data?.data])

  const totalPages = Math.max(1, Math.ceil((data?.total ?? 0) / limit))
  const rows = data?.data ?? []
  const hasFilters = Boolean(action || targetType || startAt || endAt)
  const activeFilters = useMemo(
    () =>
      [
        action ? { key: 'action', label: `动作：${action}` } : null,
        targetType ? { key: 'targetType', label: `对象：${targetType}` } : null,
        startAt ? { key: 'startAt', label: `开始：${startAt}` } : null,
        endAt ? { key: 'endAt', label: `结束：${endAt}` } : null,
      ].filter(Boolean) as Array<{ key: string; label: string }>,
    [action, endAt, startAt, targetType],
  )

  const resetToFirstPage = () => setPage(1)
  const clearFilters = () => {
    setAction('')
    setTargetType('')
    setStartAt('')
    setEndAt('')
    setPage(1)
  }

  const exportCurrent = () => {
    const header = ['created_at', 'action', 'target_type', 'target_id', 'metadata']
    const escape = (value: unknown) => {
      const text = typeof value === 'string' ? value : JSON.stringify(value ?? '')
      return `"${text.replaceAll('"', '""')}"`
    }
    const csv = [
      header.join(','),
      ...rows.map((row) =>
        [
          escape(row.created_at),
          escape(row.action),
          escape(row.target_type || ''),
          escape(row.target_id || ''),
          escape(row.metadata),
        ].join(','),
      ),
    ].join('\n')
    const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `audit-logs-page-${page}.csv`
    a.click()
    URL.revokeObjectURL(url)
  }

  return (
    <div className="space-y-6">
      <div className="relative overflow-hidden rounded-3xl bg-slate-950 text-white shadow-xl border border-slate-800">
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_top_right,rgba(34,211,238,0.22),transparent_30%),radial-gradient(circle_at_bottom_left,rgba(59,130,246,0.18),transparent_28%)]" />
        <div className="relative p-6 lg:p-8">
          <div className="inline-flex items-center gap-2 rounded-full bg-white/10 px-3 py-1 text-xs text-cyan-100">
            <Archive className="h-4 w-4" />
            审计中心
          </div>
          <h1 className="mt-4 text-3xl font-bold tracking-tight">追踪所有关键动作的责任链与时间线</h1>
          <p className="mt-3 max-w-3xl text-sm lg:text-base text-slate-300">
            审计页只关注事件事实和责任追踪，适合管理层、合规和实施团队独立查看。
          </p>
          <div className="mt-6 flex flex-wrap gap-3">
            <Link
              to="/governance"
              className="inline-flex items-center gap-2 rounded-xl bg-cyan-400 px-4 py-2.5 text-sm font-semibold text-slate-950 shadow-lg shadow-cyan-500/20 transition hover:bg-cyan-300"
            >
              返回治理总览
            </Link>
            <Link
              to="/approvals"
              className="inline-flex items-center gap-2 rounded-xl border border-white/15 bg-white/5 px-4 py-2.5 text-sm font-medium text-white transition hover:bg-white/10"
            >
              前往审批中心
            </Link>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-4">
        {stats.map((stat) => (
          <div
            key={stat.label}
            className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm transition hover:-translate-y-0.5 hover:shadow-md"
          >
            <p className="text-3xl font-bold text-slate-900">{stat.value}</p>
            <p className="mt-1 text-sm text-slate-500">{stat.label}</p>
          </div>
        ))}
      </div>

      <div className="rounded-2xl border border-slate-200 bg-white shadow-sm">
        <div className="flex flex-col gap-3 border-b px-6 py-4 lg:flex-row lg:items-center lg:justify-between">
          <div>
            <h2 className="text-lg font-semibold text-slate-900">审计时间线</h2>
            <p className="text-sm text-slate-500">所有关键操作都应该能在这里被复盘</p>
          </div>
          <div className="grid gap-3 md:grid-cols-2 lg:grid-cols-5">
            <div className="inline-flex items-center gap-2 rounded-xl border border-slate-200 bg-slate-50 px-3 py-2">
              <Filter className="h-4 w-4 text-slate-400" />
              <input
                value={action}
                onChange={(e) => {
                  setAction(e.target.value)
                  resetToFirstPage()
                }}
                placeholder="动作"
                className="w-24 bg-transparent text-sm outline-none"
              />
            </div>
            <input
              value={targetType}
              onChange={(e) => {
                setTargetType(e.target.value)
                resetToFirstPage()
              }}
              placeholder="对象类型"
              className="rounded-xl border border-slate-200 bg-white px-3 py-2.5 text-sm outline-none transition focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
            />
            <input
              type="date"
              value={startAt}
              onChange={(e) => {
                setStartAt(e.target.value)
                resetToFirstPage()
              }}
              className="rounded-xl border border-slate-200 bg-white px-3 py-2.5 text-sm outline-none transition focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
            />
            <input
              type="date"
              value={endAt}
              onChange={(e) => {
                setEndAt(e.target.value)
                resetToFirstPage()
              }}
              className="rounded-xl border border-slate-200 bg-white px-3 py-2.5 text-sm outline-none transition focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
            />
            <Button
              onClick={exportCurrent}
              disabled={rows.length === 0}
              variant="secondary"
            >
              <Download className="h-4 w-4" />
              导出当前页 CSV
            </Button>
            <Button
              onClick={clearFilters}
              disabled={!hasFilters}
              variant="secondary"
            >
              清空筛选条件
            </Button>
          </div>
        </div>

        {activeFilters.length > 0 && (
          <div className="border-b border-slate-100 px-6 py-3">
            <div className="flex flex-wrap items-center gap-2">
              <span className="text-xs font-semibold uppercase tracking-wide text-slate-400">当前筛选条件</span>
              {activeFilters.map((filter) => (
                <span key={filter.key} className="rounded-full bg-slate-100 px-3 py-1 text-xs font-medium text-slate-600">
                  {filter.label}
                </span>
              ))}
            </div>
          </div>
        )}

        <div className="space-y-3 p-6">
          {isLoading ? (
            <div className="py-10 text-center text-slate-500">正在加载审计日志...</div>
          ) : rows.length > 0 ? (
            rows.map((log) => (
              <div
                key={log.id}
                className="flex gap-4 rounded-2xl border border-slate-200 bg-slate-50 p-4 transition hover:-translate-y-0.5 hover:border-cyan-200 hover:shadow-md"
              >
                <div className="mt-1 rounded-full bg-cyan-100 p-2 text-cyan-700">
                  <Clock3 className="h-4 w-4" />
                </div>
                <div className="min-w-0 flex-1">
                  <div className="flex flex-col gap-2 lg:flex-row lg:items-center lg:justify-between">
                    <p className="font-medium text-slate-900">{log.action}</p>
                    <span className="text-xs text-slate-500">
                      {new Date(log.created_at).toLocaleString('zh-CN', { hour12: false })}
                    </span>
                  </div>
                  <p className="mt-1 text-sm text-slate-500">
                    {log.target_type ? `${log.target_type} · ` : ''}
                    {log.target_id || '—'}
                  </p>
                  <pre className="mt-3 overflow-auto rounded-xl bg-white p-3 text-xs leading-6 text-slate-600">
                    {JSON.stringify(log.metadata, null, 2)}
                  </pre>
                </div>
              </div>
            ))
          ) : (
            <div className="rounded-2xl bg-slate-50 p-10 text-center text-slate-500">
              <p>当前没有审计日志。</p>
              {hasFilters && (
                <Button
                  onClick={clearFilters}
                  variant="secondary"
                  className="mt-4"
                >
                  清空并重新查询
                </Button>
              )}
            </div>
          )}
          <div className="flex items-center justify-between border-t border-slate-200 px-6 py-4">
            <p className="text-sm text-slate-500">
              第 {page} / {totalPages} 页
            </p>
            <div className="flex items-center gap-2">
              <Button
                onClick={() => setPage((current) => Math.max(1, current - 1))}
                disabled={page === 1}
                variant="secondary"
                size="sm"
              >
                <ChevronLeft className="h-4 w-4" />
                上一页
              </Button>
              <Button
                onClick={() => setPage((current) => current + 1)}
                disabled={page >= totalPages}
                variant="secondary"
                size="sm"
              >
                下一页
                <ChevronRight className="h-4 w-4" />
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
