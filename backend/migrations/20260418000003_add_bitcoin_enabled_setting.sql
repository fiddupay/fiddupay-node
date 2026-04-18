-- Add Bitcoin maintenance toggle to system settings
INSERT INTO system_settings (key, value, description)
VALUES ('BITCOIN_ENABLED', 'true', 'Global toggle to enable/disable Bitcoin monitoring and transactions')
ON CONFLICT (key) DO NOTHING;
