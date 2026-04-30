import { useQuery } from '@tanstack/react-query'
import { analyticsService } from '@/services/analytics'
import { campaignsService } from '@/services/campaigns'
import { Users, Megaphone, Target, TrendingUp } from 'lucide-react'

export function DashboardPage() {
  const { data: dashboard, isLoading } = useQuery({
    queryKey: ['analytics', 'dashboard'],
    queryFn: analyticsService.getDashboard,
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

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-2xl font-bold text-gray-900">仪表盘</h1>
        <p className="text-gray-500 mt-1">查看您的营销数据概览</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {stats.map((stat) => (
          <div key={stat.name} className="bg-white p-6 rounded-xl border">
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

      <div className="bg-white rounded-xl border">
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