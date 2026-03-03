-- Migration: Phase 5 - P2P Support Tickets

CREATE TABLE IF NOT EXISTS p2p_support_tickets (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES p2p_profiles(id) ON DELETE CASCADE,
    subject VARCHAR(255) NOT NULL,
    category VARCHAR(50) NOT NULL, -- 'SCAM_REPORT', 'BUG', 'PAYMENT_ISSUE', 'OTHER'
    description TEXT NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'OPEN', -- 'OPEN', 'IN_PROGRESS', 'RESOLVED', 'CLOSED'
    reported_user_id BIGINT REFERENCES p2p_profiles(id) ON DELETE SET NULL, -- Optional: if reporting another user
    trade_id VARCHAR REFERENCES p2p_trades(trade_id) ON DELETE SET NULL, -- Optional: if related to a trade
    attachment_url VARCHAR(1024),
    admin_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_p2p_support_tickets_user ON p2p_support_tickets(user_id);
CREATE INDEX IF NOT EXISTS idx_p2p_support_tickets_status ON p2p_support_tickets(status);
CREATE INDEX IF NOT EXISTS idx_p2p_support_tickets_reported_user ON p2p_support_tickets(reported_user_id);
