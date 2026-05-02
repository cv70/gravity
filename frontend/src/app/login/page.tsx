import { useMemo, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { ArrowRight, BadgeCheck, BrainCircuit, Globe2, ShieldCheck } from 'lucide-react'

import { useAuthStore } from '@/stores/auth'
import { Button } from '@/components/ui/button'

type Mode = 'login' | 'register'

export function LoginPage() {
  const [mode, setMode] = useState<Mode>('login')
  const [name, setName] = useState('')
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [organizationName, setOrganizationName] = useState('')
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)

  const navigate = useNavigate()
  const login = useAuthStore((s) => s.login)
  const register = useAuthStore((s) => s.register)

  const highlights = useMemo(
    () => [
      { icon: BrainCircuit, title: '智能驾驶舱', desc: '策略、审批、执行统一在一个界面里完成' },
      { icon: ShieldCheck, title: '企业级治理', desc: '多租户、审批、审计和风控链路默认开启' },
      { icon: Globe2, title: '全渠道接入', desc: '邮件、企微、内容平台、表单、广告统一编排' },
    ],
    [],
  )

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')
    setLoading(true)

    try {
      if (mode === 'login') {
        await login(email, password, organizationName)
      } else {
        await register(name, email, password, organizationName)
      }
      navigate('/')
    } catch {
      setError(mode === 'login' ? '邮箱、组织名称或密码错误' : '注册失败，请检查输入')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="min-h-screen bg-[radial-gradient(circle_at_top_left,rgba(14,165,233,0.18),transparent_30%),radial-gradient(circle_at_bottom_right,rgba(15,23,42,0.12),transparent_30%),linear-gradient(180deg,#081120_0%,#0f172a_100%)]">
      <div className="mx-auto grid min-h-screen max-w-7xl lg:grid-cols-[1.15fr_0.85fr]">
        <div className="relative hidden overflow-hidden p-12 text-white lg:flex lg:flex-col lg:justify-between">
          <div className="relative z-10 max-w-xl space-y-8">
            <div className="inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-4 py-2 text-sm text-cyan-100 backdrop-blur">
              <BadgeCheck className="h-4 w-4" />
              企业级自动化营销运营平台
            </div>
            <div>
              <h1 className="text-5xl font-black tracking-tight">
                Gravity
                <span className="block bg-gradient-to-r from-cyan-300 via-sky-200 to-white bg-clip-text text-transparent">
                  让运营工作自动跑起来
                </span>
              </h1>
              <p className="mt-6 max-w-2xl text-lg leading-8 text-slate-300">
                从内容到留资、从线索到成交、从复购到留存，统一数据、决策、编排与执行，构建可商业化交付的增长操作系统。
              </p>
            </div>
            <div className="grid gap-4 sm:grid-cols-3">
              {highlights.map((item) => (
                <div key={item.title} className="rounded-3xl border border-white/10 bg-white/5 p-5 backdrop-blur">
                  <item.icon className="h-5 w-5 text-cyan-300" />
                  <h3 className="mt-4 font-semibold text-white">{item.title}</h3>
                  <p className="mt-2 text-sm leading-6 text-slate-300">{item.desc}</p>
                </div>
              ))}
            </div>
          </div>
          <div className="relative z-10 rounded-3xl border border-white/10 bg-white/5 p-6 backdrop-blur">
            <p className="text-sm uppercase tracking-[0.24em] text-slate-400">Enterprise readiness</p>
            <div className="mt-4 grid grid-cols-3 gap-4">
              <div>
                <div className="text-2xl font-bold text-white">24/7</div>
                <div className="text-sm text-slate-300">自动化运行</div>
              </div>
              <div>
                <div className="text-2xl font-bold text-white">RBAC</div>
                <div className="text-sm text-slate-300">权限隔离</div>
              </div>
              <div>
                <div className="text-2xl font-bold text-white">A/B</div>
                <div className="text-sm text-slate-300">实验优化</div>
              </div>
            </div>
          </div>
          <div className="absolute inset-0 bg-[radial-gradient(circle_at_top_right,rgba(34,211,238,0.18),transparent_20%),radial-gradient(circle_at_bottom_left,rgba(59,130,246,0.2),transparent_22%)]" />
        </div>

        <div className="relative flex items-center justify-center px-4 py-10 lg:px-10">
          <div className="absolute inset-0 bg-[radial-gradient(circle_at_top_right,rgba(34,211,238,0.12),transparent_25%),radial-gradient(circle_at_bottom_left,rgba(59,130,246,0.15),transparent_25%)] pointer-events-none" />
          <div className="w-full max-w-md rounded-3xl border border-slate-200/80 bg-white/95 p-8 shadow-2xl backdrop-blur-xl">
            <div className="text-center">
              <div className="inline-flex items-center justify-center rounded-2xl bg-brand-50 px-4 py-2 text-brand-700">
                <BrainCircuit className="h-5 w-5" />
              </div>
              <h2 className="mt-5 text-3xl font-bold tracking-tight text-slate-900">
                {mode === 'login' ? '欢迎回来' : '创建你的工作空间'}
              </h2>
              <p className="mt-2 text-sm text-slate-500">
                {mode === 'login'
                  ? '登录后进入你的自动化运营驾驶舱。'
                  : '注册后自动初始化组织与默认自动化模板。'}
              </p>
            </div>

            <div className="mt-6 grid grid-cols-2 rounded-2xl bg-slate-100 p-1 text-sm font-medium">
              <Button
                type="button"
                onClick={() => setMode('login')}
                variant={mode === 'login' ? 'secondary' : 'ghost'}
                className="rounded-xl px-3 py-2"
              >
                登录
              </Button>
              <Button
                type="button"
                onClick={() => setMode('register')}
                variant={mode === 'register' ? 'secondary' : 'ghost'}
                className="rounded-xl px-3 py-2"
              >
                注册
              </Button>
            </div>

            <form onSubmit={handleSubmit} className="mt-6 space-y-4">
              {error && <div className="rounded-xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">{error}</div>}

              {mode === 'register' && (
                <div>
                  <label className="mb-1 block text-sm font-medium text-slate-700">姓名</label>
                  <input
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    className="w-full rounded-xl border border-slate-200 bg-white px-3 py-2.5 outline-none transition focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                    placeholder="张三"
                    required
                  />
                </div>
              )}

              <div>
                <label className="mb-1 block text-sm font-medium text-slate-700">组织名称</label>
                <input
                  value={organizationName}
                  onChange={(e) => setOrganizationName(e.target.value)}
                  className="w-full rounded-xl border border-slate-200 bg-white px-3 py-2.5 outline-none transition focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                  placeholder="我的公司"
                  required
                />
              </div>

              <div>
                <label className="mb-1 block text-sm font-medium text-slate-700">邮箱</label>
                <input
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  className="w-full rounded-xl border border-slate-200 bg-white px-3 py-2.5 outline-none transition focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                  placeholder="admin@example.com"
                  required
                />
              </div>

              <div>
                <label className="mb-1 block text-sm font-medium text-slate-700">密码</label>
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="w-full rounded-xl border border-slate-200 bg-white px-3 py-2.5 outline-none transition focus:border-brand-500 focus:ring-4 focus:ring-brand-100"
                  placeholder="••••••••"
                  required
                />
              </div>

              <Button
                type="submit"
                disabled={loading}
                variant="brand"
                className="w-full py-3"
              >
                {loading ? '处理中...' : mode === 'login' ? '登录进入系统' : '注册并初始化'}
                {!loading && <ArrowRight className="h-4 w-4" />}
              </Button>
            </form>

            <div className="mt-6 rounded-2xl border border-slate-200 bg-slate-50 p-4 text-sm text-slate-600">
              <p className="font-medium text-slate-900">首次使用建议</p>
              <p className="mt-1 leading-6">
                先注册一个组织，系统会自动生成默认自动化模板、风控策略和实验骨架，适合直接开始商业化试运行。
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
