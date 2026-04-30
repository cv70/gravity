import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import api from '@/services/api'
import { MessageCircle, Link2, Zap } from 'lucide-react'

interface Channel {
  id: string
  tenant_id: string
  channel_type: 'email' | 'wechat' | 'xiaohongshu' | 'douyin' | 'ads'
  name: string
  status: 'connected' | 'disconnected' | 'error'
  config: Record<string, unknown>
  created_at: string
}

export function ChannelsPage() {
  const [showModal, setShowModal] = useState(false)
  const [selectedType, setSelectedType] = useState<string>('')

  const { data } = useQuery({
    queryKey: ['channels'],
    queryFn: async () => {
      const { data: resp } = await api.get('/channels')
      return resp.data as { data: Channel[] }
    },
  })

  const channelTypes = [
    { type: 'email', name: '邮件', icon: '📧', description: 'SMTP 邮件发送' },
    { type: 'wechat', name: '微信公众号', icon: '💬', description: '微信生态对接' },
    { type: 'xiaohongshu', name: '小红书', icon: '📕', description: '内容发布和互动' },
    { type: 'douyin', name: '抖音', icon: '🎵', description: '短视频和直播消息' },
    { type: 'ads', name: '广告平台', icon: '📊', description: '巨量引擎/Google Ads' },
  ]

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'connected':
        return <span className="px-2 py-1 text-xs rounded-full bg-green-100 text-green-700">已连接</span>
      case 'disconnected':
        return <span className="px-2 py-1 text-xs rounded-full bg-gray-100 text-gray-700">未连接</span>
      case 'error':
        return <span className="px-2 py-1 text-xs rounded-full bg-red-100 text-red-700">错误</span>
      default:
        return <span className="px-2 py-1 text-xs rounded-full bg-gray-100 text-gray-700">{status}</span>
    }
  }

  const getChannelIcon = (type: string) => {
    switch (type) {
      case 'email':
        return <MessageCircle className="h-5 w-5" />
      case 'wechat':
        return <MessageCircle className="h-5 w-5" />
      case 'xiaohongshu':
        return <Link2 className="h-5 w-5" />
      case 'douyin':
        return <Zap className="h-5 w-5" />
      case 'ads':
        return <Link2 className="h-5 w-5" />
      default:
        return <Link2 className="h-5 w-5" />
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-gray-900">渠道管理</h1>
        <p className="text-gray-500 mt-1">连接和管理您的营销渠道</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {channelTypes.map((channel) => {
          const connected = data?.data?.find(c => c.channel_type === channel.type)
          return (
            <div key={channel.type} className="bg-white rounded-xl border p-6">
              <div className="flex items-start justify-between">
                <div className="flex items-center gap-3">
                  <div className="p-3 bg-brand-50 rounded-lg text-2xl">
                    {channel.icon}
                  </div>
                  <div>
                    <h3 className="font-medium text-gray-900">{channel.name}</h3>
                    <p className="text-sm text-gray-500">{channel.description}</p>
                  </div>
                </div>
              </div>

              <div className="mt-4 pt-4 border-t flex items-center justify-between">
                {connected ? (
                  <>
                    {getStatusBadge(connected.status)}
                    <button className="text-sm text-red-600 hover:text-red-800">
                      断开
                    </button>
                  </>
                ) : (
                  <>
                    {getStatusBadge('disconnected')}
                    <button
                      onClick={() => {
                        setSelectedType(channel.type)
                        setShowModal(true)
                      }}
                      className="text-sm text-brand-600 hover:text-brand-700"
                    >
                      连接
                    </button>
                  </>
                )}
              </div>
            </div>
          )
        })}
      </div>

      {data?.data && data.data.length > 0 && (
        <div className="bg-white rounded-xl border">
          <div className="px-6 py-4 border-b">
            <h2 className="text-lg font-semibold text-gray-900">已连接渠道</h2>
          </div>
          <div className="p-6">
            <div className="space-y-4">
              {data.data.map((channel) => (
                <div key={channel.id} className="flex items-center justify-between py-3 border-b last:border-0">
                  <div className="flex items-center gap-3">
                    <div className="p-2 bg-brand-50 rounded-lg">
                      {getChannelIcon(channel.channel_type)}
                    </div>
                    <div>
                      <p className="font-medium text-gray-900">{channel.name}</p>
                      <p className="text-sm text-gray-500">{channel.channel_type}</p>
                    </div>
                  </div>
                  {getStatusBadge(channel.status)}
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {showModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl shadow-xl p-6 w-full max-w-md">
            <h3 className="text-lg font-semibold mb-4">连接渠道</h3>
            <p className="text-gray-500 text-sm mb-4">
              {channelTypes.find(c => c.type === selectedType)?.name} 渠道连接功能开发中...
            </p>
            <div className="mt-4 flex justify-end">
              <button
                onClick={() => setShowModal(false)}
                className="px-4 py-2 border rounded-lg"
              >
                关闭
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}