-- Migration: Add MANAGED_MODE_ONLY to system_settings
-- This ensures the setting exists across all server migrations

INSERT INTO system_settings (key, value, description)
VALUES (
    'MANAGED_MODE_ONLY', 
    'true', 
    'If set to true, merchants are restricted to managed settlement mode only.'
)
ON CONFLICT (key) DO NOTHING;
