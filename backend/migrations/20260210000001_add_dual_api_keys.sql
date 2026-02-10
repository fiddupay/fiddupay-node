-- Add new columns for separate API keys
ALTER TABLE merchants
ADD COLUMN live_api_key_hash TEXT,
ADD COLUMN test_api_key_hash TEXT;

-- Migrate existing data based on current mode
UPDATE merchants
SET
    test_api_key_hash = api_key_hash
WHERE
    sandbox_mode = true;

UPDATE merchants
SET
    live_api_key_hash = api_key_hash
WHERE
    sandbox_mode = false;

-- Drop the old column
ALTER TABLE merchants DROP COLUMN api_key_hash;