-- Update default fee percentage to 1.50%
UPDATE system_settings SET value = '0.75' WHERE key = 'DEFAULT_FEE_PERCENTAGE';

-- Also update existing merchants who were using the old default (optional, but consistent)
-- UPDATE merchants SET fee_percentage = 1.50 WHERE fee_percentage = 0.75;
