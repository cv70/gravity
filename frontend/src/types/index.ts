import api from './api'

export interface ApiResponse<T> {
  code: number
  message: string
  data: T
}

export interface Contact {
  id: string
  tenant_id: string
  email: string
  name: string
  phone?: string
  tags: string[]
  attributes: Record<string, unknown>
  subscribed: boolean
  created_at: string
  updated_at: string
}

export interface Campaign {
  id: string
  tenant_id: string
  name: string
  campaign_type: 'social' | 'email' | 'content' | 'ads'
  status: 'draft' | 'active' | 'paused' | 'completed'
  description?: string
  start_date?: string
  end_date?: string
  metrics?: {
    sent: number
    opened: number
    clicked: number
    converted: number
  }
  created_at: string
  updated_at: string
}

export interface Content {
  id: string
  tenant_id: string
  campaign_id?: string
  name: string
  content_type: string
  content: unknown
  status: string
  created_at: string
  updated_at: string
}

export interface AnalyticsDashboard {
  total_contacts: number
  active_campaigns: number
  total_conversions: number
  conversion_rate: number
}
