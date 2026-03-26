-- Performance Booster Migration
-- Optimized indexes for high-volume transactions and analytic queries
-- Created: 2026-03-26

-- 1. Index for customer transaction history (dashboards and customer detail modal)
CREATE INDEX IF NOT EXISTS idx_customer_transactions_history
    ON customer_transactions (customer_id, merchant_id, type, created_at DESC);

-- 2. Index for audit logs (merchant activity feed)
CREATE INDEX IF NOT EXISTS idx_audit_logs_merchant_activity
    ON audit_logs (merchant_id, created_at DESC);

-- 3. Composite index for system-wide verification scans and status tracking
CREATE INDEX IF NOT EXISTS idx_payment_transactions_verification_flow
    ON payment_transactions (status, created_at DESC);

-- 4. Index for filtering customer transactions by status and sandbox mode
CREATE INDEX IF NOT EXISTS idx_customer_transactions_status_sandbox
    ON customer_transactions (status, sandbox_mode);

-- 5. Index for analytics aggregation on customer transactions
CREATE INDEX IF NOT EXISTS idx_customer_transactions_merchant_created
    ON customer_transactions (merchant_id, created_at DESC);
-- 6. Refunds index for unified transaction feed (merchant portal)
CREATE INDEX IF NOT EXISTS idx_refunds_merchant_sandbox_created 
    ON refunds (merchant_id, sandbox_mode, created_at DESC);

-- 7. Withdrawals index for unified transaction feed and dashboard
CREATE INDEX IF NOT EXISTS idx_withdrawals_merchant_sandbox_status_created 
    ON withdrawals (merchant_id, sandbox_mode, status, created_at DESC);
