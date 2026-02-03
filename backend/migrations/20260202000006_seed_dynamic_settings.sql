-- Seed system_settings with default values for dynamic configuration

INSERT INTO system_settings (key, value, description, updated_at) VALUES
-- Fee Configuration
('DEFAULT_FEE_PERCENTAGE', '1.50', 'Default platform fee percentage per transaction', NOW()),

-- Limits
('DAILY_VOLUME_LIMIT_NON_KYC_USD', '1000.00', 'Daily transaction volume limit for non-KYC merchants in USD', NOW()),
('WITHDRAWAL_AUTO_APPROVAL_LIMIT_USD', '1000.00', 'Maximum withdrawal amount in USD that is auto-approved', NOW()),

-- Feature Flags
('MAINTENANCE_MODE', 'false', 'Enable maintenance mode to disable all API writes', NOW()),
('MERCHANT_REGISTRATION_ENABLED', 'true', 'Allow new merchants to register', NOW()),
('WITHDRAWAL_ENABLED', 'true', 'Global switch to enable or disable withdrawals', NOW()),
('INVOICE_ENABLED', 'true', 'Enable invoice generation feature', NOW()),

-- Security Policies
('MAX_LOGIN_ATTEMPTS', '5', 'Maximum failed login attempts before lockout', NOW()),
('ACCOUNT_LOCKOUT_DURATION_MINUTES', '30', 'Duration in minutes to lock account after max failed attempts', NOW()),
('TWO_FACTOR_ENABLED', 'true', 'Enforce 2FA for sensitive actions', NOW()),

-- Rate Limiting
('RATE_LIMIT_REQUESTS_PER_MINUTE', '100', 'API rate limit requests per minute per IP/Key', NOW()),
('RATE_LIMIT_BURST_SIZE', '20', 'API rate limit burst size', NOW())

ON CONFLICT (key) DO NOTHING;
