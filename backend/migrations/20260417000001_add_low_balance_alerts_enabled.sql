-- Add low_balance_alerts_enabled column to merchants table
ALTER TABLE merchants ADD COLUMN low_balance_alerts_enabled BOOLEAN NOT NULL DEFAULT FALSE;
