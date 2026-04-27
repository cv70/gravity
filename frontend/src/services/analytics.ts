import api from './api'
import type { AnalyticsDashboard } from '@/types'

export const analyticsService = {
  getDashboard: async () => {
    const { data } = await api.get('/analytics/dashboard')
    return data.data as AnalyticsDashboard
  },

  getFunnel: async (campaignId?: string) => {
    const { data } = await api.get('/analytics/funnel', { params: { campaign_id: campaignId } })
    return data.data as { steps: Array<{ step: string; count: number; dropoff_rate: number }> }
  },
}
