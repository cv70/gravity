import { useMemo, useState, type FormEvent } from 'react'
import { Link } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { ChevronRight, Layers3, PencilLine, Plus, Sparkles, Trash2, X } from 'lucide-react'

import { governanceService } from '@/services/governance'
import { Button } from '@/components/ui/button'
import type { Segment, SegmentCreatePayload, SegmentPreviewPayload, SegmentUpdatePayload } from '@/types'

type StatusFilter = 'all' | 'active' | 'inactive'
type Mode = 'create' | 'edit'
type TemplateKey = 'welcome' | 'hot_lead' | 'recovery' | 'vip' | 'custom'

type FormState = {
  name: string
  status: 'active' | 'inactive'
  is_dynamic: boolean
  definition: string
}

const emptyForm: FormState = {
  name: '',
  status: 'active',
  is_dynamic: true,
  definition: JSON.stringify(
    {
      lifecycle_stage: 'new',
      subscribed: true,
      tags_any: ['welcome'],
    },
    null,
    2,
  ),
}

const TEMPLATE_PRESETS: Record<
  Exclude<TemplateKey, 'custom'>,
  { label: string; name: string; definition: Record<string, unknown>; description: string }
> = {
  welcome: {
    label: '新用户欢迎',
    name: '新用户欢迎人群',
    definition: { lifecycle_stage: 'new', subscribed: true, tags_any: ['welcome'] },
    description: '适合首登、注册后欢迎流和教育型触达。',
  },
  hot_lead: {
    label: '高意向线索',
    name: '高意向线索人群',
    definition: { lifecycle_stage: 'lead', subscribed: true, tags_any: ['hot', 'demo'] },
    description: '适合销售协同、预约演示和重点跟进。',
  },
  recovery: {
    label: '沉默召回',
    name: '沉默召回人群',
    definition: { lifecycle_stage: 'inactive', subscribed: true, tags_any: ['lapsed', 'recovery'] },
    description: '适合老客唤醒、召回和复购提醒。',
  },
  vip: {
    label: '会员复购',
    name: '会员复购人群',
    definition: { lifecycle_stage: 'customer', subscribed: true, tags_any: ['vip', 'member'] },
    description: '适合会员日、补充购、权益提醒。',
  },
}

function safeParseDefinition(value: string): Record<string, unknown> {
  if (!value.trim()) return {}
  return JSON.parse(value) as Record<string, unknown>
}

