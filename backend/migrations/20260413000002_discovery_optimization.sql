-- Discovery Optimization Migration
-- Specifically targets slow SELECT DISTINCT to_address queries on startup
-- Created: 2026-04-13

-- Targets: SELECT DISTINCT to_address FROM payment_transactions WHERE status IN ('PENDING', 'CONFIRMING') AND sandbox_mode = $1
CREATE INDEX IF NOT EXISTS idx_payment_transactions_discovery_v2
    ON payment_transactions (sandbox_mode, status, network, crypto_type);

-- Targets: SELECT address FROM merchant_customer_wallets WHERE sandbox_mode = $1 AND crypto_type = ANY($3)
CREATE INDEX IF NOT EXISTS idx_merchant_customer_wallets_discovery
    ON merchant_customer_wallets (sandbox_mode, crypto_type);

-- Targets: SELECT address FROM merchant_wallets WHERE sandbox_mode = $1 AND is_active = true AND crypto_type = ANY($3)
CREATE INDEX IF NOT EXISTS idx_merchant_wallets_discovery
    ON merchant_wallets (sandbox_mode, crypto_type, is_active);
