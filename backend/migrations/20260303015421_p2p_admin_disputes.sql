-- Migration: P2P Admin Disputes
-- This table handles trades that have been contested by either the buyer or seller.

CREATE TABLE IF NOT EXISTS p2p_disputes (
    id BIGSERIAL PRIMARY KEY,
    trade_id BIGINT NOT NULL REFERENCES p2p_trades(id) ON DELETE CASCADE,
    initiator_id BIGINT NOT NULL REFERENCES p2p_profiles(id) ON DELETE CASCADE,
    respondent_id BIGINT NOT NULL REFERENCES p2p_profiles(id) ON DELETE CASCADE,
    admin_id BIGINT, -- ID of the admin staff handling the ticket
    reason VARCHAR(100) NOT NULL, -- e.g., 'BUYER_PAID_BUT_UNRELEASED', 'BUYER_MARKED_PAID_WITHOUT_PAYING'
    description TEXT,
    status VARCHAR(30) NOT NULL DEFAULT 'OPEN', -- 'OPEN', 'UNDER_INVESTIGATION', 'RESOLVED_FOR_MAKER', 'RESOLVED_FOR_TAKER', 'CANCELLED'
    sandbox_mode BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_p2p_disputes_trade ON p2p_disputes(trade_id);
CREATE INDEX IF NOT EXISTS idx_p2p_disputes_status ON p2p_disputes(status);
CREATE INDEX IF NOT EXISTS idx_p2p_disputes_sandbox ON p2p_disputes(sandbox_mode);

COMMENT ON TABLE p2p_disputes IS 'Admin resolution tickets for contested P2P trades';
