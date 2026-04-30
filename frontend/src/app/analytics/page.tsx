import { useQuery } from '@tanstack/react-query'
import { analyticsService } from '@/services/analytics'

export function AnalyticsPage() {
  const { data: dashboard, isLoading } = useQuery({
    queryKey: ['analytics', 'dashboard'],
    queryFn: analyticsService.getDashboard,
  })

  const { data: funnel } = useQuery({
    queryKey: ['analytics', 'funnel'],
    queryFn: () => analyticsService.getFunnel(),
  })

  if (isLoading) {
    return <div className="animate-pulse">加载中...</div>
  }

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-2xl font-bold text-gray-900">数据分析</h1>
        <p className="text-gray-500 mt-1">查看营销效果和转化漏斗</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <div className="bg-white p-6 rounded-xl border">
          <p className="text-sm text-gray-500">总联系人</p>
          <p className="text-2xl font-bold text-gray-900 mt-1">{dashboard?.total_contacts ?? 0}</p>
        </div>
        <div className="bg-white p-6 rounded-xl border">
          <p className="text-sm text-gray-500">活跃活动</p>
          <p className="text-2xl font-bold text-gray-900 mt-1">{dashboard?.active_campaigns ?? 0}</p>
        </div>
        <div className="bg-white p-6 rounded-xl border">
          <p className="text-sm text-gray-500">总转化数</p>
          <p className="text-2xl font-bold text-gray-900 mt-1">{dashboard?.total_conversions ?? 0}</p>
        </div>
        <div className="bg-white p-6 rounded-xl border">
          <p className="text-sm text-gray-500">转化率</p>
          <p className="text-2xl font-bold text-gray-900 mt-1">
            {((dashboard?.conversion_rate ?? 0) * 100).toFixed(1)}%
          </p>
        </div>
      </div>

      <div className="bg-white rounded-xl border">
        <div className="px-6 py-4 border-b">
          <h2 className="text-lg font-semibold text-gray-900">转化漏斗</h2>
        </div>
        <div className="p-6">
          {funnel?.steps && funnel.steps.length > 0 ? (
            <div className="space-y-4">
              {funnel.steps.map((step, index) => {
                const maxCount = funnel.steps[0].count
                const width = maxCount > 0 ? (step.count / maxCount) * 100 : 0
                return (
                  <div key={step.step}>
                    <div className="flex items-center justify-between mb-1">
                      <span className="text-sm font-medium text-gray-700">{step.step}</span>
                      <span className="text-sm text-gray-500">
                        {step.count.toLocaleString()} {index > 0 && `(${(step.dropoff_rate * 100).toFixed(1)}%)`}
                      </span>
                    </div>
                    <div className="h-8 bg-gray-100 rounded-lg overflow-hidden">
                      <div
                        className="h-full bg-brand-500 rounded-lg transition-all"
                        style={{ width: `${width}%` }}
                      />
                    </div>
                  </div>
                )
              })}
            </div>
          ) : (
            <p className="text-gray-500 text-center py-8">暂无漏斗数据</p>
          )}
        </div>
      </div>
    </div>
  )
}
