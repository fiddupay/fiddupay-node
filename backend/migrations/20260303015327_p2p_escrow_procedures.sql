-- Migration: P2P Escrow Ledger Functions
-- These stored procedures ensure atomic and failure-proof operations on the internal p2p balances.

-- Function 1: Escrow Lock (Triggered when a trade is created)
-- Deducts `available_balance` from the seller and adds to `locked_balance`.
CREATE OR REPLACE FUNCTION p2p_lock_funds_in_escrow(
    p_seller_id BIGINT,
    p_crypto_type VARCHAR,
    p_amount DECIMAL,
    p_trade_id VARCHAR,
    p_sandbox_mode BOOLEAN
) RETURNS VOID AS $$
DECLARE
    v_current_available DECIMAL;
BEGIN
    -- 1. Check if the seller has enough available balance
    SELECT available_balance INTO v_current_available
    FROM p2p_balances
    WHERE user_id = p_seller_id AND crypto_type = p_crypto_type AND sandbox_mode = p_sandbox_mode
    FOR UPDATE; -- Lock the row to prevent race conditions
    
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Seller balance record not found.';
    END IF;

    IF v_current_available < p_amount THEN
        RAISE EXCEPTION 'Insufficient available balance. Required: %, Available: %', p_amount, v_current_available;
    END IF;

    -- 2. Move funds from available to locked
    UPDATE p2p_balances
    SET 
        available_balance = available_balance - p_amount,
        locked_balance = locked_balance + p_amount,
        last_updated = NOW()
    WHERE user_id = p_seller_id AND crypto_type = p_crypto_type AND sandbox_mode = p_sandbox_mode;

    -- 3. Log the history record (Debit Available, Credit Locked)
    INSERT INTO p2p_balance_history (
        user_id, crypto_type, amount, balance_type, change_type, reason, reference_id, balance_before, balance_after, sandbox_mode
    ) VALUES (
        p_seller_id, p_crypto_type, p_amount, 'AVAILABLE', 'DEBIT', 'ESCROW_LOCK', p_trade_id, v_current_available, v_current_available - p_amount, p_sandbox_mode
    );

    INSERT INTO p2p_balance_history (
        user_id, crypto_type, amount, balance_type, change_type, reason, reference_id, balance_before, balance_after, sandbox_mode
    ) VALUES (
        p_seller_id, p_crypto_type, p_amount, 'LOCKED', 'CREDIT', 'ESCROW_LOCK', p_trade_id, 
        (SELECT locked_balance - p_amount FROM p2p_balances WHERE user_id = p_seller_id AND crypto_type = p_crypto_type AND sandbox_mode = p_sandbox_mode),
        (SELECT locked_balance FROM p2p_balances WHERE user_id = p_seller_id AND crypto_type = p_crypto_type AND sandbox_mode = p_sandbox_mode),
        p_sandbox_mode
    );

END;
$$ LANGUAGE plpgsql;

-- Function 2: Escrow Release (Triggered when seller confirms payment)
-- Removes funds from seller's `locked_balance` and adds to buyer's `available_balance`.
CREATE OR REPLACE FUNCTION p2p_release_funds_from_escrow(
    p_seller_id BIGINT,
    p_buyer_id BIGINT,
    p_crypto_type VARCHAR,
    p_amount DECIMAL,
    p_trade_id VARCHAR,
    p_sandbox_mode BOOLEAN
) RETURNS VOID AS $$
DECLARE
    v_seller_locked DECIMAL;
    v_buyer_available DECIMAL;
