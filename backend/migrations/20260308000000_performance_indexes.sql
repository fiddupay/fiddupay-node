-- Add performance indexes for merchant dashboard
-- These indexes optimize analytics aggregation and paginated listing

-- Index for main listing and analytics (filtering by merchant, sandbox mode, and date range)
CREATE INDEX IF NOT EXISTS idx_payments_merchant_sandbox_created 
ON payment_transactions(merchant_id, sandbox_mode, created_at DESC);

-- Index for status-based filtering
CREATE INDEX IF NOT EXISTS idx_payments_merchant_status 
ON payment_transactions(merchant_id, status);

-- Index for blockchain/network filtering
CREATE INDEX IF NOT EXISTS idx_payments_merchant_network 
ON payment_transactions(merchant_id, network);
