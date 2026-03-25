-- Additional performance indexes for analytics, dashboard, and join-heavy queries
-- Created: 2026-03-25

-- 1. payment_transactions (merchant_id, created_at DESC, status)
CREATE INDEX IF NOT EXISTS idx_payment_transactions_merchant_created_status
    ON payment_transactions (merchant_id, created_at DESC, status);

-- 2. merchant_customers (merchant_id, external_id)
CREATE INDEX IF NOT EXISTS idx_merchant_customers_merchant_external
    ON merchant_customers (merchant_id, external_id);

-- 3. address_only_payments (merchant_id, status, created_at DESC)
CREATE INDEX IF NOT EXISTS idx_address_only_payments_merchant_status_created
    ON address_only_payments (merchant_id, status, created_at DESC);

-- 4. webhook_deliveries (merchant_id, status, created_at DESC)
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_merchant_status_created
    ON webhook_deliveries (merchant_id, status, created_at DESC);

-- 5. merchant_customer_wallets (merchant_id, customer_id)
CREATE INDEX IF NOT EXISTS idx_merchant_customer_wallets_merchant_customer
    ON merchant_customer_wallets (merchant_id, customer_id);

-- 6. deposit_addresses (deposit_address)
CREATE INDEX IF NOT EXISTS idx_deposit_addresses_deposit_address
    ON deposit_addresses (deposit_address);