BEGIN
    -- 1. Deduct from Seller's Locked Balance
    SELECT locked_balance INTO v_seller_locked
    FROM p2p_balances
    WHERE user_id = p_seller_id AND crypto_type = p_crypto_type AND sandbox_mode = p_sandbox_mode
    FOR UPDATE;

    IF NOT FOUND OR v_seller_locked < p_amount THEN
        RAISE EXCEPTION 'Seller locked balance is insufficient to release trade.';
    END IF;

    UPDATE p2p_balances
    SET 
        locked_balance = locked_balance - p_amount,
        last_updated = NOW()
    WHERE user_id = p_seller_id AND crypto_type = p_crypto_type AND sandbox_mode = p_sandbox_mode;

    INSERT INTO p2p_balance_history (
        user_id, crypto_type, amount, balance_type, change_type, reason, reference_id, balance_before, balance_after, sandbox_mode
    ) VALUES (
        p_seller_id, p_crypto_type, p_amount, 'LOCKED', 'DEBIT', 'TRADE_RELEASE', p_trade_id, v_seller_locked, v_seller_locked - p_amount, p_sandbox_mode
    );

    -- 2. Add to Buyer's Available Balance
    -- Insert a 0 balance row if the buyer doesn't have an active balance record for this asset yet
    INSERT INTO p2p_balances (user_id, crypto_type, available_balance, locked_balance, sandbox_mode)
    VALUES (p_buyer_id, p_crypto_type, 0, 0, p_sandbox_mode)
    ON CONFLICT (user_id, crypto_type, sandbox_mode) DO NOTHING;

    SELECT available_balance INTO v_buyer_available
    FROM p2p_balances
    WHERE user_id = p_buyer_id AND crypto_type = p_crypto_type AND sandbox_mode = p_sandbox_mode
    FOR UPDATE;

    UPDATE p2p_balances
    SET 
        available_balance = available_balance + p_amount,
        last_updated = NOW()
    WHERE user_id = p_buyer_id AND crypto_type = p_crypto_type AND sandbox_mode = p_sandbox_mode;

    INSERT INTO p2p_balance_history (
        user_id, crypto_type, amount, balance_type, change_type, reason, reference_id, balance_before, balance_after, sandbox_mode
    ) VALUES (
        p_buyer_id, p_crypto_type, p_amount, 'AVAILABLE', 'CREDIT', 'TRADE_RELEASE', p_trade_id, v_buyer_available, v_buyer_available + p_amount, p_sandbox_mode
    );

END;
$$ LANGUAGE plpgsql;

-- Function 3: Escrow Cancel (Triggered when trade expires or is cancelled)
-- Removes funds from seller's `locked_balance` and returns it to seller's `available_balance`.
CREATE OR REPLACE FUNCTION p2p_cancel_escrow(
    p_seller_id BIGINT,
    p_crypto_type VARCHAR,
    p_amount DECIMAL,
    p_trade_id VARCHAR,
    p_sandbox_mode BOOLEAN
) RETURNS VOID AS $$
DECLARE
    v_seller_locked DECIMAL;
    v_seller_available DECIMAL;
BEGIN
    -- Fetch balances with lock
    SELECT locked_balance, available_balance INTO v_seller_locked, v_seller_available
    FROM p2p_balances
    WHERE user_id = p_seller_id AND crypto_type = p_crypto_type AND sandbox_mode = p_sandbox_mode
    FOR UPDATE;

    IF NOT FOUND OR v_seller_locked < p_amount THEN
        RAISE EXCEPTION 'Seller locked balance is insufficient to cancel trade.';
    END IF;

    -- Update balances: Return from locked to available
    UPDATE p2p_balances
    SET 
        locked_balance = locked_balance - p_amount,
        available_balance = available_balance + p_amount,
        last_updated = NOW()
    WHERE user_id = p_seller_id AND crypto_type = p_crypto_type AND sandbox_mode = p_sandbox_mode;

    -- History
    INSERT INTO p2p_balance_history (
        user_id, crypto_type, amount, balance_type, change_type, reason, reference_id, balance_before, balance_after, sandbox_mode
    ) VALUES (
        p_seller_id, p_crypto_type, p_amount, 'LOCKED', 'DEBIT', 'TRADE_CANCELLED', p_trade_id, v_seller_locked, v_seller_locked - p_amount, p_sandbox_mode
    );

    INSERT INTO p2p_balance_history (
        user_id, crypto_type, amount, balance_type, change_type, reason, reference_id, balance_before, balance_after, sandbox_mode
    ) VALUES (
        p_seller_id, p_crypto_type, p_amount, 'AVAILABLE', 'CREDIT', 'TRADE_CANCELLED', p_trade_id, v_seller_available, v_seller_available + p_amount, p_sandbox_mode
    );

END;
$$ LANGUAGE plpgsql;
