import { useState } from 'react'
import { Settings, Bell, Shield, Database, Palette } from 'lucide-react'

type Tab = 'general' | 'notifications' | 'security' | 'data' | 'appearance'

export function SettingsPage() {
  const [activeTab, setActiveTab] = useState<Tab>('general')

  const tabs = [
    { id: 'general', label: '通用', icon: Settings },
    { id: 'notifications', label: '通知', icon: Bell },
    { id: 'security', label: '安全', icon: Shield },
    { id: 'data', label: '数据', icon: Database },
    { id: 'appearance', label: '外观', icon: Palette },
  ] as const

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-gray-900">设置</h1>
        <p className="text-gray-500 mt-1">管理您的账户和组织设置</p>
      </div>

      <div className="flex gap-6">
        <div className="w-48 shrink-0">
          <nav className="space-y-1">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`w-full flex items-center gap-3 px-3 py-2 text-sm font-medium rounded-lg transition-colors ${
                  activeTab === tab.id
                    ? 'bg-brand-50 text-brand-700'
                    : 'text-gray-600 hover:bg-gray-100'
                }`}
              >
                <tab.icon className="h-4 w-4" />
                {tab.label}
              </button>
            ))}
          </nav>
        </div>

        <div className="flex-1 bg-white rounded-xl border">
          <div className="p-6 border-b">
            <h2 className="text-lg font-semibold text-gray-900">
              {tabs.find(t => t.id === activeTab)?.label}
            </h2>
          </div>
          <div className="p-6">
            {activeTab === 'general' && (
              <div className="space-y-6">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">组织名称</label>
                  <input
                    type="text"
                    defaultValue="我的公司"
                    className="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-brand-500 focus:border-brand-500 outline-none"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">时区</label>
                  <select className="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-brand-500 focus:border-brand-500 outline-none">
                    <option value="Asia/Shanghai">中国标准时间 (UTC+8)</option>
                    <option value="America/New_York">美东时间 (UTC-5)</option>
                    <option value="Europe/London">伦敦时间 (UTC+0)</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">语言</label>
                  <select className="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-brand-500 focus:border-brand-500 outline-none">
                    <option value="zh-CN">简体中文</option>
                    <option value="en">English</option>
                  </select>
                </div>
                <button className="px-4 py-2 bg-brand-600 text-white rounded-lg hover:bg-brand-700">
                  保存更改
                </button>
              </div>
            )}

            {activeTab === 'notifications' && (
              <div className="space-y-6">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium text-gray-900">邮件通知</p>
                    <p className="text-sm text-gray-500">接收活动状态变更通知</p>
                  </div>
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input type="checkbox" defaultChecked className="sr-only peer" />
                    <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-brand-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-brand-600"></div>
                  </label>
                </div>
                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium text-gray-900">营销报告</p>
                    <p className="text-sm text-gray-500">每周发送营销数据摘要</p>
                  </div>
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input type="checkbox" defaultChecked className="sr-only peer" />
                    <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-brand-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-brand-600"></div>
                  </label>
                </div>
              </div>
            )}

            {activeTab === 'security' && (
              <div className="space-y-6">
                <div>
                  <h3 className="font-medium text-gray-900 mb-4">修改密码</h3>
                  <div className="space-y-4">
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">当前密码</label>
                      <input type="password" className="w-full px-3 py-2 border rounded-lg" />
                    </div>
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">新密码</label>
                      <input type="password" className="w-full px-3 py-2 border rounded-lg" />
                    </div>
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">确认新密码</label>
                      <input type="password" className="w-full px-3 py-2 border rounded-lg" />
                    </div>
                    <button className="px-4 py-2 bg-brand-600 text-white rounded-lg hover:bg-brand-700">
                      更新密码
                    </button>
                  </div>
                </div>
                <div className="pt-6 border-t">
                  <h3 className="font-medium text-gray-900 mb-4">两步验证</h3>
                  <p className="text-sm text-gray-500 mb-4">启用两步验证以增强账户安全</p>
                  <button className="px-4 py-2 border border-brand-600 text-brand-600 rounded-lg hover:bg-brand-50">
                    启用两步验证
                  </button>
                </div>
              </div>
            )}

            {activeTab === 'data' && (
              <div className="space-y-6">
                <div className="flex items-center justify-between p-4 bg-gray-50 rounded-lg">
                  <div>
                    <p className="font-medium text-gray-900">数据导出</p>
                    <p className="text-sm text-gray-500">导出所有联系人数据</p>
                  </div>
                  <button className="px-4 py-2 border rounded-lg hover:bg-white">
                    导出 CSV
                  </button>
                </div>
                <div className="flex items-center justify-between p-4 bg-red-50 rounded-lg">
                  <div>
                    <p className="font-medium text-red-700">危险区域</p>
                    <p className="text-sm text-red-500">永久删除您的账户和数据</p>
                  </div>
                  <button className="px-4 py-2 border border-red-600 text-red-600 rounded-lg hover:bg-red-50">
                    删除账户
                  </button>
                </div>
              </div>
            )}

            {activeTab === 'appearance' && (
              <div className="space-y-6">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">主题</label>
                  <div className="grid grid-cols-3 gap-4">
                    <button className="p-4 border-2 border-brand-500 rounded-lg bg-white">
                      <div className="w-full h-8 bg-brand-600 rounded mb-2"></div>
                      <span className="text-sm font-medium">品牌色</span>
                    </button>
                    <button className="p-4 border rounded-lg bg-white hover:border-gray-300">
                      <div className="w-full h-8 bg-gray-900 rounded mb-2"></div>
                      <span className="text-sm font-medium">深色</span>
                    </button>
                    <button className="p-4 border rounded-lg bg-white hover:border-gray-300">
                      <div className="w-full h-8 bg-white border rounded mb-2"></div>
                      <span className="text-sm font-medium">浅色</span>
                    </button>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}