import { useState } from 'react'
import { GitBranch, Plus, Play, Pause, Trash2 } from 'lucide-react'

interface Workflow {
  id: string
  name: string
  status: 'draft' | 'active' | 'paused'
  trigger: string
  steps_count: number
  created_at: string
}

const mockWorkflows: Workflow[] = []

export function WorkflowsPage() {
  const [showModal, setShowModal] = useState(false)
  const [workflows] = useState<Workflow[]>(mockWorkflows)

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'active':
        return <span className="px-2 py-1 text-xs rounded-full bg-green-100 text-green-700">运行中</span>
      case 'paused':
        return <span className="px-2 py-1 text-xs rounded-full bg-yellow-100 text-yellow-700">已暂停</span>
      case 'draft':
        return <span className="px-2 py-1 text-xs rounded-full bg-gray-100 text-gray-700">草稿</span>
      default:
        return <span className="px-2 py-1 text-xs rounded-full bg-gray-100 text-gray-700">{status}</span>
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">工作流</h1>
          <p className="text-gray-500 mt-1">自动化营销流程</p>
        </div>
        <button
          onClick={() => setShowModal(true)}
          className="flex items-center gap-2 px-4 py-2 bg-brand-600 text-white rounded-lg hover:bg-brand-700"
        >
          <Plus className="h-4 w-4" />
          创建工作流
        </button>
      </div>

      <div className="bg-white rounded-xl border">
        {workflows.length === 0 ? (
          <div className="p-12 text-center">
            <GitBranch className="h-12 w-12 text-gray-300 mx-auto mb-4" />
            <p className="text-gray-500 mb-4">暂无工作流</p>
            <button
              onClick={() => setShowModal(true)}
              className="text-brand-600 hover:text-brand-700"
            >
              创建您的第一个工作流
            </button>
          </div>
        ) : (
          <table className="w-full">
            <thead className="bg-gray-50 border-b">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">名称</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">触发器</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">状态</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">步骤数</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">创建时间</th>
                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase">操作</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200">
              {workflows.map((workflow) => (
                <tr key={workflow.id} className="hover:bg-gray-50">
                  <td className="px-6 py-4">
                    <div className="flex items-center gap-3">
                      <div className="p-2 bg-brand-50 rounded-lg">
                        <GitBranch className="h-4 w-4 text-brand-600" />
                      </div>
                      <span className="font-medium text-gray-900">{workflow.name}</span>
                    </div>
                  </td>
                  <td className="px-6 py-4 text-gray-500">{workflow.trigger}</td>
                  <td className="px-6 py-4">{getStatusBadge(workflow.status)}</td>
                  <td className="px-6 py-4 text-gray-500">{workflow.steps_count}</td>
                  <td className="px-6 py-4 text-gray-500">
                    {new Date(workflow.created_at).toLocaleDateString('zh-CN')}
                  </td>
                  <td className="px-6 py-4 text-right">
                    <div className="flex items-center justify-end gap-2">
                      {workflow.status === 'active' ? (
                        <button className="inline-flex items-center gap-1 text-sm text-yellow-600 hover:text-yellow-800">
                          <Pause className="h-4 w-4" />
                        </button>
                      ) : workflow.status !== 'draft' ? (
                        <button className="inline-flex items-center gap-1 text-sm text-green-600 hover:text-green-800">
                          <Play className="h-4 w-4" />
                        </button>
                      ) : null}
                      <button className="inline-flex items-center gap-1 text-sm text-red-600 hover:text-red-800">
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
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl shadow-xl p-6 w-full max-w-lg">
            <h3 className="text-lg font-semibold mb-4">创建工作流</h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">工作流名称</label>
                <input
                  type="text"
                  className="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-brand-500 focus:border-brand-500 outline-none"
                  placeholder="例如：新用户欢迎流程"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">触发器类型</label>
                <select className="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-brand-500 focus:border-brand-500 outline-none">
                  <option value="">请选择</option>
                  <option value="contact_created">联系人创建时</option>
                  <option value="campaign_launched">活动启动时</option>
                  <option value="scheduled">定时触发</option>
                  <option value="event">自定义事件</option>
                </select>
              </div>
            </div>
            <p className="text-gray-500 text-sm mt-4">工作流可视化编辑器开发中...</p>
            <div className="mt-6 flex justify-end gap-3">
              <button
                onClick={() => setShowModal(false)}
                className="px-4 py-2 border rounded-lg"
              >
                取消
              </button>
              <button className="px-4 py-2 bg-brand-600 text-white rounded-lg">
                创建
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}