-- Migration: Add Account Lockout Tracking
-- Tracks failed login attempts and lockout status for merchants

CREATE TABLE IF NOT EXISTS merchant_login_attempts (
    email VARCHAR(255) PRIMARY KEY,
    failed_attempts INT NOT NULL DEFAULT 0,
    last_attempt_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    locked_until TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Index for cleaning up old records
CREATE INDEX IF NOT EXISTS idx_merchant_login_attempts_updated_at ON merchant_login_attempts(updated_at);

-- Trigger to update updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'trig_update_merchant_login_attempts_updated_at') THEN
        CREATE TRIGGER trig_update_merchant_login_attempts_updated_at
        BEFORE UPDATE ON merchant_login_attempts
        FOR EACH ROW
        EXECUTE FUNCTION update_updated_at_column();
    END IF;
END $$;
