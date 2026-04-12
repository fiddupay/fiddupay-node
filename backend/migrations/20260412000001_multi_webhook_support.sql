-- Remove unique constraint to allow multiple webhooks per merchant
-- This allows a merchant to have both a Discord webhook and a standard API webhook simultaneously.

-- 1. Drop the unique constraint
-- Note: The name is typically webhook_configs_merchant_id_key based on standard Postgres naming
ALTER TABLE webhook_configs DROP CONSTRAINT IF EXISTS webhook_configs_merchant_id_key;

-- 2. Ensure we have an index for fast lookup since it's no longer unique
CREATE INDEX IF NOT EXISTS idx_webhook_configs_merchant_id ON webhook_configs(merchant_id);

-- 3. Add a description or name field to help identify webhooks (optional but good)
ALTER TABLE webhook_configs ADD COLUMN IF NOT EXISTS name VARCHAR(100);

-- 4. Comment update
COMMENT ON TABLE webhook_configs IS 'Webhook notification URLs for merchants (supports multiple endpoints)';
