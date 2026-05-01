import { useMemo, useState } from 'react'
import type { ReactNode } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link2, MessageCircle, Plus, RefreshCw, Trash2, Zap } from 'lucide-react'

import { channelService } from '@/services/channel'
import type { ChannelAccount } from '@/types'

type ChannelFormState = {
  platform: string
  name: string
  credentials_encrypted: string
  endpoint: string
  auth_token: string
  api_key: string
}

const initialForm: ChannelFormState = {
  platform: 'email',
  name: '',
  credentials_encrypted: '',
  endpoint: '',
  auth_token: '',
  api_key: '',
}

const channelMeta: Record<string, { label: string; icon: ReactNode; description: string }> = {
  email: {
    label: '邮件',
    icon: <MessageCircle className="h-5 w-5" />,
    description: 'SMTP / webhook 发送',
  },
  wechat: {
    label: '微信',
    icon: <MessageCircle className="h-5 w-5" />,
    description: '公众号 / 企业微信触达',
  },
  xiaohongshu: {
    label: '小红书',
    icon: <Link2 className="h-5 w-5" />,
    description: '内容分发与互动',
  },
  douyin: {
    label: '抖音',
    icon: <Zap className="h-5 w-5" />,
    description: '短视频 / 直播联动',
  },
  ads: {
    label: '广告',
    icon: <Link2 className="h-5 w-5" />,
    description: '广告投放与预算执行',
  },
}

