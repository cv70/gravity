import { useState } from 'react'
import { Link, useLocation } from 'react-router-dom'
import { Archive, Building2, FileText, Layers3, LayoutDashboard, LogOut, Menu, MessageCircle, Settings, Sparkles, UserCircle2, Users, Megaphone, BarChart3, ShieldCheck, X } from 'lucide-react'

import { cn } from '@/lib/utils'
import { useAuthStore } from '@/stores/auth'

const navigation = [
  {
    label: '经营',
    items: [
      { name: '仪表盘', href: '/', icon: LayoutDashboard },
      { name: '联系人', href: '/contacts', icon: Users },
      { name: '营销活动', href: '/campaigns', icon: Megaphone },
      { name: '数据分析', href: '/analytics', icon: BarChart3 },
      { name: '内容管理', href: '/content', icon: FileText },
      { name: '渠道管理', href: '/channels', icon: MessageCircle },
    ],
  },
  {
    label: '自动化',
    items: [
      { name: '自动化中枢', href: '/workflows', icon: Sparkles },
      { name: '治理总览', href: '/governance', icon: ShieldCheck },
      { name: '分群中心', href: '/segments', icon: Layers3 },
      { name: '审批中心', href: '/approvals', icon: ShieldCheck },
      { name: '审计中心', href: '/audit', icon: Archive },
    ],
  },
  {
    label: '系统',
    items: [{ name: '设置', href: '/settings', icon: Settings }],
  },
]

export function Layout({ children }: { children: React.ReactNode }) {
  const [sidebarOpen, setSidebarOpen] = useState(false)
  const location = useLocation()
  const logout = useAuthStore((s) => s.logout)
  const user = useAuthStore((s) => s.user)
  const organizationName = useAuthStore((s) => s.organizationName)

  return (
    <div className="min-h-screen bg-[radial-gradient(circle_at_top_left,rgba(14,165,233,0.12),transparent_24%),linear-gradient(180deg,#f8fbff_0%,#f4f7fb_100%)] text-slate-900">
      <div className={cn('fixed inset-0 z-50 lg:hidden', sidebarOpen ? 'block' : 'hidden')}>
        <div className="fixed inset-0 bg-slate-950/80" onClick={() => setSidebarOpen(false)} />
        <div className="fixed inset-y-0 left-0 w-72 bg-slate-950 text-white shadow-2xl">
          <div className="flex h-16 items-center justify-between px-6 border-b border-white/10">
            <span className="text-xl font-bold tracking-tight text-white">Gravity</span>
            <button onClick={() => setSidebarOpen(false)}><X className="h-5 w-5" /></button>
          </div>
          <nav className="px-3 py-4 space-y-4">
            {navigation.map((group) => (
              <div key={group.label} className="space-y-2">
                <p className="px-3 text-[11px] font-semibold uppercase tracking-[0.22em] text-slate-500">
                  {group.label}
                </p>
                <div className="space-y-1">
                  {group.items.map((item) => {
                    const isActive = location.pathname === item.href
                    return (
                      <Link
                        key={item.name}
                        to={item.href}
                        onClick={() => setSidebarOpen(false)}
                        className={cn(
                          'flex items-center gap-3 px-3 py-2 rounded-xl text-sm font-medium transition-colors',
                          isActive ? 'bg-white/10 text-white' : 'text-slate-300 hover:bg-white/5 hover:text-white'
                        )}
                      >
                        <item.icon className="h-5 w-5" />
                        {item.name}
                      </Link>
                    )
                  })}
                </div>
              </div>
            ))}
          </nav>
          <div className="absolute bottom-4 left-3 right-3 space-y-3">
            <div className="rounded-2xl border border-white/10 bg-white/5 p-3 text-xs text-slate-300">
              <div className="flex items-center gap-2 text-white">
                <Building2 className="h-4 w-4" />
                {organizationName || 'Gravity Workspace'}
              </div>
              <div className="mt-2 flex items-center gap-2">
                <UserCircle2 className="h-4 w-4" />
                {user?.name || user?.email || '未登录'}
              </div>
            </div>
            <button
              onClick={logout}
              className="flex w-full items-center gap-3 px-3 py-2 text-sm font-medium text-slate-300 rounded-xl hover:bg-white/5 hover:text-white"
            >
              <LogOut className="h-5 w-5" />
              退出登录
            </button>
          </div>
        </div>
      </div>

      <div className="hidden lg:fixed lg:inset-y-0 lg:flex lg:w-72 lg:flex-col">
        <div className="flex h-16 items-center px-6 border-b border-white/10 bg-slate-950">
          <span className="text-xl font-bold tracking-tight text-white">Gravity</span>
        </div>
        <nav className="flex-1 px-3 py-4 space-y-4 bg-slate-950 border-r border-white/10">
          {navigation.map((group) => (
            <div key={group.label} className="space-y-2">
              <p className="px-3 text-[11px] font-semibold uppercase tracking-[0.22em] text-slate-500">
                {group.label}
              </p>
              <div className="space-y-1">
                {group.items.map((item) => {
                  const isActive = location.pathname === item.href
                  return (
                    <Link
                      key={item.name}
                      to={item.href}
                      className={cn(
                        'flex items-center gap-3 px-3 py-2 rounded-xl text-sm font-medium transition-colors',
                        isActive ? 'bg-white/10 text-white' : 'text-slate-300 hover:bg-white/5 hover:text-white'
                      )}
                    >
                      <item.icon className="h-5 w-5" />
                      {item.name}
                    </Link>
                  )
                })}
              </div>
            </div>
          ))}
        </nav>
        <div className="p-4 border-t border-white/10 bg-slate-950">
          <div className="mb-3 rounded-2xl border border-white/10 bg-white/5 p-3 text-xs text-slate-300">
            <div className="flex items-center gap-2 text-white">
              <Building2 className="h-4 w-4" />
              {organizationName || 'Gravity Workspace'}
            </div>
            <div className="mt-2 flex items-center gap-2">
              <UserCircle2 className="h-4 w-4" />
              {user?.name || user?.email || '未登录'}
            </div>
          </div>
          <button
            onClick={logout}
            className="flex w-full items-center gap-3 px-3 py-2 text-sm font-medium text-slate-300 rounded-xl hover:bg-white/5 hover:text-white"
          >
            <LogOut className="h-5 w-5" />
            退出登录
          </button>
        </div>
      </div>

      <div className="lg:pl-72">
        <div className="sticky top-0 z-40 flex h-16 items-center justify-between gap-4 px-4 backdrop-blur-xl bg-white/75 border-b border-slate-200/70 lg:px-8">
          <button
            className="lg:hidden p-2 rounded-lg hover:bg-gray-100"
            onClick={() => setSidebarOpen(true)}
          >
            <Menu className="h-5 w-5" />
          </button>
          <div className="ml-auto hidden sm:flex items-center gap-3 text-sm text-slate-600">
            <span className="rounded-full border border-slate-200 bg-white px-3 py-1.5 shadow-sm">
              {organizationName || 'Gravity Workspace'}
            </span>
            <span className="rounded-full border border-slate-200 bg-white px-3 py-1.5 shadow-sm">
              {user?.email || 'session active'}
            </span>
          </div>
        </div>
        <main className="p-4 lg:p-8">{children}</main>
      </div>
    </div>
  )
}
