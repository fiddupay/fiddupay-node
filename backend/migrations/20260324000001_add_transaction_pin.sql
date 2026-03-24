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

