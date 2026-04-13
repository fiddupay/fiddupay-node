-- Discovery Engine Optimization v3
-- Targets the slow "SELECT DISTINCT to_address" queries in BackgroundTasks
-- Created: 2026-04-13

-- 1. Partial Index for Pending Payments
-- This targets the payment_transactions table which represents the largest potential scan.
-- By indexing only PENDING/CONFIRMING rows, we reduce the search space from millions to dozens.
CREATE INDEX IF NOT EXISTS idx_payment_discovery_partial_v3
    ON payment_transactions (sandbox_mode, network, crypto_type, to_address)
    WHERE status IN ('PENDING', 'CONFIRMING') AND to_address IS NOT NULL;

-- 2. Covering Index for Customer Wallets
-- Including 'address' in the index allows for an Index-Only Scan.
DROP INDEX IF EXISTS idx_merchant_customer_wallets_discovery;
CREATE INDEX idx_merchant_customer_wallets_discovery_v3
    ON merchant_customer_wallets (sandbox_mode, crypto_type, address);

-- 3. Covering Index for Merchant Wallets
-- Including 'address' and 'is_active' in the index for optimal lookup.
DROP INDEX IF EXISTS idx_merchant_wallets_discovery;
CREATE INDEX idx_merchant_wallets_discovery_v3
    ON merchant_wallets (sandbox_mode, is_active, crypto_type, address);

-- 4. Analyze to update planner statistics
ANALYZE payment_transactions;
ANALYZE merchant_customer_wallets;
ANALYZE merchant_wallets;
