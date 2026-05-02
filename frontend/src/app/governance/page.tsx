import { useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { AlertTriangle, Archive, BadgeCheck, Layers3, Shield, Sparkles, TerminalSquare } from 'lucide-react'

import { governanceService } from '@/services/governance'
import type { ApprovalListResponse, AuditLogListResponse, InsightListResponse, SegmentListResponse } from '@/types'

function severityClass(severity: string) {
  switch (severity) {
    case 'high':
      return 'bg-rose-100 text-rose-700'
    case 'medium':
      return 'bg-amber-100 text-amber-700'
    default:
      return 'bg-emerald-100 text-emerald-700'
  }
}

export function GovernancePage() {
  const { data: segments } = useQuery<SegmentListResponse>({
    queryKey: ['governance', 'segments'],
    queryFn: () => governanceService.segments({ page: 1, limit: 100 }),
  })

  const { data: approvals } = useQuery<ApprovalListResponse>({
    queryKey: ['governance', 'approvals'],
    queryFn: () => governanceService.approvals({ page: 1, limit: 100 }),
  })

  const { data: auditLogs } = useQuery<AuditLogListResponse>({
    queryKey: ['governance', 'audit'],
    queryFn: () => governanceService.auditLogs({ page: 1, limit: 100 }),
  })

  const { data: recommendations } = useQuery<InsightListResponse>({
    queryKey: ['governance', 'recommendations'],
    queryFn: () => governanceService.recommendations(),
  })

  const { data: anomalies } = useQuery<InsightListResponse>({
    queryKey: ['governance', 'anomalies'],
    queryFn: () => governanceService.anomalies(),
  })

  const { data: opportunities } = useQuery<InsightListResponse>({
    queryKey: ['governance', 'opportunities'],
    queryFn: () => governanceService.opportunities(),
  })

  const cards = useMemo(
    () => [
      { label: '动态分群', value: segments?.data.length ?? 0, icon: Layers3, hint: '统一的人群圈选与预览' },
      { label: '审批队列', value: approvals?.data.filter((item) => item.status === 'pending').length ?? 0, icon: Shield, hint: '高风险动作放行' },
      { label: '审计日志', value: auditLogs?.data.length ?? 0, icon: Archive, hint: '所有关键操作留痕' },
      { label: '策略建议', value: recommendations?.data.length ?? 0, icon: Sparkles, hint: '自动化优化提示' },
    ],
    [approvals?.data, auditLogs?.data, recommendations?.data, segments?.data.length],
  )

  const quickLinks = [
    {
      title: '分群管理',
      href: '/segments',
      icon: Layers3,
      description: '创建欢迎流、召回流和高意向线索分群',
    },
    {
      title: '审批管理',
      href: '/approvals',
      icon: Shield,
      description: '集中处理高风险动作的放行与驳回',
    },
    {
      title: '审计日志',
      href: '/audit',
      icon: Archive,
      description: '复盘所有关键操作和责任链',
    },
  ]

  return (
    <div className="space-y-8">
      <div className="relative overflow-hidden rounded-3xl bg-slate-950 text-white shadow-xl border border-slate-800">
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_top_right,rgba(34,211,238,0.22),transparent_30%),radial-gradient(circle_at_bottom_left,rgba(59,130,246,0.18),transparent_28%)]" />
        <div className="relative p-6 lg:p-8">
          <div className="inline-flex items-center gap-2 rounded-full bg-white/10 px-3 py-1 text-xs text-cyan-100">
            <TerminalSquare className="h-4 w-4" />
            治理总览
          </div>
          <h1 className="mt-4 text-3xl font-bold tracking-tight">分群、审批、审计与洞察统一纳管</h1>
          <p className="mt-3 max-w-3xl text-sm lg:text-base text-slate-300">
            将治理能力收拢在一个控制台里，让运营、风控和实施团队都能快速定位自己需要的内容。
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-4">
        {cards.map((card) => (
          <div
            key={card.label}
            className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm transition hover:-translate-y-0.5 hover:shadow-md"
          >
            <div className="flex items-start justify-between">
              <div>
                <p className="text-3xl font-bold text-slate-900">{card.value}</p>
                <p className="mt-1 text-sm text-slate-500">{card.label}</p>
              </div>
              <div className="rounded-xl bg-cyan-50 p-2 text-cyan-700">
                <card.icon className="h-4 w-4" />
              </div>
            </div>
            <p className="mt-3 text-xs text-slate-400">{card.hint}</p>
          </div>
        ))}
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
        {quickLinks.map((item) => (
          <Link
            key={item.title}
            to={item.href}
            className="group rounded-2xl border border-slate-200 bg-white p-5 shadow-sm transition hover:-translate-y-0.5 hover:border-cyan-200 hover:shadow-md"
          >
            <div className="flex items-start justify-between gap-3">
              <div>
                <div className="inline-flex items-center gap-2 rounded-full bg-cyan-50 px-3 py-1 text-xs font-medium text-cyan-700">
                  <item.icon className="h-4 w-4" />
                  快速进入
                </div>
                <h3 className="mt-3 text-base font-semibold text-slate-900">{item.title}</h3>
                <p className="mt-1 text-sm leading-6 text-slate-500">{item.description}</p>
              </div>
              <Sparkles className="h-5 w-5 text-slate-300 transition group-hover:text-cyan-500" />
            </div>
          </Link>
        ))}
      </div>

      <div className="grid grid-cols-1 xl:grid-cols-3 gap-6">
        <section className="rounded-2xl border border-slate-200 bg-white shadow-sm xl:col-span-2">
          <div className="border-b px-6 py-4">
            <h2 className="text-lg font-semibold text-slate-900">分群与审批</h2>
            <p className="text-sm text-slate-500">策略执行前的治理层状态</p>
          </div>
          <div className="grid gap-6 p-6 lg:grid-cols-2">
            <div>
              <h3 className="mb-3 text-sm font-semibold text-slate-700">动态分群</h3>
              <div className="space-y-3">
                {(segments?.data ?? []).slice(0, 6).map((segment) => (
                  <div key={segment.id} className="rounded-xl border border-slate-200 bg-slate-50 p-4">
                    <div className="flex items-center justify-between gap-3">
                      <div>
                        <p className="font-medium text-slate-900">{segment.name}</p>
                        <p className="text-sm text-slate-500">{segment.status} · {segment.is_dynamic ? '动态' : '静态'}</p>
                      </div>
                      <span className="rounded-full bg-cyan-100 px-2.5 py-1 text-xs font-medium text-cyan-700">
                        Segment
                      </span>
                    </div>
                  </div>
                ))}
                {!segments?.data.length && (
                  <p className="rounded-xl bg-slate-50 p-4 text-sm text-slate-500">当前没有分群，建议先从欢迎流模板开始。</p>
                )}
              </div>
            </div>

            <div>
              <h3 className="mb-3 text-sm font-semibold text-slate-700">审批队列</h3>
              <div className="space-y-3">
                {(approvals?.data ?? []).slice(0, 6).map((approval) => (
                  <div key={approval.id} className="rounded-xl border border-slate-200 bg-slate-50 p-4">
                    <div className="flex items-start justify-between gap-3">
                      <div>
                        <p className="font-medium text-slate-900">{approval.object_type}</p>
                        <p className="text-sm text-slate-500">{approval.reason || approval.object_id}</p>
                      </div>
                      <span className={`rounded-full px-2.5 py-1 text-xs font-medium ${severityClass(approval.status === 'approved' ? 'low' : approval.status === 'rejected' ? 'high' : 'medium')}`}>
                        {approval.status}
                      </span>
                    </div>
                  </div>
                ))}
                {!approvals?.data.length && (
                  <p className="rounded-xl bg-slate-50 p-4 text-sm text-slate-500">当前没有审批项需要处理。</p>
                )}
              </div>
            </div>
          </div>
        </section>

        <section className="rounded-2xl border border-slate-200 bg-white shadow-sm">
          <div className="border-b px-6 py-4">
            <h2 className="text-lg font-semibold text-slate-900">审计日志</h2>
            <p className="text-sm text-slate-500">追踪关键操作与风险事件</p>
          </div>
          <div className="p-6 space-y-3">
            {(auditLogs?.data ?? []).slice(0, 8).map((item) => (
              <div key={item.id} className="rounded-xl bg-slate-50 p-4">
                <div className="flex items-center justify-between gap-3">
                  <p className="text-sm font-medium text-slate-900">{item.action}</p>
                  <span className="text-xs text-slate-500">
                    {new Date(item.created_at).toLocaleString('zh-CN', { hour12: false })}
                  </span>
                </div>
                <p className="mt-1 text-xs text-slate-500">
                  {item.target_type ? `${item.target_type} · ` : ''}
                  {item.target_id || '—'}
                </p>
              </div>
            ))}
            {!auditLogs?.data.length && (
              <p className="rounded-xl bg-slate-50 p-4 text-sm text-slate-500">当前没有审计日志。</p>
            )}
          </div>
        </section>
      </div>

      <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
        <section className="rounded-2xl border border-slate-200 bg-white shadow-sm">
          <div className="border-b px-6 py-4">
            <h2 className="text-lg font-semibold text-slate-900">策略建议</h2>
            <p className="text-sm text-slate-500">自动化中枢返回的改进建议</p>
          </div>
          <div className="p-6 space-y-3">
            {(recommendations?.data ?? []).map((item) => (
              <div key={item.title} className="rounded-xl border border-cyan-100 bg-cyan-50/70 p-4">
                <div className="flex items-start justify-between gap-3">
                  <div>
                    <p className="font-medium text-slate-900">{item.title}</p>
                    <p className="mt-1 text-sm text-slate-600 leading-6">{item.description}</p>
                  </div>
                  <span className={`rounded-full px-2.5 py-1 text-xs font-medium ${severityClass(item.severity)}`}>
                    {item.severity}
                  </span>
                </div>
              </div>
            ))}
            {!recommendations?.data.length && (
              <p className="rounded-xl bg-slate-50 p-4 text-sm text-slate-500">当前没有策略建议。</p>
            )}
          </div>
        </section>

        <section className="rounded-2xl border border-slate-200 bg-white shadow-sm">
          <div className="border-b px-6 py-4">
            <h2 className="text-lg font-semibold text-slate-900">异常与机会</h2>
            <p className="text-sm text-slate-500">帮助运营团队快速定位问题与增长空间</p>
          </div>
          <div className="grid gap-6 p-6 lg:grid-cols-2">
            <div>
              <div className="mb-3 flex items-center gap-2 text-sm font-semibold text-slate-700">
                <AlertTriangle className="h-4 w-4 text-amber-500" />
                异常
              </div>
              <div className="space-y-3">
                {(anomalies?.data ?? []).map((item) => (
                  <div key={item.title} className="rounded-xl bg-amber-50 p-4">
                    <p className="font-medium text-slate-900">{item.title}</p>
                    <p className="mt-1 text-sm text-slate-600 leading-6">{item.description}</p>
                  </div>
                ))}
                {!anomalies?.data.length && (
                  <p className="rounded-xl bg-slate-50 p-4 text-sm text-slate-500">当前没有异常事件。</p>
                )}
              </div>
            </div>
            <div>
              <div className="mb-3 flex items-center gap-2 text-sm font-semibold text-slate-700">
                <BadgeCheck className="h-4 w-4 text-emerald-500" />
                机会
              </div>
              <div className="space-y-3">
                {(opportunities?.data ?? []).map((item) => (
                  <div key={item.title} className="rounded-xl bg-emerald-50 p-4">
                    <p className="font-medium text-slate-900">{item.title}</p>
                    <p className="mt-1 text-sm text-slate-600 leading-6">{item.description}</p>
                  </div>
                ))}
                {!opportunities?.data.length && (
                  <p className="rounded-xl bg-slate-50 p-4 text-sm text-slate-500">当前没有增长机会建议。</p>
                )}
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>
  )
}
