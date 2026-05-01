import api from './api'
import type { Campaign, CampaignCreatePayload, CampaignUpdatePayload } from '@/types'

export const campaignsService = {
  list: async () => {
    const { data } = await api.get('/campaigns')
    return data.data as { data: Campaign[]; total: number }
  },

  get: async (id: string) => {
    const { data } = await api.get(`/campaigns/${id}`)
    return data.data as Campaign
  },

  create: async (campaign: CampaignCreatePayload) => {
    const { data } = await api.post('/campaigns', campaign)
    return data.data as Campaign
  },

  update: async (id: string, campaign: CampaignUpdatePayload) => {
    const { data } = await api.patch(`/campaigns/${id}`, campaign)
    return data.data as Campaign
  },

  launch: async (id: string) => {
    const { data } = await api.post(`/campaigns/${id}/launch`)
    return data.data as Campaign
  },

  pause: async (id: string) => {
    const { data } = await api.post(`/campaigns/${id}/pause`)
    return data.data as Campaign
  },

  delete: async (id: string) => {
    await api.delete(`/campaigns/${id}`)
  },
}
