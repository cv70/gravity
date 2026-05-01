import api from './api'
import type { Content, CreateContentPayload } from '@/types'

export const contentsService = {
  list: async (params?: { page?: number; limit?: number }) => {
    const { data } = await api.get('/contents', { params })
    return data.data as { data: Content[]; total: number }
  },

  create: async (payload: CreateContentPayload) => {
    const { data } = await api.post('/contents', payload)
    return data.data as Content
  },

  remove: async (id: string) => {
    await api.delete(`/contents/${id}`)
  },
}
