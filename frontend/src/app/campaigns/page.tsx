import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { campaignsService } from '@/services/campaigns'
import { Plus, Play, Pause } from 'lucide-react'

export function CampaignsPage() {
  const queryClient = useQueryClient()
  const [showModal, setShowModal] = useState(false)

  const { data, isLoading } = useQuery({
    queryKey: ['campaigns'],
    queryFn: campaignsService.list,
  })

  const launchMutation = useMutation({
    mutationFn: campaignsService.launch,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['campaigns'] }),
  })

  const pauseMutation = useMutation({
    mutationFn: campaignsService.pause,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['campaigns'] }),
  })

  const createMutation = useMutation({
    mutationFn: campaignsService.create,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['campaigns'] })
      setShowModal(false)
    },
  })

  const handleCreate = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const form = e.currentTarget
    const formData = new FormData(form)
    await createMutation.mutateAsync({
      name: formData.get('name') as string,
      campaign_type: formData.get('type') as 'social' | 'email' | 'content' | 'ads',
      status: 'draft',
    })
    form.reset()
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">营销活动</h1>
          <p className="text-gray-500 mt-1">共 {data?.total ?? 0} 个活动</p>
        </div>
        <button
          onClick={() => setShowModal(true)}
          className="flex items-center gap-2 px-4 py-2 bg-brand-600 text-white rounded-lg hover:bg-brand-700"
        >
          <Plus className="h-4 w-4" />
          创建活动
        </button>
      </div>

      <div className="bg-white rounded-xl border">
        {isLoading ? (
          <div className="p-8 text-center text-gray-500">加载中...</div>
        ) : data?.data.length === 0 ? (
          <div className="p-8 text-center text-gray-500">暂无活动</div>
        ) : (
          <table className="w-full">
            <thead className="bg-gray-50 border-b">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">活动名称</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">类型</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">状态</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">创建时间</th>
                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase">操作</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200">
              {data?.data.map((campaign) => (
                <tr key={campaign.id} className="hover:bg-gray-50">
                  <td className="px-6 py-4 font-medium text-gray-900">{campaign.name}</td>
                  <td className="px-6 py-4 text-gray-500">{campaign.campaign_type}</td>
                  <td className="px-6 py-4">
                    <span className={`px-2 py-1 text-xs rounded-full ${
                      campaign.status === 'active' ? 'bg-green-100 text-green-700' :
                      campaign.status === 'paused' ? 'bg-yellow-100 text-yellow-700' :
                      campaign.status === 'completed' ? 'bg-blue-100 text-blue-700' :
                      'bg-gray-100 text-gray-700'
                    }`}>
                      {campaign.status}
                    </span>
                  </td>
                  <td className="px-6 py-4 text-gray-500">
                    {new Date(campaign.created_at).toLocaleDateString('zh-CN')}
                  </td>
                  <td className="px-6 py-4 text-right">
                    {campaign.status === 'active' ? (
                      <button
                        onClick={() => pauseMutation.mutate(campaign.id)}
                        disabled={pauseMutation.isPending}
                        className="inline-flex items-center gap-1 text-sm text-yellow-600 hover:text-yellow-800 disabled:opacity-50"
                      >
                        <Pause className="h-4 w-4" /> {pauseMutation.isPending ? '暂停中...' : '暂停'}
                      </button>
                    ) : campaign.status !== 'completed' ? (
                      <button
                        onClick={() => launchMutation.mutate(campaign.id)}
                        disabled={launchMutation.isPending}
                        className="inline-flex items-center gap-1 text-sm text-green-600 hover:text-green-800 disabled:opacity-50"
                      >
                        <Play className="h-4 w-4" /> {launchMutation.isPending ? '启动中...' : '启动'}
                      </button>
                    ) : null}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {showModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl shadow-xl p-6 w-full max-w-md">
            <h3 className="text-lg font-semibold mb-4">创建活动</h3>
            <form onSubmit={handleCreate} className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">活动名称 *</label>
                <input name="name" required className="w-full px-3 py-2 border rounded-lg" />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">类型 *</label>
                <select name="type" required className="w-full px-3 py-2 border rounded-lg">
                  <option value="email">邮件</option>
                  <option value="social">社交媒体</option>
                  <option value="content">内容营销</option>
                  <option value="ads">付费广告</option>
                </select>
              </div>
              <div className="flex justify-end gap-3 pt-4">
                <button type="button" onClick={() => setShowModal(false)} className="px-4 py-2 border rounded-lg">取消</button>
                <button type="submit" className="px-4 py-2 bg-brand-600 text-white rounded-lg">创建</button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  )
}