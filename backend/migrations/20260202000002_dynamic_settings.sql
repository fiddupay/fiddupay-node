-- System Settings Table
-- Stores dynamic configuration controllable by admins at runtime

CREATE TABLE IF NOT EXISTS system_settings (
    key VARCHAR(255) PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by BIGINT REFERENCES merchants(id) -- Admin who changed it
);

-- Index for quick lookups
CREATE INDEX idx_system_settings_key ON system_settings(key);

-- Insert default values (fallback to env vars if not present)
INSERT INTO system_settings (key, value, description) VALUES
    ('MAINTENANCE_MODE', 'false', 'Master switch to enable maintenance mode'),
    ('PAYMENTS_DISABLED', 'false', 'Disable new payment creation'),
    ('WITHDRAWALS_DISABLED', 'false', 'Disable all withdrawals'),
    ('REGISTRATION_DISABLED', 'false', 'Disable new merchant registrations'),
    ('DEFAULT_FEE_PERCENTAGE', '0.75', 'Default fee percentage for new merchants'),
    ('DAILY_VOLUME_LIMIT_NON_KYC_USD', '1000.00', 'Daily volume limit for non-KYC merchants in USD'),
    ('WITHDRAWAL_AUTO_APPROVAL_LIMIT_USD', '1000.00', 'Withdrawal auto-approval threshold'),
    ('MERCHANT_AUTO_APPROVAL', 'true', 'Automatically approve new merchant accounts')
ON CONFLICT (key) DO NOTHING;
