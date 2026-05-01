import { useMemo, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { FileText, Plus, Sparkles, Trash2 } from 'lucide-react'

import { contentsService } from '@/services/contents'
import type { Content, CreateContentPayload } from '@/types'

type ContentFormState = {
  name: string
  content_type: string
  content_json: string
  campaign_id: string
}

const initialForm: ContentFormState = {
  name: '',
  content_type: 'email',
  content_json: JSON.stringify(
    {
      subject: '欢迎加入 Gravity',
      body: '系统会自动帮你管理线索、触达和转化。',
      cta: '立即开始',
    },
    null,
    2,
  ),
  campaign_id: '',
}

export function ContentsPage() {
  const queryClient = useQueryClient()
  const [showModal, setShowModal] = useState(false)
  const [form, setForm] = useState<ContentFormState>(initialForm)

  const { data, isLoading } = useQuery({
    queryKey: ['contents'],
    queryFn: () => contentsService.list(),
  })

  const createMutation = useMutation({
    mutationFn: async (payload: CreateContentPayload) => contentsService.create(payload),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['contents'] })
      setShowModal(false)
      setForm(initialForm)
    },
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => contentsService.remove(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['contents'] })
    },
  })

  const contentTypeLabel = useMemo(
    () =>
      ({
        email: '邮件',
        social: '社媒',
        article: '文章',
        image: '图片',
        video: '视频',
        landing_page: '落地页',
      }) as Record<string, string>,
    [],
  )

  const handleCreate = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const parsed = JSON.parse(form.content_json)
    await createMutation.mutateAsync({
      name: form.name,
      content_type: form.content_type,
      content: parsed,
      campaign_id: form.campaign_id || null,
    })
  }

  const handleDelete = (content: Content) => {
    if (window.confirm(`确定要删除内容 "${content.name}" 吗？`)) {
      deleteMutation.mutate(content.id)
    }
  }

  return (
    <div className="space-y-6">
      <div className="relative overflow-hidden rounded-3xl bg-slate-950 text-white shadow-xl border border-slate-800">
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_top_right,rgba(34,211,238,0.22),transparent_30%),radial-gradient(circle_at_bottom_left,rgba(59,130,246,0.18),transparent_26%)]" />
        <div className="relative flex flex-col gap-4 p-6 lg:p-8">
          <div className="flex items-start justify-between gap-4">
            <div>
              <div className="inline-flex items-center gap-2 rounded-full bg-white/10 px-3 py-1 text-xs text-cyan-100">
                <Sparkles className="h-4 w-4" />
                内容工厂
              </div>
              <h1 className="mt-4 text-3xl font-bold tracking-tight">统一管理营销素材，内容直接进入自动化链路</h1>
              <p className="mt-3 max-w-3xl text-sm lg:text-base text-slate-300">
                文案、脚本、邮件、落地页、社媒内容都可以在这里统一生成、存储、复用和分发。
              </p>
            </div>
            <button
              onClick={() => setShowModal(true)}
              className="inline-flex items-center gap-2 rounded-xl bg-cyan-400 px-4 py-2 text-sm font-semibold text-slate-950 shadow-lg shadow-cyan-500/20 transition hover:bg-cyan-300"
            >
              <Plus className="h-4 w-4" />
              创建内容
            </button>
          </div>
        </div>
      </div>

      <div className="bg-white rounded-2xl border border-gray-200 shadow-sm overflow-hidden">
        <div className="px-6 py-4 border-b flex items-center justify-between">
          <div>
            <h2 className="text-lg font-semibold text-gray-900">内容资产</h2>
            <p className="text-sm text-gray-500">内容会作为工作流和渠道层的标准输入</p>
          </div>
          <FileText className="h-5 w-5 text-brand-600" />
        </div>

        {isLoading ? (
          <div className="p-8 text-center text-gray-500">加载中...</div>
        ) : !data?.data.length ? (
          <div className="p-10 text-center">
            <FileText className="h-12 w-12 text-gray-300 mx-auto mb-4" />
            <p className="text-gray-500">暂无内容，先创建你的第一份素材</p>
            <button
              onClick={() => setShowModal(true)}
              className="mt-4 text-brand-600 hover:text-brand-700 font-medium"
            >
              创建第一个内容
            </button>
          </div>
        ) : (
          <table className="w-full">
            <thead className="bg-gray-50 border-b">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">名称</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">类型</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">状态</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">创建时间</th>
                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase">操作</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200">
              {data.data.map((content) => (
                <tr key={content.id} className="hover:bg-gray-50/80">
                  <td className="px-6 py-4">
                    <div className="flex items-center gap-3">
                      <div className="p-2 bg-brand-50 rounded-lg">
                        <FileText className="h-4 w-4 text-brand-600" />
                      </div>
                      <div>
                        <p className="font-medium text-gray-900">{content.name}</p>
                        <p className="text-xs text-gray-400">campaign: {content.campaign_id || '未关联'}</p>
                      </div>
                    </div>
                  </td>
                  <td className="px-6 py-4 text-gray-500">{contentTypeLabel[content.content_type] || content.content_type}</td>
                  <td className="px-6 py-4">
                    <span className={`px-2 py-1 text-xs rounded-full ${
                      content.status === 'published'
                        ? 'bg-green-100 text-green-700'
                        : content.status === 'draft'
                          ? 'bg-gray-100 text-gray-700'
                          : 'bg-yellow-100 text-yellow-700'
                    }`}>
                      {content.status === 'published' ? '已发布' : content.status === 'draft' ? '草稿' : content.status}
                    </span>
                  </td>
                  <td className="px-6 py-4 text-gray-500">{new Date(content.created_at).toLocaleString('zh-CN')}</td>
                  <td className="px-6 py-4 text-right">
                    <button
                      onClick={() => handleDelete(content)}
                      disabled={deleteMutation.isPending}
                      className="inline-flex items-center gap-1 text-sm text-red-600 hover:text-red-800 disabled:opacity-50"
                    >
                      <Trash2 className="h-4 w-4" />
                      删除
                    </button>
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
            <div className="border-b px-6 py-4">
              <h3 className="text-lg font-semibold text-gray-900">创建内容</h3>
              <p className="text-sm text-gray-500">内容将成为后续工作流、渠道发布和 A/B 实验的基础素材。</p>
            </div>
            <form onSubmit={handleCreate} className="space-y-4 p-6">
              <div className="grid gap-4 md:grid-cols-2">
                <div>
                  <label className="mb-1 block text-sm font-medium text-gray-700">名称</label>
                  <input
                    value={form.name}
                    onChange={(e) => setForm((prev) => ({ ...prev, name: e.target.value }))}
                    required
                    className="w-full rounded-xl border px-3 py-2 focus:border-brand-500 focus:outline-none focus:ring-2 focus:ring-brand-100"
                  />
                </div>
                <div>
                  <label className="mb-1 block text-sm font-medium text-gray-700">类型</label>
                  <select
                    value={form.content_type}
                    onChange={(e) => setForm((prev) => ({ ...prev, content_type: e.target.value }))}
                    className="w-full rounded-xl border px-3 py-2 focus:border-brand-500 focus:outline-none focus:ring-2 focus:ring-brand-100"
                  >
                    <option value="email">邮件</option>
                    <option value="social">社媒</option>
                    <option value="article">文章</option>
                    <option value="image">图片</option>
                    <option value="video">视频</option>
                    <option value="landing_page">落地页</option>
                  </select>
                </div>
              </div>
              <div>
                <label className="mb-1 block text-sm font-medium text-gray-700">关联活动 ID（可选）</label>
                <input
                  value={form.campaign_id}
                  onChange={(e) => setForm((prev) => ({ ...prev, campaign_id: e.target.value }))}
                  placeholder="UUID"
                  className="w-full rounded-xl border px-3 py-2 focus:border-brand-500 focus:outline-none focus:ring-2 focus:ring-brand-100"
                />
              </div>
              <div>
                <label className="mb-1 block text-sm font-medium text-gray-700">内容 JSON</label>
                <textarea
                  value={form.content_json}
                  onChange={(e) => setForm((prev) => ({ ...prev, content_json: e.target.value }))}
                  rows={12}
                  className="w-full rounded-2xl border px-3 py-2 font-mono text-sm focus:border-brand-500 focus:outline-none focus:ring-2 focus:ring-brand-100"
                />
              </div>
              <div className="flex justify-end gap-3 pt-2">
                <button type="button" onClick={() => setShowModal(false)} className="rounded-xl border px-4 py-2 text-sm">
                  取消
                </button>
                <button
                  type="submit"
                  disabled={createMutation.isPending}
                  className="rounded-xl bg-brand-600 px-4 py-2 text-sm font-medium text-white disabled:opacity-50"
                >
                  {createMutation.isPending ? '创建中...' : '创建内容'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  )
}
