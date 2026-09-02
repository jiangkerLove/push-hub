-- Backfill columns for databases created before the current CREATE TABLE definitions.
-- Safe to run on fresh installs (ADD COLUMN IF NOT EXISTS / idempotent indexes).

ALTER TABLE push_job_targets ADD COLUMN IF NOT EXISTS outbox_id TEXT;
ALTER TABLE push_job_targets ADD COLUMN IF NOT EXISTS vendor_message_id TEXT;
ALTER TABLE push_outbox_trace ADD COLUMN IF NOT EXISTS target_id TEXT;
CREATE INDEX IF NOT EXISTS idx_push_job_targets_vendor_message_id
    ON push_job_targets (vendor_message_id)
    WHERE vendor_message_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_push_outbox_trace_target_id ON push_outbox_trace (target_id);

ALTER TABLE admin_users ADD COLUMN IF NOT EXISTS is_owner BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE admin_users ADD COLUMN IF NOT EXISTS display_time_zone TEXT;
ALTER TABLE admin_users ADD COLUMN IF NOT EXISTS password_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
UPDATE admin_users
SET is_owner = TRUE
WHERE id = (
    SELECT id FROM admin_users ORDER BY created_at ASC LIMIT 1
)
AND NOT EXISTS (SELECT 1 FROM admin_users WHERE is_owner = TRUE);

ALTER TABLE apps ADD COLUMN IF NOT EXISTS push_api_key TEXT;
UPDATE apps
SET push_api_key = 'phk_' || replace(gen_random_uuid()::text, '-', '')
WHERE push_api_key IS NULL OR push_api_key = '';

ALTER TABLE push_outbox ADD COLUMN IF NOT EXISTS template_vars_json JSONB;
ALTER TABLE push_outbox ADD COLUMN IF NOT EXISTS notify_id INTEGER;
