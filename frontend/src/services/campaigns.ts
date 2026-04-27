import api from './api'
import type { Campaign } from '@/types'

export const campaignsService = {
  list: async () => {
    const { data } = await api.get('/campaigns')
    return data.data as { data: Campaign[]; total: number }
  },

  get: async (id: string) => {
    const { data } = await api.get(`/campaigns/${id}`)
    return data.data as Campaign
  },

  create: async (campaign: Omit<Campaign, 'id' | 'tenant_id' | 'created_at' | 'updated_at'>) => {
    const { data } = await api.post('/campaigns', campaign)
    return data.data as Campaign
  },

  update: async (id: string, campaign: Partial<Campaign>) => {
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
