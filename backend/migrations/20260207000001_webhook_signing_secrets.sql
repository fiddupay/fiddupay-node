-- Add signing_secret column to webhook_configs
ALTER TABLE webhook_configs
ADD COLUMN IF NOT EXISTS signing_secret TEXT;

-- Generate random secrets for existing configurations
-- We'll use a simple random string for initial population
UPDATE webhook_configs
SET
    signing_secret = encode (gen_random_bytes (32), 'hex')
WHERE
    signing_secret IS NULL;

-- Make it NOT NULL for future entries
ALTER TABLE webhook_configs ALTER COLUMN signing_secret SET NOT NULL;