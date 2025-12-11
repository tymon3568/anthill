-- Create event_outbox table for transactional outbox pattern
-- This table stores events to be published to NATS reliably

CREATE TABLE event_outbox (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    tenant_id UUID NOT NULL,
    event_type TEXT NOT NULL,
    event_data JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'published', 'failed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at TIMESTAMPTZ,
    retry_count INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for efficient polling of pending events per tenant
CREATE INDEX idx_event_outbox_tenant_status_created ON event_outbox (tenant_id, status, created_at);

-- Index for cleanup of old published events
CREATE INDEX idx_event_outbox_published_at ON event_outbox (published_at) WHERE status = 'published';
