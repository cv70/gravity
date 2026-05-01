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

export interface ChannelAccount {
  id: string
  tenant_id: string
  platform: 'email' | 'wechat' | 'xiaohongshu' | 'douyin' | 'ads' | string
  name: string
  credentials_encrypted: string
  settings: Record<string, unknown>
  status: 'connected' | 'disconnected' | 'error' | string
  last_sync_at?: string | null
  created_at: string
  updated_at: string
}

export interface AnalyticsDashboard {
  total_contacts: number
  active_campaigns: number
  total_conversions: number
  conversion_rate: number
}

export interface AutomationJob {
  id: string
  tenant_id: string
  goal: string
  target_audience: Record<string, unknown>
  channel_preferences: string[]
  strategy: Record<string, unknown>
  status: 'draft' | 'waiting_approval' | 'active' | 'paused' | 'completed' | 'failed'
  risk_level: 'low' | 'medium' | 'high' | string
  approval_required: boolean
  budget_limit?: number | null
  currency: string
  next_action_at?: string | null
  created_at: string
  updated_at: string
}

export interface AutomationRun {
  id: string
  tenant_id: string
  job_id: string
  status: 'queued' | 'running' | 'waiting_approval' | 'completed' | 'failed' | string
  current_step: string
  input_context: Record<string, unknown>
  output_context: Record<string, unknown>
  started_at: string
  completed_at?: string | null
  last_error?: string | null
}

export interface AutomationAction {
  id: string
  tenant_id: string
  run_id: string
  action_type: string
  channel: string
  payload: Record<string, unknown>
  risk_level: string
  status: string
  requires_approval: boolean
  approved_by?: string | null
  approved_at?: string | null
  executed_at?: string | null
  failure_reason?: string | null
  created_at: string
  updated_at: string
}

export interface ApprovalRequest {
  id: string
  tenant_id: string
  action_id: string
  title: string
  reason: string
  status: 'pending' | 'approved' | 'rejected' | string
  requested_by?: string | null
  reviewed_by?: string | null
  reviewed_at?: string | null
  decision_note?: string | null
  created_at: string
  updated_at: string
}

export interface PolicyRule {
  id: string
  tenant_id: string
  name: string
  rule_type: string
  scope: Record<string, unknown>
  settings: Record<string, unknown>
  enabled: boolean
  created_at: string
  updated_at: string
}

export interface Experiment {
  id: string
  tenant_id: string
  job_id?: string | null
  name: string
  hypothesis: string
  variant_a: Record<string, unknown>
  variant_b: Record<string, unknown>
  status: string
  winner?: string | null
  metric?: string | null
  created_at: string
  updated_at: string
}

export interface AutomationDashboard {
  overview: {
    total_jobs: number
    active_jobs: number
    runs_in_progress: number
    pending_approvals: number
    blocked_actions: number
    enabled_policies: number
    experiments_running: number
    automation_coverage: number
    human_intervention_rate: number
  }
  jobs: AutomationJob[]
  runs: AutomationRun[]
  actions: AutomationAction[]
  approvals: ApprovalRequest[]
  policies: PolicyRule[]
  experiments: Experiment[]
  recommendations: string[]
}

export interface AutomationBootstrapResult {
  jobs_created: AutomationJob[]
  policies_created: PolicyRule[]
  experiments_created: Experiment[]
  messages: string[]
}

export interface ChannelAccountListResponse {
  data: ChannelAccount[]
  total: number
}
