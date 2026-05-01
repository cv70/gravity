import api from './api'
import type { Contact, ContactCreatePayload, ContactUpdatePayload } from '@/types'

export const contactsService = {
  list: async (params?: { page?: number; limit?: number; search?: string }) => {
    const { data } = await api.get('/contacts', { params })
    return data.data as { data: Contact[]; total: number; page: number; limit: number }
  },

  get: async (id: string) => {
    const { data } = await api.get(`/contacts/${id}`)
    return data.data as Contact
  },

  create: async (contact: ContactCreatePayload) => {
    const { data } = await api.post('/contacts', contact)
    return data.data as Contact
  },

  update: async (id: string, contact: ContactUpdatePayload) => {
    const { data } = await api.patch(`/contacts/${id}`, contact)
    return data.data as Contact
  },

  delete: async (id: string) => {
    await api.delete(`/contacts/${id}`)
  },
}
