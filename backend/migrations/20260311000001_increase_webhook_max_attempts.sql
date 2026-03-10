-- Drop the old constraint that limits to 5 attempts
-- This fixes the issue where retry logic (up to 12 attempts) was blocked by the DB
ALTER TABLE webhook_deliveries DROP CONSTRAINT IF EXISTS chk_webhook_attempts_valid;

-- Add updated constraint allowing up to 12 attempts to match the backend retry code
ALTER TABLE webhook_deliveries
    ADD CONSTRAINT chk_webhook_attempts_valid
    CHECK (attempts >= 0 AND attempts <= 12);