export function SegmentsPage() {
  const queryClient = useQueryClient()
  const [status, setStatus] = useState<StatusFilter>('all')
  const [showModal, setShowModal] = useState(false)
  const [mode, setMode] = useState<Mode>('create')
  const [editing, setEditing] = useState<Segment | null>(null)
  const [form, setForm] = useState<FormState>(emptyForm)
  const [preview, setPreview] = useState<{
    matching_count: number
    sample_contacts: Array<{ id: string; email: string; name: string; lifecycle_stage: string; tags: string[] }>
  } | null>(null)
  const [formError, setFormError] = useState<string | null>(null)
  const [templateKey, setTemplateKey] = useState<TemplateKey>('custom')

  const { data, isLoading } = useQuery({
    queryKey: ['governance', 'segments'],
    queryFn: () => governanceService.segments({ page: 1, limit: 100 }),
  })

  const createMutation = useMutation({
    mutationFn: (payload: SegmentCreatePayload) => governanceService.createSegment(payload),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['governance', 'segments'] })
      closeModal()
    },
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, payload }: { id: string; payload: SegmentUpdatePayload }) =>
      governanceService.updateSegment(id, payload),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['governance', 'segments'] })
      closeModal()
    },
  })

  const deleteMutation = useMutation({
    mutationFn: governanceService.deleteSegment,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['governance', 'segments'] })
    },
  })

  const previewMutation = useMutation({
    mutationFn: ({ id, payload }: { id: string; payload?: SegmentPreviewPayload }) =>
      governanceService.previewSegment(id, payload),
    onSuccess: (result) => setPreview(result),
  })

  const segments = useMemo(() => {
    const items = data?.data ?? []
    if (status === 'all') return items
    return items.filter((item) => item.status === status)
  }, [data?.data, status])

  const stats = useMemo(() => {
    const items = data?.data ?? []
    return [
      { label: '总分群', value: items.length },
      { label: '动态分群', value: items.filter((item) => item.is_dynamic).length },
      { label: '启用中', value: items.filter((item) => item.status === 'active').length },
      { label: '静态分群', value: items.filter((item) => !item.is_dynamic).length },
    ]
  }, [data?.data])

  const definitionSummary = useMemo(() => {
    try {
      const parsed = safeParseDefinition(form.definition)
      const entries = Object.entries(parsed)
      if (entries.length === 0) {
        return ['未配置规则：默认匹配全部联系人']
      }

      return entries.map(([key, value]) => {
        if (Array.isArray(value)) {
          return `${key}: ${value.map((item) => String(item)).join(' / ')}`
        }
        if (value && typeof value === 'object') {
          return `${key}: 复杂对象`
        }
        return `${key}: ${String(value)}`
      })
    } catch {
      return null
    }
  }, [form.definition])

  const openCreate = () => {
    setMode('create')
    setEditing(null)
    setForm(emptyForm)
    setPreview(null)
    setFormError(null)
    setTemplateKey('custom')
    setShowModal(true)
  }

  const openEdit = (segment: Segment) => {
    setMode('edit')
    setEditing(segment)
    setForm({
      name: segment.name,
      status: segment.status === 'inactive' ? 'inactive' : 'active',
      is_dynamic: segment.is_dynamic,
      definition: JSON.stringify(segment.definition, null, 2),
    })
    setPreview(null)
    setFormError(null)
    setTemplateKey('custom')
    setShowModal(true)
  }

  const closeModal = () => {
    setShowModal(false)
    setEditing(null)
    setForm(emptyForm)
    setPreview(null)
    setFormError(null)
    setTemplateKey('custom')
    setMode('create')
  }

  const handlePreview = async () => {
    try {
      setFormError(null)
      const definition = safeParseDefinition(form.definition)
      await previewMutation.mutateAsync({
        id: editing?.id || '00000000-0000-0000-0000-000000000000',
        payload: { definition },
      })
    } catch {
      setFormError('分群定义必须是合法 JSON。')
    }
  }

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    try {
      setFormError(null)
      const definition = safeParseDefinition(form.definition)
      if (mode === 'edit' && editing) {
        await updateMutation.mutateAsync({
          id: editing.id,
          payload: {
            name: form.name,
            definition,
            is_dynamic: form.is_dynamic,
            status: form.status,
          },
        })
      } else {
        await createMutation.mutateAsync({
          name: form.name,
          definition,
          is_dynamic: form.is_dynamic,
          status: form.status,
        })
      }
    } catch {
      setFormError('分群定义必须是合法 JSON。')
    }
  }

  const handleDelete = async (segment: Segment) => {
    if (!window.confirm(`确认删除分群「${segment.name}」吗？`)) return
    await deleteMutation.mutateAsync(segment.id)
  }

  const applyTemplate = (key: Exclude<TemplateKey, 'custom'>) => {
    const preset = TEMPLATE_PRESETS[key]
    setTemplateKey(key)
    const nextDefinition = JSON.stringify(preset.definition, null, 2)
    setForm((current) => ({
      ...current,
      name: preset.name,
      status: 'active',
      is_dynamic: true,
      definition: nextDefinition,
    }))
    void previewMutation.mutateAsync({
      id: editing?.id || '00000000-0000-0000-0000-000000000000',
      payload: { definition: preset.definition },
    })
  }

  return (
    <div className="space-y-6">
      <div className="relative overflow-hidden rounded-3xl bg-slate-950 text-white shadow-xl border border-slate-800">
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_top_right,rgba(34,211,238,0.22),transparent_30%),radial-gradient(circle_at_bottom_left,rgba(59,130,246,0.18),transparent_28%)]" />
        <div className="relative p-6 lg:p-8">
          <div className="inline-flex items-center gap-2 rounded-full bg-white/10 px-3 py-1 text-xs text-cyan-100">
            <Layers3 className="h-4 w-4" />
            分群中心
          </div>
          <h1 className="mt-4 text-3xl font-bold tracking-tight">管理动态人群、圈选规则和预览结果</h1>
          <p className="mt-3 max-w-3xl text-sm lg:text-base text-slate-300">
            这里专注于受众定义和分群可见性，不再混入审批与审计内容，方便运营团队单独使用。
          </p>
          <div className="mt-6 flex flex-wrap gap-3">
            <Button onClick={openCreate} variant="brand">
              <Plus className="h-4 w-4" />
              新建分群
            </Button>
            <Link
              to="/governance"
              className="inline-flex items-center gap-2 rounded-xl border border-white/15 bg-white/5 px-4 py-2.5 text-sm font-medium text-white transition hover:bg-white/10"
            >
              返回治理总览
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
            <h2 className="text-lg font-semibold text-slate-900">分群列表</h2>
            <p className="text-sm text-slate-500">基于标签、生命周期和行为定义的目标人群</p>
          </div>
          <div className="inline-flex items-center gap-2 rounded-xl border border-slate-200 bg-slate-50 p-1">
            {(['all', 'active', 'inactive'] as const).map((item) => (
              <Button
                key={item}
                onClick={() => setStatus(item)}
                variant="secondary"
                size="sm"
                className={`rounded-lg px-3 py-1.5 text-sm font-medium transition ${
                  status === item ? 'bg-slate-950 text-white' : 'text-slate-600 hover:bg-white'
                }`}
              >
                {item === 'all' ? '全部' : item === 'active' ? '启用中' : '停用'}
              </Button>
            ))}
          </div>
        </div>

        <div className="grid gap-4 p-6 lg:grid-cols-2">
          {isLoading ? (
            <div className="py-10 text-center text-slate-500">正在加载分群...</div>
          ) : segments.length > 0 ? (
            segments.map((segment) => (
            <div
              key={segment.id}
              className="rounded-2xl border border-slate-200 bg-slate-50 p-5 transition hover:-translate-y-0.5 hover:border-cyan-200 hover:shadow-md"
            >
              <div className="flex items-start justify-between gap-4">
                <div>
                  <p className="font-semibold text-slate-900">{segment.name}</p>
                  <p className="mt-1 text-sm text-slate-500">
                    {segment.status} · {segment.is_dynamic ? '动态分群' : '静态分群'}
                  </p>
                </div>
                <span className="rounded-full bg-cyan-100 px-2.5 py-1 text-xs font-medium text-cyan-700">
                  Segment
                </span>
              </div>
                <div className="mt-4 rounded-xl bg-white p-4 text-sm text-slate-600">
                  <p className="font-medium text-slate-900">定义</p>
                  <pre className="mt-2 overflow-auto text-xs leading-6 text-slate-500">
                    {JSON.stringify(segment.definition, null, 2)}
                  </pre>
                </div>
                <div className="mt-4 flex items-center gap-2">
                  <Button onClick={() => openEdit(segment)} variant="secondary" size="sm">
                    <PencilLine className="h-4 w-4" />
                    编辑
                  </Button>
                  <Button onClick={() => handleDelete(segment)} variant="destructive" size="sm">
                    <Trash2 className="h-4 w-4" />
                    删除
                  </Button>
                  <Button
                    onClick={async () => {
                      await previewMutation.mutateAsync({ id: segment.id, payload: {} })
                      setEditing(segment)
                      setMode('edit')
                      setShowModal(true)
                    }}
                    variant="secondary"
                    size="sm"
                  >
                    <Sparkles className="h-4 w-4" />
                    预览
                  </Button>
                </div>
              </div>
            ))
          ) : (
            <div className="col-span-full rounded-2xl bg-slate-50 p-10 text-center text-slate-500">
              当前没有分群。
            </div>
          )}
        </div>
      </div>

      <div className="rounded-2xl border border-cyan-100 bg-cyan-50/60 p-5">
        <div className="flex items-center gap-2 text-sm font-semibold text-slate-800">
          <Sparkles className="h-4 w-4 text-cyan-600" />
          使用建议
        </div>
        <p className="mt-2 text-sm leading-6 text-slate-600">
          将欢迎流、召回流和高意向线索分别拆成独立分群，可以直接复用到自动化工作流和审批策略中。
        </p>
      </div>

      {showModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/70 p-4 backdrop-blur-sm">
          <div className="max-h-[90vh] w-full max-w-6xl overflow-hidden rounded-3xl bg-white shadow-2xl">
            <div className="flex items-center justify-between border-b px-6 py-4">
              <div>
                <h3 className="text-lg font-semibold text-slate-900">
                  {mode === 'edit' ? '编辑分群' : '新建分群'}
                </h3>
                <p className="text-sm text-slate-500">配置分群定义并预览匹配结果</p>
              </div>
              <Button onClick={closeModal} variant="ghost" size="sm" className="rounded-lg p-2 text-slate-500">
                <X className="h-5 w-5" />
              </Button>
            </div>

            <div className="grid gap-0 lg:grid-cols-[1.15fr_0.85fr]">
              <form onSubmit={handleSubmit} className="space-y-5 overflow-y-auto p-6">
                <div>
                  <label className="mb-2 block text-sm font-medium text-slate-700">规则向导</label>
                  <div className="grid gap-3 md:grid-cols-2">
                    {(Object.entries(TEMPLATE_PRESETS) as Array<
                      [Exclude<TemplateKey, 'custom'>, (typeof TEMPLATE_PRESETS)[Exclude<TemplateKey, 'custom'>]]
                    >).map(([key, preset]) => (
                      <Button
                        key={key}
                        type="button"
                        onClick={() => applyTemplate(key)}
                        variant="secondary"
                        className={`h-auto w-full flex-col items-start rounded-2xl border p-4 text-left transition ${
                          templateKey === key
                            ? 'border-cyan-300 bg-cyan-50 shadow-sm'
                            : 'border-slate-200 bg-white hover:bg-slate-50'
                        }`}
                      >
                        <div className="flex items-center justify-between gap-3">
                          <div>
                            <p className="font-semibold text-slate-900">{preset.label}</p>
                            <p className="mt-1 text-xs text-slate-500">{preset.description}</p>
                          </div>
                          <ChevronRight className="h-4 w-4 text-slate-400" />
                        </div>
                      </Button>
                    ))}
                  </div>
                  <div className="mt-3 rounded-2xl bg-slate-50 p-4 text-xs leading-6 text-slate-500">
                    选一个模板，系统会自动生成常见分群规则；你仍然可以在下方 JSON 编辑器里继续微调。
                  </div>
                </div>

                <div>
                  <label className="mb-1 block text-sm font-medium text-slate-700">分群名称</label>
                  <input
                    value={form.name}
                    onChange={(e) => setForm((prev) => ({ ...prev, name: e.target.value }))}
                    className="w-full rounded-xl border border-slate-200 bg-white px-3 py-2.5 outline-none transition focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                    placeholder="例如：新用户欢迎流人群"
                    required
                  />
                </div>

                <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
                  <div>
                    <label className="mb-1 block text-sm font-medium text-slate-700">状态</label>
                    <select
                      value={form.status}
                      onChange={(e) =>
                        setForm((prev) => ({ ...prev, status: e.target.value as FormState['status'] }))
                      }
                      className="w-full rounded-xl border border-slate-200 bg-white px-3 py-2.5 outline-none transition focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                    >
                      <option value="active">启用</option>
                      <option value="inactive">停用</option>
                    </select>
                  </div>
                  <div className="flex items-end">
                    <label className="flex w-full items-center justify-between rounded-xl border border-slate-200 px-4 py-3">
                      <span>
                        <span className="block text-sm font-medium text-slate-700">动态分群</span>
                        <span className="text-xs text-slate-500">开启后适合自动化闭环</span>
                      </span>
                      <input
                        type="checkbox"
                        checked={form.is_dynamic}
                        onChange={(e) => setForm((prev) => ({ ...prev, is_dynamic: e.target.checked }))}
                        className="h-5 w-5 rounded border-slate-300 text-cyan-600 focus:ring-cyan-500"
                      />
                    </label>
                  </div>
                </div>

                <div>
                  <label className="mb-1 block text-sm font-medium text-slate-700">分群定义 JSON</label>
                  <textarea
                    value={form.definition}
                    onChange={(e) => setForm((prev) => ({ ...prev, definition: e.target.value }))}
                    rows={14}
                    className="w-full rounded-2xl border border-slate-200 bg-slate-950 px-4 py-3 font-mono text-sm text-cyan-50 outline-none transition focus:border-cyan-400 focus:ring-4 focus:ring-cyan-100"
                    placeholder='{"lifecycle_stage":"new","tags_any":["welcome"]}'
                  />
                  <p className="mt-2 text-xs leading-5 text-slate-500">
                    支持 `search`、`lifecycle_stage`、`subscribed`、`tags_any` 和 `tags_all` 等字段。
                  </p>
                </div>

                <div className="rounded-2xl border border-slate-200 bg-slate-50 p-4">
                  <div className="flex items-center justify-between gap-3">
                    <p className="text-sm font-semibold text-slate-800">规则预览</p>
                    <span className="text-xs text-slate-500">
                      {definitionSummary ? `${definitionSummary.length} 条条件` : '格式有误'}
                    </span>
                  </div>
                  {definitionSummary ? (
                    <div className="mt-3 flex flex-wrap gap-2">
                      {definitionSummary.map((item) => (
                        <span key={item} className="rounded-full bg-white px-3 py-1 text-xs text-slate-600 shadow-sm">
                          {item}
                        </span>
                      ))}
                    </div>
                  ) : (
                    <p className="mt-2 text-sm text-rose-600">JSON 解析失败，请检查字段和引号是否正确。</p>
                  )}
                </div>

                {formError && <div className="rounded-xl bg-rose-50 p-3 text-sm text-rose-700">{formError}</div>}

                <div className="flex flex-wrap items-center gap-3">
                  <Button type="button" onClick={handlePreview} variant="secondary">
                    <Sparkles className="h-4 w-4" />
                    预览匹配
                  </Button>
                  <Button type="submit" variant="brand">
                    {mode === 'edit' ? '保存修改' : '创建分群'}
                    <ChevronRight className="h-4 w-4" />
                  </Button>
                </div>
              </form>

              <div className="border-t bg-slate-50 p-6 lg:sticky lg:top-0 lg:self-start lg:border-l lg:border-t-0">
                <div className="flex items-center justify-between gap-3">
                  <h4 className="text-sm font-semibold text-slate-800">预览结果</h4>
                  <span className="rounded-full bg-white px-2.5 py-1 text-xs font-medium text-slate-500 shadow-sm">
                    即时更新
                  </span>
                </div>
                {preview ? (
                  <div className="mt-4 space-y-4">
                    <div className="rounded-2xl bg-white p-4 shadow-sm">
                      <p className="text-sm text-slate-500">匹配人数</p>
                      <p className="mt-1 text-3xl font-bold text-slate-900">{preview.matching_count}</p>
                    </div>
                    <div>
                      <p className="mb-2 text-sm font-medium text-slate-700">样本联系人</p>
                      <div className="space-y-3">
                        {preview.sample_contacts.length > 0 ? (
                          preview.sample_contacts.map((contact) => (
                            <div key={contact.id} className="rounded-xl border border-slate-200 bg-white p-4">
                              <p className="font-medium text-slate-900">{contact.name}</p>
                              <p className="text-sm text-slate-500">{contact.email}</p>
                              <p className="mt-1 text-xs text-slate-400">
                                {contact.lifecycle_stage} · {contact.tags.join(' / ') || '无标签'}
                              </p>
                            </div>
                          ))
                        ) : (
                          <p className="rounded-xl bg-white p-4 text-sm text-slate-500">暂无可展示的样本联系人。</p>
                        )}
                      </div>
                    </div>
                  </div>
                ) : (
                  <div className="mt-4 rounded-2xl border border-dashed border-slate-300 bg-white p-6 text-sm text-slate-500">
                    点击“预览匹配”后会在这里显示匹配人数和样本联系人。
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
