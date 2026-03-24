-- Add Transaction PIN support to Merchants and Customers
-- This migration adds secure storage for 4-digit numeric PINs used for financial actions.

-- 1. Add fields to merchants table
ALTER TABLE merchants 
ADD COLUMN IF NOT EXISTS transaction_pin_hash TEXT,
ADD COLUMN IF NOT EXISTS pin_setup_at TIMESTAMPTZ;

-- 2. Add fields to merchant_customers table
ALTER TABLE merchant_customers
ADD COLUMN IF NOT EXISTS transaction_pin_hash TEXT,
ADD COLUMN IF NOT EXISTS pin_setup_at TIMESTAMPTZ;

-- 3. Add audit logs for PIN events if not already supported
-- (Assuming audit_events table exists)
INSERT INTO audit_events (event_type, description) 
VALUES 
('transaction_pin_set', 'Merchant or customer has set/updated their transaction PIN'),
('transaction_pin_verify_failed', 'Failed PIN verification attempt during financial action')
ON CONFLICT DO NOTHING;
