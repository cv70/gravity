import { useQuery } from '@tanstack/react-query'
import { automationService } from '@/services/automation'
import { analyticsService } from '@/services/analytics'
import { campaignsService } from '@/services/campaigns'
import { Bot, Users, Megaphone, Target, TrendingUp, ShieldAlert, Wand2, Gauge, Workflow } from 'lucide-react'

export function DashboardPage() {
  const { data: dashboard, isLoading } = useQuery({
    queryKey: ['analytics', 'dashboard'],
    queryFn: analyticsService.getDashboard,
  })

  const { data: automation } = useQuery({
    queryKey: ['automation', 'dashboard'],
    queryFn: automationService.getDashboard,
  })

  const { data: campaignsData } = useQuery({
    queryKey: ['campaigns'],
    queryFn: campaignsService.list,
  })

  if (isLoading) {
    return <div className="animate-pulse">加载中...</div>
  }

  const stats = [
    { name: '总联系人', value: dashboard?.total_contacts ?? 0, icon: Users },
    { name: '活跃活动', value: campaignsData?.data?.filter((c) => c.status === 'active').length ?? 0, icon: Megaphone },
    { name: '总转化数', value: dashboard?.total_conversions ?? 0, icon: Target },
    { name: '转化率', value: `${((dashboard?.conversion_rate ?? 0) * 100).toFixed(1)}%`, icon: TrendingUp },
  ]

  const automationStats = [
    { name: '自动化覆盖率', value: `${((automation?.overview.automation_coverage ?? 0) * 100).toFixed(0)}%`, icon: Bot },
    { name: '人工介入率', value: `${((automation?.overview.human_intervention_rate ?? 0) * 100).toFixed(0)}%`, icon: ShieldAlert },
    { name: '待审批动作', value: automation?.overview.pending_approvals ?? 0, icon: Workflow },
    { name: '拦截动作', value: automation?.overview.blocked_actions ?? 0, icon: Gauge },
  ]

  return (
    <div className="space-y-8 relative">
      <div className="absolute inset-x-0 -top-8 h-40 bg-gradient-to-r from-cyan-100/60 via-sky-50/40 to-transparent blur-3xl pointer-events-none" />
      <div className="relative">
        <h1 className="text-2xl font-bold text-gray-900">运营驾驶舱</h1>
        <p className="text-gray-500 mt-1">实时查看投放、自动化决策和风控拦截的闭环状态</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {stats.map((stat) => (
          <div key={stat.name} className="bg-white/90 backdrop-blur p-6 rounded-2xl border border-gray-200 shadow-sm">
            <div className="flex items-center justify-between">
              <div className="p-2 bg-brand-50 rounded-lg">
                <stat.icon className="h-5 w-5 text-brand-600" />
              </div>
            </div>
            <div className="mt-4">
              <p className="text-2xl font-bold text-gray-900">{stat.value}</p>
              <p className="text-sm text-gray-500">{stat.name}</p>
            </div>
          </div>
        ))}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
        {automationStats.map((stat) => (
          <div key={stat.name} className="bg-slate-950 text-white p-6 rounded-2xl shadow-lg border border-slate-800 relative overflow-hidden">
            <div className="absolute inset-0 bg-gradient-to-br from-sky-400/20 via-transparent to-cyan-400/10 pointer-events-none" />
            <div className="relative flex items-center justify-between">
              <div className="p-2 bg-white/10 rounded-lg">
                <stat.icon className="h-5 w-5 text-cyan-200" />
              </div>
            </div>
            <div className="relative mt-4">
              <p className="text-2xl font-bold">{stat.value}</p>
              <p className="text-sm text-slate-300">{stat.name}</p>
            </div>
          </div>
        ))}
      </div>

      <div className="grid grid-cols-1 xl:grid-cols-3 gap-6">
        <div className="xl:col-span-2 bg-white rounded-2xl border border-gray-200 shadow-sm overflow-hidden">
          <div className="px-6 py-4 border-b flex items-center justify-between">
            <div>
              <h2 className="text-lg font-semibold text-gray-900">自动化任务流</h2>
              <p className="text-sm text-gray-500">AI 生成的任务、执行状态和下一步节奏</p>
            </div>
            <Wand2 className="h-5 w-5 text-brand-600" />
          </div>
          <div className="p-6">
            {automation?.jobs && automation.jobs.length > 0 ? (
              <div className="space-y-4">
                {automation.jobs.map((job) => (
                  <div key={job.id} className="rounded-xl border border-gray-200 p-4 bg-gradient-to-r from-white to-sky-50/40">
                    <div className="flex items-start justify-between gap-4">
                      <div>
                        <p className="font-semibold text-gray-900">{job.goal}</p>
                        <p className="text-sm text-gray-500 mt-1">
                          渠道: {job.channel_preferences.join(' / ') || '未配置'} · 风险: {job.risk_level} · 状态: {job.status}
                        </p>
                      </div>
                      <span className={`px-2 py-1 text-xs rounded-full ${
                        job.status === 'active' ? 'bg-green-100 text-green-700' :
                        job.status === 'waiting_approval' ? 'bg-amber-100 text-amber-700' :
                        job.status === 'completed' ? 'bg-blue-100 text-blue-700' :
                        'bg-gray-100 text-gray-700'
                      }`}>
                        {job.status}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="py-8 text-center text-gray-500">暂无自动化任务，先创建欢迎流或再激活流。</div>
            )}
          </div>
        </div>

        <div className="bg-white rounded-2xl border border-gray-200 shadow-sm overflow-hidden">
          <div className="px-6 py-4 border-b">
            <h2 className="text-lg font-semibold text-gray-900">风控建议</h2>
            <p className="text-sm text-gray-500">系统当前给出的自动化优化提示</p>
          </div>
          <div className="p-6 space-y-3">
            {automation?.recommendations?.length ? automation.recommendations.map((item) => (
              <div key={item} className="rounded-xl bg-gray-50 p-4 text-sm text-gray-700 leading-6">
                {item}
              </div>
            )) : (
              <div className="rounded-xl bg-gray-50 p-4 text-sm text-gray-500">
                暂无建议，说明当前自动化链路比较平稳。
              </div>
            )}
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="bg-white rounded-2xl border border-gray-200 shadow-sm overflow-hidden">
          <div className="px-6 py-4 border-b">
            <h2 className="text-lg font-semibold text-gray-900">审批队列</h2>
          </div>
          <div className="p-6">
            {automation?.approvals?.length ? (
              <div className="space-y-4">
                {automation.approvals.map((item) => (
                  <div key={item.id} className="flex items-start justify-between gap-4 rounded-xl border border-gray-200 p-4">
                    <div>
                      <p className="font-medium text-gray-900">{item.title}</p>
                      <p className="text-sm text-gray-500 mt-1">{item.reason}</p>
                    </div>
                    <span className="px-2 py-1 text-xs rounded-full bg-amber-100 text-amber-700">
                      {item.status}
                    </span>
                  </div>
                ))}
              </div>
            ) : (
              <div className="py-8 text-center text-gray-500">没有待审批项</div>
            )}
          </div>
        </div>

        <div className="bg-white rounded-2xl border border-gray-200 shadow-sm overflow-hidden">
          <div className="px-6 py-4 border-b">
            <h2 className="text-lg font-semibold text-gray-900">政策与实验</h2>
          </div>
          <div className="p-6 space-y-5">
            <div>
              <p className="text-sm font-medium text-gray-700 mb-3">政策规则</p>
              {automation?.policies?.length ? (
                <div className="space-y-3">
                  {automation.policies.map((rule) => (
                    <div key={rule.id} className="rounded-xl bg-gray-50 p-4 text-sm">
                      <div className="flex items-center justify-between">
                        <span className="font-medium text-gray-900">{rule.name}</span>
                        <span className="text-gray-500">{rule.enabled ? '启用' : '停用'}</span>
                      </div>
                      <p className="text-gray-500 mt-1">{rule.rule_type}</p>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="text-sm text-gray-500">暂无政策规则</p>
              )}
            </div>
            <div>
              <p className="text-sm font-medium text-gray-700 mb-3">实验看板</p>
              {automation?.experiments?.length ? (
                <div className="space-y-3">
                  {automation.experiments.map((exp) => (
                    <div key={exp.id} className="rounded-xl bg-gray-50 p-4 text-sm">
                      <div className="flex items-center justify-between">
                        <span className="font-medium text-gray-900">{exp.name}</span>
                        <span className="text-gray-500">{exp.status}</span>
                      </div>
                      <p className="text-gray-500 mt-1">{exp.hypothesis}</p>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="text-sm text-gray-500">暂无实验</p>
              )}
            </div>
          </div>
        </div>
      </div>

      <div className="bg-white rounded-2xl border border-gray-200 shadow-sm">
        <div className="px-6 py-4 border-b">
          <h2 className="text-lg font-semibold text-gray-900">活动列表</h2>
        </div>
        <div className="p-6">
          {campaignsData?.data && campaignsData.data.length > 0 ? (
            <div className="space-y-4">
              {campaignsData.data.map((campaign) => (
                <div key={campaign.id} className="flex items-center justify-between py-3 border-b last:border-0">
                  <div>
                    <p className="font-medium text-gray-900">{campaign.name}</p>
                    <p className="text-sm text-gray-500">{campaign.campaign_type} · {campaign.status}</p>
                  </div>
                  <span className={`px-2 py-1 text-xs rounded-full ${
                    campaign.status === 'active' ? 'bg-green-100 text-green-700' :
                    campaign.status === 'paused' ? 'bg-yellow-100 text-yellow-700' :
                    campaign.status === 'completed' ? 'bg-blue-100 text-blue-700' :
                    'bg-gray-100 text-gray-700'
                  }`}>
                    {campaign.status}
                  </span>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-gray-500 text-center py-8">暂无活动数据</p>
          )}
        </div>
      </div>
    </div>
  )
}
