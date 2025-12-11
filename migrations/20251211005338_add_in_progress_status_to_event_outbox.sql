-- Add 'in_progress' status to event_outbox table for atomic claim pattern
-- This prevents double processing of events by multiple workers

ALTER TABLE event_outbox
DROP CONSTRAINT event_outbox_status_check;

ALTER TABLE event_outbox
ADD CONSTRAINT event_outbox_status_check
CHECK (status IN ('pending', 'in_progress', 'published', 'failed'));
