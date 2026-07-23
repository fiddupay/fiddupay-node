-- Add auto_settlement_enabled column to merchants table (default true for all merchants)
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS auto_settlement_enabled BOOLEAN NOT NULL DEFAULT TRUE;
