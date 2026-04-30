import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/services/api'
import type { Content } from '@/types'
import { Plus, FileText, Trash2 } from 'lucide-react'
import { useState } from 'react'

export function ContentsPage() {
  const queryClient = useQueryClient()
  const [showModal, setShowModal] = useState(false)

  const { data, isLoading } = useQuery({
    queryKey: ['contents'],
    queryFn: async () => {
      const { data: resp } = await api.get('/contents')
      return resp.data as { data: Content[]; total: number }
    },
  })

  const deleteMutation = useMutation({
    mutationFn: async (id: string) => {
      await api.delete(`/contents/${id}`)
    },
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['contents'] }),
  })

  const handleDelete = (content: Content) => {
    if (window.confirm(`确定要删除内容 "${content.name}" 吗？`)) {
      deleteMutation.mutate(content.id)
    }
  }

  const getTypeLabel = (type: string) => {
    const labels: Record<string, string> = {
      email: '邮件',
      social: '社交媒体',
      article: '文章',
      image: '图片',
      video: '视频',
    }
    return labels[type] || type
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">内容管理</h1>
          <p className="text-gray-500 mt-1">管理您的营销内容资产</p>
        </div>
        <button
          onClick={() => setShowModal(true)}
          className="flex items-center gap-2 px-4 py-2 bg-brand-600 text-white rounded-lg hover:bg-brand-700"
        >
          <Plus className="h-4 w-4" />
          创建内容
        </button>
      </div>

      <div className="bg-white rounded-xl border overflow-hidden">
        {isLoading ? (
          <div className="p-8 text-center text-gray-500">加载中...</div>
        ) : !data?.data.length ? (
          <div className="p-8 text-center">
            <FileText className="h-12 w-12 text-gray-300 mx-auto mb-4" />
            <p className="text-gray-500">暂无内容</p>
            <button
              onClick={() => setShowModal(true)}
              className="mt-4 text-brand-600 hover:text-brand-700"
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
                <tr key={content.id} className="hover:bg-gray-50">
                  <td className="px-6 py-4">
                    <div className="flex items-center gap-3">
                      <div className="p-2 bg-brand-50 rounded-lg">
                        <FileText className="h-4 w-4 text-brand-600" />
                      </div>
                      <span className="font-medium text-gray-900">{content.name}</span>
                    </div>
                  </td>
                  <td className="px-6 py-4 text-gray-500">{getTypeLabel(content.content_type)}</td>
                  <td className="px-6 py-4">
                    <span className={`px-2 py-1 text-xs rounded-full ${
                      content.status === 'published' ? 'bg-green-100 text-green-700' :
                      content.status === 'draft' ? 'bg-gray-100 text-gray-700' :
                      'bg-yellow-100 text-yellow-700'
                    }`}>
                      {content.status === 'published' ? '已发布' : content.status === 'draft' ? '草稿' : content.status}
                    </span>
                  </td>
                  <td className="px-6 py-4 text-gray-500">
                    {new Date(content.created_at).toLocaleDateString('zh-CN')}
                  </td>
                  <td className="px-6 py-4 text-right">
                    <button
                      onClick={() => handleDelete(content)}
                      disabled={deleteMutation.isPending}
                      className="inline-flex items-center gap-1 text-sm text-red-600 hover:text-red-800 disabled:opacity-50"
                    >
                      <Trash2 className="h-4 w-4" />
                    </button>
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
            <h3 className="text-lg font-semibold mb-4">创建内容</h3>
            <p className="text-gray-500 text-sm">内容管理功能开发中...</p>
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