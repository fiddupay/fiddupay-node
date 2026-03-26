-- Merchant Portal Performance Optimization
-- Additional indexes for refunds and withdrawals to support the unified transaction feed
-- Created: 2026-03-26

-- 1. Refunds index for unified transaction feed (merchant portal)
CREATE INDEX IF NOT EXISTS idx_refunds_merchant_sandbox_created 
    ON refunds (merchant_id, sandbox_mode, created_at DESC);

-- 2. Withdrawals index for unified transaction feed and dashboard
CREATE INDEX IF NOT EXISTS idx_withdrawals_merchant_sandbox_status_created 
    ON withdrawals (merchant_id, sandbox_mode, status, created_at DESC);
