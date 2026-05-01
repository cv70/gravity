import { useEffect, useMemo, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Calendar, Clock3, Megaphone, Pause, PencilLine, Play, Plus, Rocket, Search, Trash2, X } from 'lucide-react'

import { campaignsService } from '@/services/campaigns'
import type { Campaign, CampaignCreatePayload } from '@/types'

type CampaignFormState = {
  name: string
  campaign_type: CampaignCreatePayload['campaign_type']
  description: string
  start_date: string
  end_date: string
}

const emptyForm: CampaignFormState = {
  name: '',
  campaign_type: 'email',
  description: '',
  start_date: '',
  end_date: '',
}

export function CampaignsPage() {
  const queryClient = useQueryClient()
  const [showModal, setShowModal] = useState(false)
  const [editing, setEditing] = useState<Campaign | null>(null)
  const [form, setForm] = useState<CampaignFormState>(emptyForm)
  const [searchInput, setSearchInput] = useState('')
  const [debouncedSearch, setDebouncedSearch] = useState('')

  useEffect(() => {
    const timer = setTimeout(() => setDebouncedSearch(searchInput), 300)
    return () => clearTimeout(timer)
  }, [searchInput])

  useEffect(() => {
    if (!showModal) {
      setEditing(null)
      setForm(emptyForm)
    }
  }, [showModal])

  const { data, isLoading } = useQuery({
    queryKey: ['campaigns'],
    queryFn: campaignsService.list,
  })

  const createMutation = useMutation({
    mutationFn: (payload: CampaignCreatePayload) => campaignsService.create(payload),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['campaigns'] })
      setShowModal(false)
    },
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, payload }: { id: string; payload: Partial<CampaignCreatePayload> & { status?: Campaign['status'] } }) =>
      campaignsService.update(id, payload),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['campaigns'] })
      setShowModal(false)
    },
  })

  const launchMutation = useMutation({
    mutationFn: campaignsService.launch,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['campaigns'] })
    },
  })

  const pauseMutation = useMutation({
    mutationFn: campaignsService.pause,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['campaigns'] })
    },
  })

  const deleteMutation = useMutation({
    mutationFn: campaignsService.delete,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['campaigns'] })
    },
  })

  const filteredCampaigns = useMemo(() => {
    const list = data?.data ?? []
    if (!debouncedSearch) return list
    const q = debouncedSearch.toLowerCase()
    return list.filter((campaign) => campaign.name.toLowerCase().includes(q) || campaign.campaign_type.toLowerCase().includes(q))
  }, [data?.data, debouncedSearch])

  const stats = useMemo(() => {
    const list = data?.data ?? []
    return [
      { label: '活动总数', value: list.length, hint: '当前租户的营销活动池', icon: Megaphone },
      { label: '运行中', value: list.filter((item) => item.status === 'active').length, hint: '正在执行的活动', icon: Rocket },
      { label: '草稿', value: list.filter((item) => item.status === 'draft').length, hint: '待启动的活动', icon: PencilLine },
      { label: '暂停', value: list.filter((item) => item.status === 'paused').length, hint: '被暂时挂起的活动', icon: Pause },
    ]
  }, [data])

  const openCreate = () => {
    setEditing(null)
    setForm(emptyForm)
    setShowModal(true)
  }

  const openEdit = (campaign: Campaign) => {
    setEditing(campaign)
    setForm({
      name: campaign.name,
      campaign_type: campaign.campaign_type,
      description: campaign.description || '',
      start_date: campaign.start_date || '',
      end_date: campaign.end_date || '',
    })
    setShowModal(true)
  }

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const payload: CampaignCreatePayload = {
      name: form.name,
      campaign_type: form.campaign_type,
      description: form.description || undefined,
      start_date: form.start_date || null,
      end_date: form.end_date || null,
    }

    if (editing) {
      await updateMutation.mutateAsync({
        id: editing.id,
        payload: {
          ...payload,
          status: editing.status,
        },
      })
    } else {
      await createMutation.mutateAsync(payload)
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
                <Calendar className="h-4 w-4" />
                活动编排中心
              </div>
              <h1 className="mt-4 text-3xl font-bold tracking-tight">统一创建、执行和切换营销活动</h1>
              <p className="mt-3 max-w-3xl text-sm lg:text-base text-slate-300">
                活动页支持起草、启动、暂停、编辑和删除，帮助团队快速推进增长计划。
              </p>
            </div>
            <button
              onClick={openCreate}
              className="inline-flex items-center gap-2 rounded-xl bg-cyan-400 px-4 py-2 text-sm font-semibold text-slate-950 shadow-lg shadow-cyan-500/20 transition hover:bg-cyan-300"
            >
              <Plus className="h-4 w-4" />
              创建活动
            </button>
          </div>

          <div className="grid grid-cols-1 gap-3 md:grid-cols-4">
            {stats.map((stat) => (
              <div key={stat.label} className="rounded-2xl border border-white/10 bg-white/5 p-4 backdrop-blur">
                <div className="flex items-start justify-between">
                  <div>
                    <p className="text-3xl font-bold">{stat.value}</p>
                    <p className="mt-1 text-sm text-slate-300">{stat.label}</p>
                  </div>
                  <div className="rounded-xl bg-white/10 p-2 text-cyan-200">
                    <stat.icon className="h-4 w-4" />
                  </div>
                </div>
                <p className="mt-3 text-xs text-slate-400">{stat.hint}</p>
              </div>
            ))}
          </div>
        </div>
      </div>

      <div className="rounded-2xl border border-slate-200 bg-white p-4 shadow-sm lg:p-5">
        <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
          <div className="relative flex-1">
            <Search className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
            <input
              value={searchInput}
              onChange={(e) => setSearchInput(e.target.value)}
              placeholder="搜索活动名称或类型"
              className="w-full rounded-xl border border-slate-200 bg-white py-3 pl-10 pr-4 outline-none transition focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
            />
          </div>
          <button
            onClick={() => queryClient.invalidateQueries({ queryKey: ['campaigns'] })}
            className="rounded-xl border border-slate-200 px-4 py-3 text-sm font-medium text-slate-700 transition hover:bg-slate-50"
          >
            刷新
          </button>
        </div>
      </div>

      <div className="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm">
        {isLoading ? (
          <div className="p-10 text-center text-slate-500">加载中...</div>
        ) : filteredCampaigns.length === 0 ? (
          <div className="p-10 text-center text-slate-500">暂无活动。</div>
        ) : (
          <table className="w-full">
            <thead className="bg-slate-50">
              <tr className="border-b border-slate-200">
                <th className="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-500">活动名称</th>
                <th className="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-500">类型</th>
                <th className="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-500">状态</th>
                <th className="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-500">时间</th>
                <th className="px-6 py-3 text-right text-xs font-semibold uppercase tracking-wide text-slate-500">操作</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-200">
              {filteredCampaigns.map((campaign) => (
                <tr key={campaign.id} className="hover:bg-slate-50/80">
                  <td className="px-6 py-4">
                    <div>
                      <p className="font-semibold text-slate-900">{campaign.name}</p>
                      <p className="mt-1 text-xs text-slate-400">ID: {campaign.id.slice(0, 8)}</p>
                    </div>
                  </td>
                  <td className="px-6 py-4 text-slate-600">{campaign.campaign_type}</td>
                  <td className="px-6 py-4">
                    <span className={`rounded-full px-2.5 py-1 text-xs font-medium ${
                      campaign.status === 'active'
                        ? 'bg-emerald-100 text-emerald-700'
                        : campaign.status === 'paused'
                          ? 'bg-amber-100 text-amber-700'
                          : campaign.status === 'completed'
                            ? 'bg-sky-100 text-sky-700'
                            : 'bg-slate-100 text-slate-600'
                    }`}>
                      {campaign.status}
                    </span>
                  </td>
                  <td className="px-6 py-4 text-slate-600">
                    <div className="flex items-center gap-2 text-sm">
                      <Clock3 className="h-4 w-4 text-slate-400" />
                      {new Date(campaign.created_at).toLocaleDateString('zh-CN')}
                    </div>
                  </td>
                  <td className="px-6 py-4">
                    <div className="flex justify-end gap-2">
                      <button
                        onClick={() => openEdit(campaign)}
                        className="inline-flex items-center gap-1 rounded-lg border border-slate-200 px-3 py-1.5 text-sm text-slate-700 hover:bg-slate-50"
                      >
                        <PencilLine className="h-4 w-4" />
                        编辑
                      </button>
                      {campaign.status === 'active' ? (
                        <button
                          onClick={() => pauseMutation.mutate(campaign.id)}
                          disabled={pauseMutation.isPending}
                          className="inline-flex items-center gap-1 rounded-lg border border-amber-200 bg-amber-50 px-3 py-1.5 text-sm text-amber-700 hover:bg-amber-100 disabled:opacity-50"
                        >
                          <Pause className="h-4 w-4" />
                          暂停
                        </button>
                      ) : campaign.status !== 'completed' ? (
                        <button
                          onClick={() => launchMutation.mutate(campaign.id)}
                          disabled={launchMutation.isPending}
                          className="inline-flex items-center gap-1 rounded-lg border border-emerald-200 bg-emerald-50 px-3 py-1.5 text-sm text-emerald-700 hover:bg-emerald-100 disabled:opacity-50"
                        >
                          <Play className="h-4 w-4" />
                          启动
                        </button>
                      ) : null}
                      <button
                        onClick={() => {
                          if (window.confirm(`确定删除活动 "${campaign.name}" 吗？`)) {
                            deleteMutation.mutate(campaign.id)
                          }
                        }}
                        className="rounded-lg px-3 py-1.5 text-sm text-red-600 hover:bg-red-50"
                      >
                        <Trash2 className="h-4 w-4" />
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {showModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/60 p-4">
          <div className="w-full max-w-2xl rounded-3xl bg-white shadow-2xl">
            <div className="flex items-start justify-between gap-4 border-b px-6 py-5">
              <div>
                <h3 className="text-lg font-semibold text-slate-900">{editing ? '编辑活动' : '创建活动'}</h3>
                <p className="text-sm text-slate-500">活动定义会驱动内容、触达和转化执行。</p>
              </div>
              <button onClick={() => setShowModal(false)} className="rounded-full p-2 text-slate-400 hover:bg-slate-100 hover:text-slate-700">
                <X className="h-5 w-5" />
              </button>
            </div>
            <form onSubmit={handleSubmit} className="space-y-4 p-6">
              <div className="grid gap-4 md:grid-cols-2">
                <div>
                  <label className="mb-1 block text-sm font-medium text-slate-700">活动名称</label>
                  <input
                    value={form.name}
                    onChange={(e) => setForm((prev) => ({ ...prev, name: e.target.value }))}
                    required
                    className="w-full rounded-xl border border-slate-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                  />
                </div>
                <div>
                  <label className="mb-1 block text-sm font-medium text-slate-700">类型</label>
                  <select
                    value={form.campaign_type}
                    onChange={(e) => setForm((prev) => ({ ...prev, campaign_type: e.target.value as CampaignFormState['campaign_type'] }))}
                    className="w-full rounded-xl border border-slate-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                  >
                    <option value="email">邮件</option>
                    <option value="social">社交媒体</option>
                    <option value="content">内容营销</option>
                    <option value="ads">付费广告</option>
                  </select>
                </div>
              </div>
              <div>
                <label className="mb-1 block text-sm font-medium text-slate-700">描述</label>
                <textarea
                  value={form.description}
                  onChange={(e) => setForm((prev) => ({ ...prev, description: e.target.value }))}
                  rows={4}
                  className="w-full rounded-2xl border border-slate-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                />
              </div>
              <div className="grid gap-4 md:grid-cols-2">
                <div>
                  <label className="mb-1 block text-sm font-medium text-slate-700">开始日期</label>
                  <input
                    type="date"
                    value={form.start_date}
                    onChange={(e) => setForm((prev) => ({ ...prev, start_date: e.target.value }))}
                    className="w-full rounded-xl border border-slate-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                  />
                </div>
                <div>
                  <label className="mb-1 block text-sm font-medium text-slate-700">结束日期</label>
                  <input
                    type="date"
                    value={form.end_date}
                    onChange={(e) => setForm((prev) => ({ ...prev, end_date: e.target.value }))}
                    className="w-full rounded-xl border border-slate-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                  />
                </div>
              </div>
              <div className="flex justify-end gap-3 pt-2">
                <button type="button" onClick={() => setShowModal(false)} className="rounded-xl border border-slate-200 px-4 py-2.5 text-sm font-medium text-slate-700 hover:bg-slate-50">
                  取消
                </button>
                <button
                  type="submit"
                  disabled={createMutation.isPending || updateMutation.isPending}
                  className="rounded-xl bg-brand-600 px-4 py-2.5 text-sm font-medium text-white hover:bg-brand-700 disabled:opacity-50"
                >
                  {editing ? '保存更改' : createMutation.isPending ? '创建中...' : '创建活动'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  )
}
