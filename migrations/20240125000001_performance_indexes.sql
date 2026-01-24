-- Performance Optimization Indexes
-- Add critical indexes for frequently queried columns

-- Merchants table indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_merchants_email ON merchants(email);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_merchants_active ON merchants(is_active) WHERE is_active = true;
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_merchants_created_at ON merchants(created_at);

-- Payments table indexes  
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_payments_merchant_id ON payments(merchant_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_payments_status ON payments(status);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_payments_created_at ON payments(created_at);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_payments_expires_at ON payments(expires_at);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_payments_merchant_status ON payments(merchant_id, status);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_payments_status_created ON payments(status, created_at);

-- Payment links table indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_payment_links_payment_id ON payment_links(payment_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_payment_links_link_id ON payment_links(link_id);

-- Merchant wallets table indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_merchant_wallets_merchant_id ON merchant_wallets(merchant_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_merchant_wallets_crypto_type ON merchant_wallets(crypto_type);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_merchant_wallets_merchant_crypto ON merchant_wallets(merchant_id, crypto_type);

-- Webhooks table indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_webhooks_merchant_id ON webhooks(merchant_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_webhooks_payment_id ON webhooks(payment_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_webhooks_status ON webhooks(status);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_webhooks_created_at ON webhooks(created_at);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_webhooks_next_retry ON webhooks(next_retry_at) WHERE status = 'PENDING';

-- Security tables indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_login_attempts_email ON login_attempts(email);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_login_attempts_ip ON login_attempts(ip_address);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_login_attempts_attempted_at ON login_attempts(attempted_at);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_login_attempts_email_time ON login_attempts(email, attempted_at);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_blocked_ips_ip ON blocked_ips(ip_address);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_blocked_ips_expires ON blocked_ips(expires_at);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_blocked_ips_active ON blocked_ips(ip_address, expires_at) WHERE expires_at > NOW();

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_security_events_timestamp ON security_events(timestamp);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_security_events_type ON security_events(event_type);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_security_events_severity ON security_events(severity);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_security_events_source_ip ON security_events(source_ip);

-- Composite indexes for common query patterns
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_payments_merchant_created_desc ON payments(merchant_id, created_at DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_payments_status_expires ON payments(status, expires_at) WHERE status IN ('PENDING', 'CONFIRMED');
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_webhooks_status_retry ON webhooks(status, next_retry_at) WHERE status = 'PENDING';

-- Partial indexes for better performance
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_payments_pending ON payments(created_at) WHERE status = 'PENDING';
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_payments_expired ON payments(expires_at) WHERE status = 'PENDING' AND expires_at < NOW();
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_merchants_active_created ON merchants(created_at) WHERE is_active = true;
