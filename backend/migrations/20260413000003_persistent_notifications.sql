-- Persistent Notifications Table
-- Stores history of merchant events (deposits, payments, security alerts)

CREATE TABLE IF NOT EXISTS merchant_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id BIGINT NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    notification_type VARCHAR(50) NOT NULL DEFAULT 'info', -- 'success', 'error', 'info', 'warning'
    event_type VARCHAR(100) NOT NULL, -- e.g. 'merchant.deposit', 'system.alert'
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    sandbox_mode BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP WITH TIME ZONE -- Optional pruning logic
);

-- Optimize for unread lookups and merchant-specific history
CREATE INDEX IF NOT EXISTS idx_notifications_merchant_unread ON merchant_notifications(merchant_id, is_read, sandbox_mode);
CREATE INDEX IF NOT EXISTS idx_notifications_merchant_created ON merchant_notifications(merchant_id, created_at DESC);
