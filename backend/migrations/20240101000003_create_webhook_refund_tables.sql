-- Create webhook_deliveries table
-- Tracks webhook delivery attempts with retry logic
CREATE TABLE webhook_deliveries (
    id BIGSERIAL PRIMARY KEY,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    payment_id BIGINT NOT NULL REFERENCES payment_transactions(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL,  -- "payment.confirmed", "payment.expired", "payment.partial", "refund.created", "refund.completed"
    url VARCHAR(500) NOT NULL,
    payload JSONB NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',  -- "pending", "delivered", "failed"
    attempts INT NOT NULL DEFAULT 0,
    last_attempt_at TIMESTAMPTZ,
    next_retry_at TIMESTAMPTZ,
    response_status INT,
    response_body TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for webhook_deliveries table
CREATE INDEX idx_webhook_deliveries_merchant ON webhook_deliveries(merchant_id);
CREATE INDEX idx_webhook_deliveries_payment ON webhook_deliveries(payment_id);
CREATE INDEX idx_webhook_deliveries_status ON webhook_deliveries(status);
CREATE INDEX idx_webhook_deliveries_event_type ON webhook_deliveries(event_type);
CREATE INDEX idx_webhook_deliveries_next_retry ON webhook_deliveries(next_retry_at) 
    WHERE status = 'pending';
CREATE INDEX idx_webhook_deliveries_created_at ON webhook_deliveries(created_at);

-- Create refunds table
-- Tracks refund requests and their status
CREATE TABLE refunds (
    id BIGSERIAL PRIMARY KEY,
    refund_id VARCHAR(100) UNIQUE NOT NULL,  -- Public-facing ID (e.g., "ref_abc123")
    merchant_id BIGINT NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    payment_id BIGINT NOT NULL REFERENCES payment_transactions(id) ON DELETE CASCADE,
    amount DECIMAL(20,8) NOT NULL,
    amount_usd DECIMAL(20,2) NOT NULL,
    reason TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',  -- "pending", "completed", "failed"
    transaction_hash VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

-- Indexes for refunds table
CREATE INDEX idx_refunds_merchant ON refunds(merchant_id);
CREATE INDEX idx_refunds_payment ON refunds(payment_id);
CREATE INDEX idx_refunds_refund_id ON refunds(refund_id);
CREATE INDEX idx_refunds_status ON refunds(status);
CREATE INDEX idx_refunds_created_at ON refunds(created_at);
CREATE INDEX idx_refunds_transaction_hash ON refunds(transaction_hash) 
    WHERE transaction_hash IS NOT NULL;

-- Create audit_logs table
-- Tracks all system actions for compliance and security
CREATE TABLE audit_logs (
    id BIGSERIAL PRIMARY KEY,
    merchant_id BIGINT REFERENCES merchants(id) ON DELETE SET NULL,
    action_type VARCHAR(100) NOT NULL,  -- "merchant.created", "merchant.updated", "api_key.generated", "api_key.rotated", "payment.created", "payment.confirmed", "payment.expired", "refund.created", "refund.completed", "webhook.configured", "wallet.added", "wallet.updated", "ip_whitelist.updated"
    entity_type VARCHAR(50),  -- "merchant", "payment", "refund", "webhook", "wallet", "api_key", "ip_whitelist"
    entity_id VARCHAR(100),
    ip_address VARCHAR(45),  -- Supports IPv6
    user_agent TEXT,
    details JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for audit_logs table
CREATE INDEX idx_audit_logs_merchant ON audit_logs(merchant_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);
CREATE INDEX idx_audit_logs_action_type ON audit_logs(action_type);
CREATE INDEX idx_audit_logs_entity_type ON audit_logs(entity_type);
CREATE INDEX idx_audit_logs_entity_id ON audit_logs(entity_id);

-- Create ip_whitelist table
-- Stores IP restrictions per merchant for enhanced security
CREATE TABLE ip_whitelist (
    id BIGSERIAL PRIMARY KEY,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    ip_address VARCHAR(45) NOT NULL,  -- Supports IPv6
    cidr_range VARCHAR(50),  -- Optional CIDR notation (e.g., "192.168.1.0/24")
    description TEXT,  -- Optional description for this IP entry
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for ip_whitelist table
CREATE INDEX idx_ip_whitelist_merchant ON ip_whitelist(merchant_id);
CREATE INDEX idx_ip_whitelist_is_active ON ip_whitelist(is_active);
CREATE INDEX idx_ip_whitelist_merchant_active ON ip_whitelist(merchant_id, is_active);

-- Add comments for documentation
COMMENT ON TABLE webhook_deliveries IS 'Webhook delivery attempts with retry tracking and response logging';
COMMENT ON TABLE refunds IS 'Refund requests for completed payments (full or partial refunds)';
COMMENT ON TABLE audit_logs IS 'Audit trail of all system actions for compliance and security';
COMMENT ON TABLE ip_whitelist IS 'IP address restrictions per merchant for enhanced API security';

COMMENT ON COLUMN webhook_deliveries.event_type IS 'Type of event that triggered the webhook';
COMMENT ON COLUMN webhook_deliveries.payload IS 'JSON payload sent to the webhook endpoint';
COMMENT ON COLUMN webhook_deliveries.attempts IS 'Number of delivery attempts made';
COMMENT ON COLUMN webhook_deliveries.next_retry_at IS 'Timestamp for next retry attempt (with exponential backoff)';
COMMENT ON COLUMN webhook_deliveries.response_status IS 'HTTP status code from webhook endpoint';
COMMENT ON COLUMN webhook_deliveries.response_body IS 'Response body from webhook endpoint (truncated)';

COMMENT ON COLUMN refunds.refund_id IS 'Public-facing refund identifier (e.g., ref_abc123)';
COMMENT ON COLUMN refunds.amount IS 'Refund amount in cryptocurrency';
COMMENT ON COLUMN refunds.amount_usd IS 'Refund amount in USD';
COMMENT ON COLUMN refunds.reason IS 'Merchant-provided reason for the refund';
COMMENT ON COLUMN refunds.transaction_hash IS 'Blockchain transaction hash for the refund transaction';

COMMENT ON COLUMN audit_logs.action_type IS 'Type of action performed (e.g., merchant.created, payment.confirmed)';
COMMENT ON COLUMN audit_logs.entity_type IS 'Type of entity affected by the action';
COMMENT ON COLUMN audit_logs.entity_id IS 'Identifier of the entity affected';
COMMENT ON COLUMN audit_logs.ip_address IS 'IP address of the request that triggered the action';
COMMENT ON COLUMN audit_logs.details IS 'Additional context and details stored as JSON';

COMMENT ON COLUMN ip_whitelist.ip_address IS 'IP address or start of CIDR range';
COMMENT ON COLUMN ip_whitelist.cidr_range IS 'CIDR notation for IP range (e.g., 192.168.1.0/24)';
COMMENT ON COLUMN ip_whitelist.description IS 'Optional description for this IP entry (e.g., "Office network")';

-- Add constraints
-- Ensure webhook attempts don't exceed maximum retries
ALTER TABLE webhook_deliveries 
    ADD CONSTRAINT chk_webhook_attempts_valid 
    CHECK (attempts >= 0 AND attempts <= 5);

-- Ensure refund amount is positive
ALTER TABLE refunds 
    ADD CONSTRAINT chk_refund_amount_positive 
    CHECK (amount > 0 AND amount_usd > 0);

-- Ensure transaction_hash is unique when not null for refunds
CREATE UNIQUE INDEX idx_refunds_transaction_hash_unique 
    ON refunds(transaction_hash) 
    WHERE transaction_hash IS NOT NULL;

-- Ensure merchant doesn't exceed 10 IP whitelist entries
-- Note: This is enforced at application level, but we add a comment for documentation
COMMENT ON TABLE ip_whitelist IS 'IP address restrictions per merchant (max 10 entries per merchant enforced at application level)';

-- Add trigger to automatically update webhook next_retry_at with exponential backoff
-- This will be handled at application level, but we document the retry schedule:
-- Attempt 1: immediate
-- Attempt 2: 1 second
-- Attempt 3: 2 seconds
-- Attempt 4: 4 seconds
-- Attempt 5: 8 seconds
-- Attempt 6: 16 seconds (final attempt)

-- Add trigger to update merchants.updated_at on changes
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply trigger to merchants table (if not already exists)
DROP TRIGGER IF EXISTS update_merchants_updated_at ON merchants;
CREATE TRIGGER update_merchants_updated_at 
    BEFORE UPDATE ON merchants 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- Apply trigger to merchant_wallets table (if not already exists)
DROP TRIGGER IF EXISTS update_merchant_wallets_updated_at ON merchant_wallets;
CREATE TRIGGER update_merchant_wallets_updated_at 
    BEFORE UPDATE ON merchant_wallets 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- Apply trigger to webhook_configs table (if not already exists)
DROP TRIGGER IF EXISTS update_webhook_configs_updated_at ON webhook_configs;
CREATE TRIGGER update_webhook_configs_updated_at 
    BEFORE UPDATE ON webhook_configs 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();
