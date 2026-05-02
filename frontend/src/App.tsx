import { useEffect } from 'react'
import { Routes, Route, Navigate } from 'react-router-dom'
import { useAuthStore } from '@/stores/auth'
import { Layout } from '@/components/ui/layout'
import { LoginPage } from '@/app/login/page'
import { DashboardPage } from '@/app/dashboard/page'
import { ContactsPage } from '@/app/contacts/page'
import { CampaignsPage } from '@/app/campaigns/page'
import { AnalyticsPage } from '@/app/analytics/page'
import { ContentsPage } from '@/app/content/page'
import { ChannelsPage } from '@/app/channels/page'
import { WorkflowsPage } from '@/app/workflows/page'
import { GovernancePage } from '@/app/governance/page'
import { SegmentsPage } from '@/app/segments/page'
import { ApprovalsPage } from '@/app/approvals/page'
import { AuditPage } from '@/app/audit/page'
import { SettingsPage } from '@/app/settings/page'

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated)
  if (!isAuthenticated) return <Navigate to="/login" replace />
  return <Layout>{children}</Layout>
}

export default function App() {
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated)
  const hydrateFromProfile = useAuthStore((s) => s.hydrateFromProfile)

  useEffect(() => {
    if (isAuthenticated) {
      void hydrateFromProfile()
    }
  }, [hydrateFromProfile, isAuthenticated])

  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />
      <Route path="/" element={<ProtectedRoute><DashboardPage /></ProtectedRoute>} />
      <Route path="/contacts" element={<ProtectedRoute><ContactsPage /></ProtectedRoute>} />
      <Route path="/campaigns" element={<ProtectedRoute><CampaignsPage /></ProtectedRoute>} />
      <Route path="/analytics" element={<ProtectedRoute><AnalyticsPage /></ProtectedRoute>} />
      <Route path="/content" element={<ProtectedRoute><ContentsPage /></ProtectedRoute>} />
      <Route path="/channels" element={<ProtectedRoute><ChannelsPage /></ProtectedRoute>} />
      <Route path="/workflows" element={<ProtectedRoute><WorkflowsPage /></ProtectedRoute>} />
      <Route path="/governance" element={<ProtectedRoute><GovernancePage /></ProtectedRoute>} />
      <Route path="/segments" element={<ProtectedRoute><SegmentsPage /></ProtectedRoute>} />
      <Route path="/approvals" element={<ProtectedRoute><ApprovalsPage /></ProtectedRoute>} />
      <Route path="/audit" element={<ProtectedRoute><AuditPage /></ProtectedRoute>} />
      <Route path="/settings" element={<ProtectedRoute><SettingsPage /></ProtectedRoute>} />
    </Routes>
  )
}
