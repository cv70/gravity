import api from './api'
import type { ChannelAccount, ChannelAccountListResponse, ChannelCreatePayload, ChannelUpdatePayload } from '@/types'

export const channelService = {
  list: async () => {
    const { data } = await api.get('/channels')
    return data.data as ChannelAccountListResponse
  },

  create: async (payload: ChannelCreatePayload) => {
    const { data } = await api.post('/channels', payload)
    return data.data as ChannelAccount
  },

  update: async (id: string, payload: ChannelUpdatePayload) => {
    const { data } = await api.patch(`/channels/${id}`, payload)
    return data.data as ChannelAccount
  },

  remove: async (id: string) => {
    await api.delete(`/channels/${id}`)
  },
}
