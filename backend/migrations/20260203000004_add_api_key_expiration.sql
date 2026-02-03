-- Add api_key_expires_at column to merchants table
ALTER TABLE merchants ADD COLUMN api_key_expires_at TIMESTAMPTZ;

-- For existing users, let's set a default expiration of 30 days from now 
-- to avoid logging them out immediately after migration
UPDATE merchants SET api_key_expires_at = NOW() + INTERVAL '30 days' WHERE api_key_expires_at IS NULL;

-- Add a comment for documentation
COMMENT ON COLUMN merchants.api_key_expires_at IS 'Timestamp when the current API key expires. If NULL, it expires based on session (or never, depending on policy).';
