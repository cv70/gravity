import { useEffect, useMemo, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { CheckCircle2, PencilLine, Plus, Search, Tag, Users, X } from 'lucide-react'

import { contactsService } from '@/services/contacts'
import { Button } from '@/components/ui/button'
import type { Contact, ContactCreatePayload } from '@/types'

type ContactFormState = {
  email: string
  name: string
  phone: string
  tags: string
  subscribed: boolean
}

const emptyForm: ContactFormState = {
  email: '',
  name: '',
  phone: '',
  tags: '',
  subscribed: true,
}

export function ContactsPage() {
  const queryClient = useQueryClient()
  const [searchInput, setSearchInput] = useState('')
  const [debouncedSearch, setDebouncedSearch] = useState('')
  const [page, setPage] = useState(1)
  const [showModal, setShowModal] = useState(false)
  const [editing, setEditing] = useState<Contact | null>(null)
  const [form, setForm] = useState<ContactFormState>(emptyForm)

  useEffect(() => {
    const timer = setTimeout(() => setDebouncedSearch(searchInput), 300)
    return () => clearTimeout(timer)
  }, [searchInput])

  const closeModal = () => {
    setShowModal(false)
    setEditing(null)
    setForm(emptyForm)
  }

  const { data, isLoading } = useQuery({
    queryKey: ['contacts', page, debouncedSearch],
    queryFn: () => contactsService.list({ page, limit: 20, search: debouncedSearch || undefined }),
  })

  const createMutation = useMutation({
    mutationFn: (payload: ContactCreatePayload) => contactsService.create(payload),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['contacts'] })
      closeModal()
    },
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, payload }: { id: string; payload: ContactCreatePayload }) => contactsService.update(id, payload),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['contacts'] })
      closeModal()
    },
  })

  const deleteMutation = useMutation({
    mutationFn: contactsService.delete,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['contacts'] })
    },
  })

  const stats = useMemo(() => {
    const items = data?.data ?? []
    const subscribed = items.filter((item) => item.subscribed).length
    const tagged = items.filter((item) => item.tags.length > 0).length
    const withPhone = items.filter((item) => Boolean(item.phone)).length
    return [
      { label: '总联系人', value: data?.total ?? 0, hint: '统一的线索和联系人池', icon: Users },
      { label: '已订阅', value: subscribed, hint: '允许自动触达的人群', icon: CheckCircle2 },
      { label: '有标签', value: tagged, hint: '可以直接进入分群', icon: Tag },
      { label: '有电话', value: withPhone, hint: '适合销售跟进', icon: PencilLine },
    ]
  }, [data])

  const contacts = data?.data ?? []

  const openCreate = () => {
    setEditing(null)
    setForm(emptyForm)
    setShowModal(true)
  }

  const openEdit = (contact: Contact) => {
    setEditing(contact)
    setForm({
      email: contact.email,
      name: contact.name,
      phone: contact.phone || '',
      tags: contact.tags.join(', '),
      subscribed: contact.subscribed,
    })
    setShowModal(true)
  }

  const handleSearchInput = (value: string) => {
    setSearchInput(value)
    setPage(1)
  }

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const payload: ContactCreatePayload = {
      email: form.email,
      name: form.name,
      phone: form.phone || undefined,
      subscribed: form.subscribed,
      tags: form.tags
        .split(',')
        .map((tag) => tag.trim())
        .filter(Boolean),
      attributes: {},
    }

    if (editing) {
      await updateMutation.mutateAsync({ id: editing.id, payload })
    } else {
      await createMutation.mutateAsync(payload)
    }
  }

  const handleDelete = async (contact: Contact) => {
    if (window.confirm(`确定要删除联系人 "${contact.name}" 吗？此操作不可撤销。`)) {
      deleteMutation.mutate(contact.id)
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
                <Users className="h-4 w-4" />
                客户与线索中心
              </div>
              <h1 className="mt-4 text-3xl font-bold tracking-tight">联系人、线索、标签和订阅状态统一管理</h1>
              <p className="mt-3 max-w-3xl text-sm lg:text-base text-slate-300">
                支持搜索、编辑、打标和订阅管理，让后续自动化分群和触达有可靠输入。
              </p>
            </div>
            <Button
              onClick={openCreate}
              variant="brand"
            >
              <Plus className="h-4 w-4" />
              添加联系人
            </Button>
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
                type="text"
                placeholder="搜索姓名、邮箱或电话"
                value={searchInput}
                onChange={(e) => handleSearchInput(e.target.value)}
                className="w-full rounded-xl border border-slate-200 bg-white py-3 pl-10 pr-4 outline-none transition focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
              />
          </div>
          <div className="flex items-center gap-3">
            <Button
              onClick={() => queryClient.invalidateQueries({ queryKey: ['contacts'] })}
              variant="secondary"
            >
              刷新列表
            </Button>
          </div>
        </div>
      </div>

      <div className="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm">
        {isLoading ? (
          <div className="p-10 text-center text-slate-500">加载中...</div>
        ) : contacts.length === 0 ? (
          <div className="p-10 text-center text-slate-500">暂无联系人，先添加第一个线索。</div>
        ) : (
          <table className="w-full">
            <thead className="bg-slate-50">
              <tr className="border-b border-slate-200">
                <th className="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-500">姓名</th>
                <th className="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-500">邮箱</th>
                <th className="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-500">电话</th>
                <th className="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-500">标签</th>
                <th className="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-500">订阅</th>
                <th className="px-6 py-3 text-right text-xs font-semibold uppercase tracking-wide text-slate-500">操作</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-200">
              {contacts.map((contact) => (
                <tr key={contact.id} className="hover:bg-slate-50/80">
                  <td className="px-6 py-4">
                    <div className="flex items-center gap-3">
                      <div className="flex h-10 w-10 items-center justify-center rounded-full bg-brand-50 font-semibold text-brand-700">
                        {contact.name?.charAt(0).toUpperCase() || '?'}
                      </div>
                      <div>
                        <p className="font-semibold text-slate-900">{contact.name}</p>
                        <p className="text-xs text-slate-400">ID: {contact.id.slice(0, 8)}</p>
                      </div>
                    </div>
                  </td>
                  <td className="px-6 py-4 text-slate-600">{contact.email}</td>
                  <td className="px-6 py-4 text-slate-600">{contact.phone || '-'}</td>
                  <td className="px-6 py-4">
                    <div className="flex flex-wrap gap-1.5">
                      {contact.tags.length ? contact.tags.map((tag) => (
                        <span key={tag} className="inline-flex items-center gap-1 rounded-full bg-slate-100 px-2.5 py-1 text-xs text-slate-700">
                          <Tag className="h-3 w-3" />
                          {tag}
                        </span>
                      )) : <span className="text-sm text-slate-400">未打标</span>}
                    </div>
                  </td>
                  <td className="px-6 py-4">
                    <span className={`rounded-full px-2.5 py-1 text-xs font-medium ${
                      contact.subscribed ? 'bg-emerald-100 text-emerald-700' : 'bg-slate-100 text-slate-600'
                    }`}>
                      {contact.subscribed ? '已订阅' : '已退订'}
                    </span>
                  </td>
                  <td className="px-6 py-4">
                    <div className="flex justify-end gap-3">
                      <Button
                        onClick={() => openEdit(contact)}
                        variant="secondary"
                        size="sm"
                      >
                        <PencilLine className="h-4 w-4" />
                        编辑
                      </Button>
                      <Button
                        onClick={() => handleDelete(contact)}
                        disabled={deleteMutation.isPending}
                        variant="ghost"
                        className="rounded-lg px-3 py-1.5 text-sm text-red-600 hover:bg-red-50"
                      >
                        删除
                      </Button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {data && data.total > 20 && (
        <div className="flex items-center justify-center gap-2">
          <Button
            onClick={() => setPage((p) => Math.max(1, p - 1))}
            disabled={page === 1}
            variant="secondary"
          >
            上一页
          </Button>
          <span className="px-3 py-2 text-sm text-slate-500">第 {page} 页</span>
          <Button
            onClick={() => setPage((p) => p + 1)}
            disabled={data.data.length < 20}
            variant="secondary"
          >
            下一页
          </Button>
        </div>
      )}

      {showModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/60 p-4">
          <div className="w-full max-w-2xl rounded-3xl bg-white shadow-2xl">
            <div className="flex items-start justify-between gap-4 border-b px-6 py-5">
              <div>
                <h3 className="text-lg font-semibold text-slate-900">{editing ? '编辑联系人' : '添加联系人'}</h3>
                <p className="text-sm text-slate-500">联系人会用于分群、评分、触达和销售协同。</p>
              </div>
              <Button onClick={closeModal} variant="ghost" size="sm" className="rounded-full p-2 text-slate-400">
                <X className="h-5 w-5" />
              </Button>
            </div>

            <form onSubmit={handleSubmit} className="space-y-4 p-6">
              <div className="grid gap-4 md:grid-cols-2">
                <div>
                  <label className="mb-1 block text-sm font-medium text-slate-700">邮箱</label>
                  <input
                    value={form.email}
                    onChange={(e) => setForm((prev) => ({ ...prev, email: e.target.value }))}
                    type="email"
                    required
                    className="w-full rounded-xl border border-slate-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                  />
                </div>
                <div>
                  <label className="mb-1 block text-sm font-medium text-slate-700">姓名</label>
                  <input
                    value={form.name}
                    onChange={(e) => setForm((prev) => ({ ...prev, name: e.target.value }))}
                    required
                    className="w-full rounded-xl border border-slate-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                  />
                </div>
              </div>
              <div className="grid gap-4 md:grid-cols-2">
                <div>
                  <label className="mb-1 block text-sm font-medium text-slate-700">电话</label>
                  <input
                    value={form.phone}
                    onChange={(e) => setForm((prev) => ({ ...prev, phone: e.target.value }))}
                    className="w-full rounded-xl border border-slate-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                  />
                </div>
                <div>
                  <label className="mb-1 block text-sm font-medium text-slate-700">订阅状态</label>
                  <select
                    value={form.subscribed ? 'yes' : 'no'}
                    onChange={(e) => setForm((prev) => ({ ...prev, subscribed: e.target.value === 'yes' }))}
                    className="w-full rounded-xl border border-slate-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                  >
                    <option value="yes">已订阅</option>
                    <option value="no">已退订</option>
                  </select>
                </div>
              </div>
              <div>
                <label className="mb-1 block text-sm font-medium text-slate-700">标签（逗号分隔）</label>
                <input
                  value={form.tags}
                  onChange={(e) => setForm((prev) => ({ ...prev, tags: e.target.value }))}
                  placeholder="高意向, 广告来源, 7天未跟进"
                  className="w-full rounded-xl border border-slate-200 px-4 py-3 outline-none focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                />
              </div>
              <div className="flex justify-end gap-3 pt-2">
                <Button type="button" onClick={closeModal} variant="secondary">
                  取消
                </Button>
                <Button
                  type="submit"
                  disabled={createMutation.isPending || updateMutation.isPending}
                  variant="brand"
                >
                  {editing ? '保存更改' : createMutation.isPending ? '创建中...' : '创建联系人'}
                </Button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  )
}
