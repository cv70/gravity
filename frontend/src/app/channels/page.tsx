import { useMemo, useState } from 'react'
import type { ReactNode } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { CheckCircle2, ExternalLink, Link2, MessageCircle, PencilLine, Plus, RefreshCw, Settings2, Trash2, X, Zap } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { channelService } from '@/services/channel'
import type { ChannelAccount, ChannelCreatePayload } from '@/types'

type ChannelFormState = {
  platform: string
  name: string
  credentials_encrypted: string
  endpoint: string
  auth_token: string
  api_key: string
  status: string
}

const initialForm: ChannelFormState = {
  platform: 'email',
  name: '',
  credentials_encrypted: '',
  endpoint: '',
  auth_token: '',
  api_key: '',
  status: 'connected',
}

const channelMeta: Record<string, { label: string; icon: ReactNode; description: string }> = {
  email: { label: '邮件', icon: <MessageCircle className="h-5 w-5" />, description: 'SMTP / webhook 发送' },
  wechat: { label: '微信', icon: <MessageCircle className="h-5 w-5" />, description: '公众号 / 企业微信触达' },
  xiaohongshu: { label: '小红书', icon: <Link2 className="h-5 w-5" />, description: '内容分发与互动' },
  douyin: { label: '抖音', icon: <Zap className="h-5 w-5" />, description: '短视频 / 直播联动' },
  ads: { label: '广告', icon: <ExternalLink className="h-5 w-5" />, description: '广告投放与预算执行' },
}

