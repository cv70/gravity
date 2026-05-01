import api from './api'
import type { ChannelAccount, ChannelAccountListResponse } from '@/types'

export const channelService = {
  list: async () => {
    const { data } = await api.get('/channels')
    return data.data as ChannelAccountListResponse
  },

  create: async (payload: {
    platform: string
    name: string
    credentials_encrypted: string
    settings: Record<string, unknown>
    status?: string
  }) => {
    const { data } = await api.post('/channels', payload)
    return data.data as ChannelAccount
  },

  update: async (
    id: string,
    payload: {
      name?: string
      status?: string
      settings?: Record<string, unknown>
      last_sync_at?: string
    },
  ) => {
    const { data } = await api.patch(`/channels/${id}`, payload)
    return data.data as ChannelAccount
  },

  remove: async (id: string) => {
    await api.delete(`/channels/${id}`)
  },
}
