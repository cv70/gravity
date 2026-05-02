import api from './api'
import type {
  Approval,
  ApprovalListResponse,
  AuditLogListResponse,
  InsightListResponse,
  Segment,
  SegmentListResponse,
  SegmentCreatePayload,
  SegmentUpdatePayload,
  SegmentPreviewPayload,
  ApprovalQuery,
  AuditLogQuery,
} from '@/types'

export const governanceService = {
  segments: async (params?: { page?: number; limit?: number }) => {
    const { data } = await api.get('/segments', { params })
    return data.data as SegmentListResponse
  },

  createSegment: async (payload: SegmentCreatePayload) => {
    const { data } = await api.post('/segments', payload)
    return data.data as Segment
  },

  updateSegment: async (id: string, payload: SegmentUpdatePayload) => {
    const { data } = await api.patch(`/segments/${id}`, payload)
    return data.data as Segment
  },

  deleteSegment: async (id: string) => {
    await api.delete(`/segments/${id}`)
  },

  previewSegment: async (id: string, payload?: SegmentPreviewPayload) => {
    const { data } = await api.post(`/segments/${id}/preview`, payload ?? {})
    return data.data as { matching_count: number; sample_contacts: Array<{ id: string; email: string; name: string; lifecycle_stage: string; tags: string[] }> }
  },

  approvals: async (params?: ApprovalQuery) => {
    const { data } = await api.get('/approvals', { params })
    return data.data as ApprovalListResponse
  },

  reviewApproval: async (id: string, approved: boolean, note?: string) => {
    const { data } = await api.patch(`/approvals/${id}`, { approved, note })
    return data.data as Approval
  },

  auditLogs: async (params?: AuditLogQuery) => {
    const { data } = await api.get('/audit-logs', { params })
    return data.data as AuditLogListResponse
  },

  recommendations: async () => {
    const { data } = await api.get('/insights/recommendations')
    return data.data as InsightListResponse
  },

  anomalies: async () => {
    const { data } = await api.get('/insights/anomalies')
    return data.data as InsightListResponse
  },

  opportunities: async () => {
    const { data } = await api.get('/insights/opportunities')
    return data.data as InsightListResponse
  },
}
