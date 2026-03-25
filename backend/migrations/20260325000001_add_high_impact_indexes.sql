-- Add high-impact indexes for query performance
-- Created: 2026-03-25

-- 1. payment_transactions (to_address, status)
CREATE INDEX IF NOT EXISTS idx_payment_transactions_to_address_status
    ON payment_transactions (to_address, status);

-- 2. merchant_wallets (address, sandbox_mode, is_active)
CREATE INDEX IF NOT EXISTS idx_merchant_wallets_address_sandbox_active
    ON merchant_wallets (address, sandbox_mode, is_active);

-- 3. merchant_customer_wallets (address, sandbox_mode)
CREATE INDEX IF NOT EXISTS idx_merchant_customer_wallets_address_sandbox
    ON merchant_customer_wallets (address, sandbox_mode);
