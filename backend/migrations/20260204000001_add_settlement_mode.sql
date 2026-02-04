-- Migration: Add settlement_mode to merchants table
-- settlement_mode can be 'forwarding', 'managed', or 'imported'

ALTER TABLE merchants
ADD COLUMN settlement_mode VARCHAR(20) NOT NULL DEFAULT 'managed';

-- Add check constraint to ensure only valid modes are used
ALTER TABLE merchants
ADD CONSTRAINT check_settlement_mode CHECK (
    settlement_mode IN (
        'forwarding',
        'managed',
        'imported'
    )
);

COMMENT ON COLUMN merchants.settlement_mode IS 'Settlement strategy: forwarding (auto-bridge), managed (generated), imported (private key)';