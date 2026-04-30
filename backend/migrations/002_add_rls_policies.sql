-- Migration: 002_add_rls_policies
-- Fix RLS policies that were missing, fix GIN index, add tenant_id to workflow_executions

-- Fix GIN composite index (PostgreSQL requires btree_gin for non-array types in composite index)
-- Drop the invalid composite GIN index
DROP INDEX IF EXISTS idx_contacts_tenant_tags;

-- Create separate indexes instead
CREATE INDEX idx_contacts_tenant ON contacts(tenant_id);
CREATE INDEX idx_contacts_tags ON contacts USING GIN(tags);

-- Add tenant_id to workflow_executions for RLS support
ALTER TABLE workflow_executions ADD COLUMN IF NOT EXISTS tenant_id UUID;
UPDATE workflow_executions we SET tenant_id = w.tenant_id FROM workflows w WHERE we.workflow_id = w.id;
ALTER TABLE workflow_executions ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE workflow_executions ADD CONSTRAINT fk_workflow_executions_tenant FOREIGN KEY (tenant_id) REFERENCES organizations(id) ON DELETE CASCADE;

-- RLS for users table
CREATE POLICY tenant_isolation_users ON users
    FOR ALL
    USING (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- RLS for contacts table
CREATE POLICY tenant_isolation_contacts ON contacts
    FOR ALL
    USING (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- RLS for campaigns table
CREATE POLICY tenant_isolation_campaigns ON campaigns
    FOR ALL
    USING (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- RLS for contents table
CREATE POLICY tenant_isolation_contents ON contents
    FOR ALL
    USING (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- RLS for events table
CREATE POLICY tenant_isolation_events ON events
    FOR ALL
    USING (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- RLS for conversions table
CREATE POLICY tenant_isolation_conversions ON conversions
    FOR ALL
    USING (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- RLS for channel_accounts table
CREATE POLICY tenant_isolation_channel_accounts ON channel_accounts
    FOR ALL
    USING (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- RLS for workflows table
CREATE POLICY tenant_isolation_workflows ON workflows
    FOR ALL
    USING (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- RLS for workflow_executions table
CREATE POLICY tenant_isolation_workflow_executions ON workflow_executions
    FOR ALL
    USING (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- Note: organizations table should NOT have RLS because organizations ARE the tenants.
-- The root organization record should always be accessible to its own users.