export function ChannelsPage() {
  const queryClient = useQueryClient()
  const [showModal, setShowModal] = useState(false)
  const [form, setForm] = useState<ChannelFormState>(initialForm)

  const { data, isLoading } = useQuery({
    queryKey: ['channels'],
    queryFn: () => channelService.list(),
  })

  const createMutation = useMutation({
    mutationFn: async (payload: ChannelFormState) => {
      return channelService.create({
        platform: payload.platform,
        name: payload.name,
        credentials_encrypted: payload.credentials_encrypted,
        settings: {
          endpoint: payload.endpoint || undefined,
          auth_token: payload.auth_token || undefined,
          api_key: payload.api_key || undefined,
        },
        status: 'connected',
      })
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['channels'] })
      setShowModal(false)
      setForm(initialForm)
    },
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => channelService.remove(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['channels'] })
    },
  })

  const connectedMap = useMemo(() => {
    const entries = new Map<string, ChannelAccount>()
    data?.data?.forEach((item) => entries.set(item.platform, item))
    return entries
  }, [data?.data])


  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'connected':
        return <span className="rounded-full bg-emerald-100 px-2 py-1 text-xs text-emerald-700">已连接</span>
      case 'disconnected':
        return <span className="rounded-full bg-gray-100 px-2 py-1 text-xs text-gray-700">未连接</span>
      case 'error':
        return <span className="rounded-full bg-red-100 px-2 py-1 text-xs text-red-700">错误</span>
      default:
        return <span className="rounded-full bg-gray-100 px-2 py-1 text-xs text-gray-700">{status}</span>
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-end justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">渠道管理</h1>
          <p className="mt-1 text-gray-500">连接 webhook、公众号、内容平台和投放账户，供自动化执行器直接调用</p>
        </div>
        <button
          onClick={() => setShowModal(true)}
          className="inline-flex items-center gap-2 rounded-xl bg-brand-600 px-4 py-2.5 text-white hover:bg-brand-700"
        >
          <Plus className="h-4 w-4" />
          添加渠道
        </button>
      </div>

      <div className="grid gap-6 md:grid-cols-2 xl:grid-cols-3">
        {Object.entries(channelMeta).map(([platform, meta]) => {
          const connected = connectedMap.get(platform)
          return (
            <div key={platform} className="rounded-2xl border border-gray-200 bg-white p-6 shadow-sm">
              <div className="flex items-start justify-between gap-4">
                <div className="flex items-center gap-3">
                  <div className="rounded-xl bg-brand-50 p-3 text-brand-700">{meta.icon}</div>
                  <div>
                    <h3 className="font-semibold text-gray-900">{meta.label}</h3>
                    <p className="text-sm text-gray-500">{meta.description}</p>
                  </div>
                </div>
                {connected ? getStatusBadge(connected.status) : getStatusBadge('disconnected')}
              </div>

              <div className="mt-6 flex items-center justify-between border-t border-gray-100 pt-4">
                <div className="text-sm text-gray-500">
                  {connected ? connected.name : '尚未配置'}
                </div>
                {connected ? (
                  <button
                    onClick={() => deleteMutation.mutate(connected.id)}
                    className="inline-flex items-center gap-1 text-sm text-red-600 hover:text-red-700"
                  >
                    <Trash2 className="h-4 w-4" />
                    断开
                  </button>
                ) : (
                  <button
                    onClick={() => {
                      setForm((prev) => ({ ...prev, platform }))
                      setShowModal(true)
                    }}
                    className="inline-flex items-center gap-1 text-sm text-brand-600 hover:text-brand-700"
                  >
                    <Plus className="h-4 w-4" />
                    连接
                  </button>
                )}
              </div>
            </div>
          )
        })}
      </div>

      <div className="rounded-2xl border border-gray-200 bg-white shadow-sm">
        <div className="flex items-center justify-between border-b border-gray-100 px-6 py-4">
          <div>
            <h2 className="text-lg font-semibold text-gray-900">已连接渠道</h2>
            <p className="text-sm text-gray-500">自动化执行器优先使用这些账号执行真实动作</p>
          </div>
          <button
            onClick={() => queryClient.invalidateQueries({ queryKey: ['channels'] })}
            className="inline-flex items-center gap-2 rounded-lg border border-gray-200 px-3 py-2 text-sm text-gray-700 hover:bg-gray-50"
          >
            <RefreshCw className="h-4 w-4" />
            刷新
          </button>
        </div>

        <div className="px-6 py-4">
          {isLoading ? (
            <div className="py-8 text-sm text-gray-500">加载渠道中...</div>
          ) : data?.data?.length ? (
            <div className="space-y-3">
              {data.data.map((channel) => {
                const meta = channelMeta[channel.platform] ?? {
                  label: channel.platform,
                  icon: <Link2 className="h-5 w-5" />,
                  description: '自定义渠道',
                }
                return (
                  <div key={channel.id} className="flex items-center justify-between rounded-xl border border-gray-100 px-4 py-3">
                    <div className="flex items-center gap-3">
                      <div className="rounded-lg bg-brand-50 p-2 text-brand-700">{meta.icon}</div>
                      <div>
                        <p className="font-medium text-gray-900">{channel.name}</p>
                        <p className="text-sm text-gray-500">
                          {meta.label} · {'endpoint' in channel.settings ? '已配置 endpoint' : '未配置 endpoint'}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center gap-3">
                      {getStatusBadge(channel.status)}
                      <button
                        onClick={() => deleteMutation.mutate(channel.id)}
                        className="text-sm text-red-600 hover:text-red-700"
                      >
                        删除
                      </button>
                    </div>
                  </div>
                )
              })}
            </div>
          ) : (
            <div className="py-8 text-sm text-gray-500">还没有连接任何渠道。</div>
          )}
        </div>
      </div>

      {showModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 px-4">
          <div className="w-full max-w-xl rounded-2xl bg-white p-6 shadow-2xl">
            <div className="mb-4">
              <h3 className="text-lg font-semibold text-gray-900">连接渠道</h3>
              <p className="text-sm text-gray-500">填写 webhook 或 API 配置后，自动化任务可以直接调用该渠道。</p>
            </div>

            <div className="grid gap-4 md:grid-cols-2">
              <label className="space-y-2">
                <span className="text-sm font-medium text-gray-700">渠道类型</span>
                <select
                  value={form.platform}
                  onChange={(e) => setForm((prev) => ({ ...prev, platform: e.target.value }))}
                  className="w-full rounded-xl border border-gray-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100"
                >
                  {Object.entries(channelMeta).map(([key, meta]) => (
                    <option key={key} value={key}>{meta.label}</option>
                  ))}
                </select>
              </label>

              <label className="space-y-2">
                <span className="text-sm font-medium text-gray-700">名称</span>
                <input
                  value={form.name}
                  onChange={(e) => setForm((prev) => ({ ...prev, name: e.target.value }))}
                  placeholder="例如：企业微信-营销群"
                  className="w-full rounded-xl border border-gray-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100"
                />
              </label>

              <label className="space-y-2 md:col-span-2">
                <span className="text-sm font-medium text-gray-700">credentials_encrypted</span>
                <input
                  value={form.credentials_encrypted}
                  onChange={(e) => setForm((prev) => ({ ...prev, credentials_encrypted: e.target.value }))}
                  placeholder="可先填 placeholder，后续接密钥管理"
                  className="w-full rounded-xl border border-gray-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100"
                />
              </label>

              <label className="space-y-2 md:col-span-2">
                <span className="text-sm font-medium text-gray-700">Endpoint</span>
                <input
                  value={form.endpoint}
                  onChange={(e) => setForm((prev) => ({ ...prev, endpoint: e.target.value }))}
                  placeholder="https://hooks.example.com/dispatch"
                  className="w-full rounded-xl border border-gray-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100"
                />
              </label>

              <label className="space-y-2">
                <span className="text-sm font-medium text-gray-700">Bearer Token</span>
                <input
                  value={form.auth_token}
                  onChange={(e) => setForm((prev) => ({ ...prev, auth_token: e.target.value }))}
                  className="w-full rounded-xl border border-gray-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100"
                />
              </label>

              <label className="space-y-2">
                <span className="text-sm font-medium text-gray-700">API Key</span>
                <input
                  value={form.api_key}
                  onChange={(e) => setForm((prev) => ({ ...prev, api_key: e.target.value }))}
                  className="w-full rounded-xl border border-gray-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100"
                />
              </label>
            </div>

            <div className="mt-6 flex items-center justify-end gap-3">
              <button
                onClick={() => setShowModal(false)}
                className="rounded-xl border border-gray-200 px-4 py-2.5 text-gray-700 hover:bg-gray-50"
              >
                取消
              </button>
              <button
                onClick={() => createMutation.mutate(form)}
                disabled={createMutation.isPending}
                className="rounded-xl bg-brand-600 px-4 py-2.5 text-white hover:bg-brand-700 disabled:opacity-50"
              >
                {createMutation.isPending ? '连接中...' : '保存并连接'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
