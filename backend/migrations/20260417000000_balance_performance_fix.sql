-- Balance Performance Fix
-- Adds a targeted composite index for transaction volume aggregation
-- Created: 2026-04-17

-- Targets the heavy LATERAL join in get_wallet_balances and dashboard stats
-- Covering: merchant_id, crypto_type, sandbox_mode, status + amount for SUM
CREATE INDEX IF NOT EXISTS idx_payment_transactions_dashboard_volume
    ON payment_transactions (merchant_id, crypto_type, sandbox_mode, status)
    INCLUDE (amount);

-- Index for currency-wide activity summary
CREATE INDEX IF NOT EXISTS idx_payment_transactions_crypto_summary
    ON payment_transactions (merchant_id, crypto_type, status);
