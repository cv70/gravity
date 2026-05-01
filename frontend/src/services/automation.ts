import api from './api'
import type {
  ApprovalRequest,
  AutomationAction,
  AutomationBootstrapResult,
  AutomationDashboard,
  AutomationJob,
  AutomationRun,
  Experiment,
  PolicyRule,
} from '@/types'

export const automationService = {
  getDashboard: async () => {
    const { data } = await api.get('/automation/dashboard', { params: { limit: 5 } })
    return data.data as AutomationDashboard
  },

  listJobs: async () => {
    const { data } = await api.get('/automation/jobs')
    return data.data as { data: AutomationJob[]; total: number }
  },

  createJob: async (payload: {
    goal: string
    target_audience: Record<string, unknown>
    channel_preferences: string[]
    budget_limit?: number | null
    currency?: string
    desired_outcome?: string
    approval_required?: boolean
  }) => {
    const { data } = await api.post('/automation/jobs', payload)
    return data.data as AutomationJob
  },

  executeJob: async (id: string, note?: string) => {
    const { data } = await api.post(`/automation/jobs/${id}/execute`, { note })
    return data.data as {
      run: AutomationRun
      approval?: ApprovalRequest
      action?: AutomationAction
    }
  },

  listApprovals: async () => {
    const { data } = await api.get('/automation/approvals')
    return data.data as { data: ApprovalRequest[]; total: number }
  },

  reviewApproval: async (id: string, approved: boolean, note?: string) => {
    const { data } = await api.post(`/automation/approvals/${id}/decision`, { approved, note })
    return data.data as { approval: ApprovalRequest; action?: AutomationAction }
  },

  listPolicies: async () => {
    const { data } = await api.get('/automation/policies')
    return data.data as { data: PolicyRule[]; total: number }
  },

  listRuns: async () => {
    const { data } = await api.get('/automation/runs')
    return data.data as { data: AutomationRun[]; total: number }
  },

  listActions: async () => {
    const { data } = await api.get('/automation/actions')
    return data.data as { data: AutomationAction[]; total: number }
  },

  listExperiments: async () => {
    const { data } = await api.get('/automation/experiments')
    return data.data as { data: Experiment[]; total: number }
  },

  bootstrapDefaults: async () => {
    const { data } = await api.post('/automation/bootstrap')
    return data.data as AutomationBootstrapResult
  },
}