export function ChannelsPage() {
  const queryClient = useQueryClient()
  const [showModal, setShowModal] = useState(false)
  const [editing, setEditing] = useState<ChannelAccount | null>(null)
  const [form, setForm] = useState<ChannelFormState>(initialForm)

  const { data, isLoading } = useQuery({
    queryKey: ['channels'],
    queryFn: () => channelService.list(),
  })

  const createMutation = useMutation({
    mutationFn: (payload: ChannelCreatePayload) => channelService.create(payload),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['channels'] })
      setShowModal(false)
    },
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, payload }: { id: string; payload: Partial<ChannelCreatePayload> & { last_sync_at?: string } }) =>
      channelService.update(id, payload),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['channels'] })
      setShowModal(false)
    },
  })

  const deleteMutation = useMutation({
    mutationFn: channelService.remove,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['channels'] })
    },
  })

  const channelCards = useMemo(() => {
    const list = data?.data ?? []
    return [
      { label: '已连接渠道', value: list.filter((item) => item.status === 'connected').length, hint: '可以直接被工作流调用', icon: CheckCircle2 },
      { label: '错误状态', value: list.filter((item) => item.status === 'error').length, hint: '需要优先排查的渠道', icon: Settings2 },
      { label: '总渠道数', value: list.length, hint: '当前租户的渠道资产池', icon: Link2 },
      { label: '默认类型', value: Object.keys(channelMeta).length, hint: '支持的接入类型', icon: Zap },
    ]
  }, [data])

  const openCreate = () => {
    setEditing(null)
    setForm(initialForm)
    setShowModal(true)
  }

  const openEdit = (channel: ChannelAccount) => {
    setEditing(channel)
    setForm({
      platform: channel.platform,
      name: channel.name,
      credentials_encrypted: channel.credentials_encrypted,
      endpoint: (channel.settings.endpoint as string) || '',
      auth_token: (channel.settings.auth_token as string) || '',
      api_key: (channel.settings.api_key as string) || '',
      status: channel.status,
    })
    setShowModal(true)
  }

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const payload: ChannelCreatePayload = {
      platform: form.platform,
      name: form.name,
      credentials_encrypted: form.credentials_encrypted,
      status: form.status,
      settings: {
        endpoint: form.endpoint || undefined,
        auth_token: form.auth_token || undefined,
        api_key: form.api_key || undefined,
      },
    }

    if (editing) {
      await updateMutation.mutateAsync({
        id: editing.id,
        payload: {
          ...payload,
          last_sync_at: editing.last_sync_at || undefined,
        },
      })
    } else {
      await createMutation.mutateAsync(payload)
    }
  }

  const statusBadge = (status: string) => {
    switch (status) {
      case 'connected':
        return <span className="rounded-full bg-emerald-100 px-2.5 py-1 text-xs font-medium text-emerald-700">已连接</span>
      case 'disconnected':
        return <span className="rounded-full bg-slate-100 px-2.5 py-1 text-xs font-medium text-slate-600">未连接</span>
      case 'error':
        return <span className="rounded-full bg-red-100 px-2.5 py-1 text-xs font-medium text-red-700">错误</span>
      default:
        return <span className="rounded-full bg-slate-100 px-2.5 py-1 text-xs font-medium text-slate-600">{status}</span>
    }
  }

  return (
    <div className="space-y-6">
      <div className="relative overflow-hidden rounded-3xl bg-slate-950 text-white shadow-xl border border-slate-800">
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_top_right,rgba(34,211,238,0.22),transparent_30%),radial-gradient(circle_at_bottom_left,rgba(59,130,246,0.18),transparent_28%)]" />
        <div className="relative flex flex-col gap-5 p-6 lg:p-8">
          <div className="flex items-start justify-between gap-4">
            <div>
              <div className="inline-flex items-center gap-2 rounded-full bg-white/10 px-3 py-1 text-xs text-cyan-100">
                <Link2 className="h-4 w-4" />
                渠道控制台
              </div>
              <h1 className="mt-4 text-3xl font-bold tracking-tight">统一连接渠道、管理配置和执行状态</h1>
              <p className="mt-3 max-w-3xl text-sm lg:text-base text-slate-300">
                邮件、微信、广告、内容平台都在这里纳管，供自动化工作流统一调度。
              </p>
            </div>
            <Button
              onClick={openCreate}
              variant="brand"
            >
              <Plus className="h-4 w-4" />
              添加渠道
            </Button>
          </div>

          <div className="grid grid-cols-1 gap-3 md:grid-cols-4">
            {channelCards.map((card) => (
              <div key={card.label} className="rounded-2xl border border-white/10 bg-white/5 p-4 backdrop-blur">
                <div className="flex items-start justify-between">
                  <div>
                    <p className="text-3xl font-bold">{card.value}</p>
                    <p className="mt-1 text-sm text-slate-300">{card.label}</p>
                  </div>
                  <div className="rounded-xl bg-white/10 p-2 text-cyan-200">
                    <card.icon className="h-4 w-4" />
                  </div>
                </div>
                <p className="mt-3 text-xs text-slate-400">{card.hint}</p>
              </div>
            ))}
          </div>
        </div>
      </div>

      <div className="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm">
        <div className="flex items-center justify-between border-b px-6 py-4">
          <div>
            <h2 className="text-lg font-semibold text-slate-900">已连接渠道</h2>
            <p className="text-sm text-slate-500">管理渠道配置、凭证和执行状态</p>
          </div>
          <Button
            onClick={() => queryClient.invalidateQueries({ queryKey: ['channels'] })}
            variant="secondary"
            size="sm"
          >
            <RefreshCw className="h-4 w-4" />
            刷新
          </Button>
        </div>

        {isLoading ? (
          <div className="p-10 text-center text-slate-500">加载渠道中...</div>
        ) : data?.data.length ? (
          <div className="grid gap-4 p-6 md:grid-cols-2 xl:grid-cols-3">
            {data.data.map((channel) => {
              const meta = channelMeta[channel.platform] ?? {
                label: channel.platform,
                icon: <Link2 className="h-5 w-5" />,
                description: '自定义渠道',
              }

              return (
                <div key={channel.id} className="rounded-2xl border border-slate-200 p-5 shadow-sm transition hover:shadow-md">
                  <div className="flex items-start justify-between gap-4">
                    <div className="flex items-center gap-3">
                      <div className="rounded-xl bg-brand-50 p-3 text-brand-700">{meta.icon}</div>
                      <div>
                        <h3 className="font-semibold text-slate-900">{channel.name}</h3>
                        <p className="text-sm text-slate-500">{meta.label} · {meta.description}</p>
                      </div>
                    </div>
                    {statusBadge(channel.status)}
                  </div>

                  <div className="mt-4 space-y-2 rounded-2xl bg-slate-50 p-4 text-sm text-slate-600">
                    <div className="flex items-center justify-between">
                      <span>Endpoint</span>
                      <span className="max-w-[12rem] truncate font-medium text-slate-900">{(channel.settings.endpoint as string) || '未配置'}</span>
                    </div>
                    <div className="flex items-center justify-between">
                      <span>最后同步</span>
                      <span className="font-medium text-slate-900">{channel.last_sync_at ? new Date(channel.last_sync_at).toLocaleString('zh-CN') : '未同步'}</span>
                    </div>
                  </div>

                  <div className="mt-4 flex items-center justify-between border-t border-slate-100 pt-4">
                    <Button
                      onClick={() => openEdit(channel)}
                      variant="secondary"
                      size="sm"
                    >
                      <PencilLine className="h-4 w-4" />
                      编辑
                    </Button>
                    <div className="flex items-center gap-2">
                      <Button
                        onClick={() => queryClient.invalidateQueries({ queryKey: ['channels'] })}
                        variant="ghost"
                        size="sm"
                        className="rounded-lg px-3 py-1.5"
                      >
                        <RefreshCw className="h-4 w-4" />
                      </Button>
                      <Button
                        onClick={() => {
                          if (window.confirm(`确定要断开渠道 "${channel.name}" 吗？`)) {
                            deleteMutation.mutate(channel.id)
                          }
                        }}
                        variant="destructive"
                        size="sm"
                      >
                        <Trash2 className="h-4 w-4" />
                        断开
                      </Button>
                    </div>
                  </div>
                </div>
              )
            })}
          </div>
        ) : (
          <div className="p-10 text-center text-slate-500">还没有连接任何渠道。</div>
        )}
      </div>

      {showModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/60 p-4">
          <div className="w-full max-w-2xl rounded-3xl bg-white shadow-2xl">
            <div className="flex items-start justify-between gap-4 border-b px-6 py-5">
              <div>
                <h3 className="text-lg font-semibold text-slate-900">{editing ? '编辑渠道' : '连接渠道'}</h3>
                <p className="text-sm text-slate-500">填写 webhook 或 API 配置后，自动化任务可以直接调用该渠道。</p>
              </div>
              <Button onClick={() => setShowModal(false)} variant="ghost" size="sm" className="rounded-full p-2 text-slate-400 hover:bg-slate-100 hover:text-slate-700">
                <X className="h-5 w-5" />
              </Button>
            </div>

            <form onSubmit={handleSubmit} className="space-y-4 p-6">
              <div className="grid gap-4 md:grid-cols-2">
                <div>
                  <label className="mb-1 block text-sm font-medium text-slate-700">渠道类型</label>
                  <select
                    value={form.platform}
                    onChange={(e) => setForm((prev) => ({ ...prev, platform: e.target.value }))}
                    className="w-full rounded-xl border border-slate-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                  >
                    {Object.entries(channelMeta).map(([key, meta]) => (
                      <option key={key} value={key}>{meta.label}</option>
                    ))}
                  </select>
                </div>
                <div>
                  <label className="mb-1 block text-sm font-medium text-slate-700">名称</label>
                  <input
                    value={form.name}
                    onChange={(e) => setForm((prev) => ({ ...prev, name: e.target.value }))}
                    className="w-full rounded-xl border border-slate-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                  />
                </div>
              </div>

              <div className="grid gap-4 md:grid-cols-2">
                <div>
                  <label className="mb-1 block text-sm font-medium text-slate-700">凭证密文</label>
                  <input
                    value={form.credentials_encrypted}
                    onChange={(e) => setForm((prev) => ({ ...prev, credentials_encrypted: e.target.value }))}
                    placeholder="encrypted-token"
                    className="w-full rounded-xl border border-slate-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                  />
                </div>
                <div>
                  <label className="mb-1 block text-sm font-medium text-slate-700">状态</label>
                  <select
                    value={form.status}
                    onChange={(e) => setForm((prev) => ({ ...prev, status: e.target.value }))}
                    className="w-full rounded-xl border border-slate-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                  >
                    <option value="connected">已连接</option>
                    <option value="disconnected">未连接</option>
                    <option value="error">错误</option>
                  </select>
                </div>
              </div>

              <div className="grid gap-4 md:grid-cols-3">
                <div>
                  <label className="mb-1 block text-sm font-medium text-slate-700">Endpoint</label>
                  <input
                    value={form.endpoint}
                    onChange={(e) => setForm((prev) => ({ ...prev, endpoint: e.target.value }))}
                    className="w-full rounded-xl border border-slate-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                  />
                </div>
                <div>
                  <label className="mb-1 block text-sm font-medium text-slate-700">Auth Token</label>
                  <input
                    value={form.auth_token}
                    onChange={(e) => setForm((prev) => ({ ...prev, auth_token: e.target.value }))}
                    className="w-full rounded-xl border border-slate-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                  />
                </div>
                <div>
                  <label className="mb-1 block text-sm font-medium text-slate-700">API Key</label>
                  <input
                    value={form.api_key}
                    onChange={(e) => setForm((prev) => ({ ...prev, api_key: e.target.value }))}
                    className="w-full rounded-xl border border-slate-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                  />
                </div>
              </div>

              <div className="flex justify-end gap-3 pt-2">
                <Button type="button" onClick={() => setShowModal(false)} variant="secondary">
                  取消
                </Button>
                <Button
                  type="submit"
                  disabled={createMutation.isPending || updateMutation.isPending}
                  variant="brand"
                >
                  {editing ? '保存更改' : createMutation.isPending ? '连接中...' : '连接渠道'}
                </Button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  )
}
