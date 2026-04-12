-- Global Performance Booster
-- Optimized Composite and Expression Indexes for Dashboard Activity and Lookups
-- Created: 2026-04-13

-- 1. Optimized Unified Activity Indexing
-- These indexes target the UNION ALL query in list_unified_transactions
CREATE INDEX IF NOT EXISTS idx_payment_transactions_unified_feed
    ON payment_transactions (merchant_id, sandbox_mode, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_customer_transactions_unified_feed
    ON customer_transactions (merchant_id, sandbox_mode, created_at DESC);

-- Note: Refunds and Withdrawals already have similar indexes, but we ensure coverage
CREATE INDEX IF NOT EXISTS idx_refunds_unified_feed
    ON refunds (merchant_id, sandbox_mode, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_withdrawals_unified_feed
    ON withdrawals (merchant_id, sandbox_mode, created_at DESC);


-- 2. Expression Indexes for Wallet Lookups
-- Standard B-Tree indexes don't work with LOWER(). These make address matching O(1)
CREATE INDEX IF NOT EXISTS idx_payment_transactions_to_address_lwr
    ON payment_transactions (LOWER(to_address));

CREATE INDEX IF NOT EXISTS idx_merchant_customer_wallets_addr_lwr
    ON merchant_customer_wallets (LOWER(address));

CREATE INDEX IF NOT EXISTS idx_merchant_wallets_addr_lwr
    ON merchant_wallets (LOWER(address));


-- 3. Analytics Aggregation Indexes
-- These target the SUM() and COUNT() queries in AnalyticsService
CREATE INDEX IF NOT EXISTS idx_payment_transactions_analytics_status
    ON payment_transactions (merchant_id, status, created_at DESC);

-- 4. Cross-Table Join Optimization
-- Accelerates joins between refunds and payments in the unified feed
CREATE INDEX IF NOT EXISTS idx_refunds_payment_lookup
    ON refunds (payment_id);
