-- Migration: Phase 5 - P2P Security, Fees & Reputation

-- 1. Reputation System (Ratings)
CREATE TABLE IF NOT EXISTS p2p_ratings (
    id BIGSERIAL PRIMARY KEY,
    trade_id BIGINT NOT NULL REFERENCES p2p_trades(id) ON DELETE CASCADE,
    reviewer_id BIGINT NOT NULL REFERENCES p2p_profiles(id) ON DELETE CASCADE,
    target_id BIGINT NOT NULL REFERENCES p2p_profiles(id) ON DELETE CASCADE,
    rating VARCHAR(15) NOT NULL, -- 'THUMBS_UP', 'THUMBS_DOWN'
    comment TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(trade_id, reviewer_id) -- A user can only review a specific trade once
);

CREATE INDEX IF NOT EXISTS idx_p2p_ratings_target ON p2p_ratings(target_id);

-- Profile Aggregates (Add columns to track positive/negative reputation)
ALTER TABLE p2p_profiles 
ADD COLUMN thumbs_up_count INT NOT NULL DEFAULT 0,
ADD COLUMN thumbs_down_count INT NOT NULL DEFAULT 0;

-- 2. Fraud Warning Broadcasts (Chat System)
ALTER TABLE p2p_chat_messages
ADD COLUMN is_warning_broadcast BOOLEAN NOT NULL DEFAULT false;

-- 3. Fee Engine (0.15% Escrow Lock fee)
-- Update the escrow lock procedure to deduct a 0.15% fee from the seller's locked balance
-- and route it to the global platform fee wallet.

CREATE OR REPLACE FUNCTION p2p_lock_funds_in_escrow_with_fee(
    p_seller_id BIGINT,
    p_crypto_type VARCHAR,
    p_amount DECIMAL,
    p_trade_id VARCHAR,
    p_sandbox_mode BOOLEAN
) RETURNS VOID AS $$
DECLARE
    v_current_available DECIMAL;
    v_fee_percentage DECIMAL := 0.0015; -- 0.15%
    v_fee_amount DECIMAL;
    v_amount_after_fee DECIMAL;
BEGIN
    SELECT available_balance INTO v_current_available
    FROM p2p_balances
    WHERE user_id = p_seller_id AND crypto_type = p_crypto_type AND sandbox_mode = p_sandbox_mode
    FOR UPDATE;
    
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Seller balance record not found.';
    END IF;

    IF v_current_available < p_amount THEN
        RAISE EXCEPTION 'Insufficient available balance. Required: %, Available: %', p_amount, v_current_available;
    END IF;

    -- Calculate Fee
    v_fee_amount := p_amount * v_fee_percentage;
    v_amount_after_fee := p_amount - v_fee_amount;

    -- 1. Deduct whole amount from available
    UPDATE p2p_balances
    SET 
        available_balance = available_balance - p_amount,
        locked_balance = locked_balance + v_amount_after_fee,
        last_updated = NOW()
    WHERE user_id = p_seller_id AND crypto_type = p_crypto_type AND sandbox_mode = p_sandbox_mode;

    -- History: Debit Available (Full Amount)
    INSERT INTO p2p_balance_history (
        user_id, crypto_type, amount, balance_type, change_type, reason, reference_id, balance_before, balance_after, sandbox_mode
    ) VALUES (
        p_seller_id, p_crypto_type, p_amount, 'AVAILABLE', 'DEBIT', 'ESCROW_LOCK', p_trade_id, v_current_available, v_current_available - p_amount, p_sandbox_mode
    );

    -- History: Credit Locked (Amount - Fee)
    INSERT INTO p2p_balance_history (
        user_id, crypto_type, amount, balance_type, change_type, reason, reference_id, balance_before, balance_after, sandbox_mode
    ) VALUES (
        p_seller_id, p_crypto_type, v_amount_after_fee, 'LOCKED', 'CREDIT', 'ESCROW_LOCK', p_trade_id, 
        (SELECT locked_balance - v_amount_after_fee FROM p2p_balances WHERE user_id = p_seller_id AND crypto_type = p_crypto_type AND sandbox_mode = p_sandbox_mode),
        (SELECT locked_balance FROM p2p_balances WHERE user_id = p_seller_id AND crypto_type = p_crypto_type AND sandbox_mode = p_sandbox_mode),
        p_sandbox_mode
    );

    -- 2. Route Fee to Platform (p2p_platform_fees table)
    -- Ensure platform fee table exists
    CREATE TABLE IF NOT EXISTS p2p_platform_fees (
        id BIGSERIAL PRIMARY KEY,
        trade_id VARCHAR NOT NULL,
        crypto_type VARCHAR NOT NULL,
        amount DECIMAL NOT NULL,
        sandbox_mode BOOLEAN NOT NULL DEFAULT false,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    INSERT INTO p2p_platform_fees (trade_id, crypto_type, amount, sandbox_mode)
    VALUES (p_trade_id, p_crypto_type, v_fee_amount, p_sandbox_mode);

END;
$$ LANGUAGE plpgsql;
