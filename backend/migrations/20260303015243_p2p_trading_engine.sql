-- Migration: P2P Trading Engine Core
-- This establishes the tables forAds, Trades, Payment Methods, and Escrow tracking.

-- 1. P2P Payment Methods
-- Stores the bank/fiat payment details a user accepts
CREATE TABLE IF NOT EXISTS p2p_payment_methods (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES p2p_profiles(id) ON DELETE CASCADE,
    method_name VARCHAR(100) NOT NULL, -- e.g., 'Bank Transfer', 'Chipper Cash', 'Opay'
    currency VARCHAR(10) NOT NULL, -- e.g., 'NGN', 'USD'
    account_name VARCHAR(255) NOT NULL,
    account_number VARCHAR(255) NOT NULL,
    bank_name VARCHAR(255),
    is_active BOOLEAN NOT NULL DEFAULT true,
    sandbox_mode BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_p2p_payment_methods_user ON p2p_payment_methods(user_id);

-- 2. P2P Ads (Buy/Sell Offers)
CREATE TABLE IF NOT EXISTS p2p_ads (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES p2p_profiles(id) ON DELETE CASCADE,
    ad_type VARCHAR(10) NOT NULL, -- 'BUY' or 'SELL' (From the Ad creator's perspective)
    crypto_type VARCHAR(50) NOT NULL, -- e.g., 'USDT'
    fiat_currency VARCHAR(10) NOT NULL, -- e.g., 'NGN'
    price DECIMAL(18, 2) NOT NULL, -- Fiat price per 1 Crypto
    total_amount DECIMAL(36, 18) NOT NULL, -- Total crypto available in this ad
    min_limit DECIMAL(18, 2) NOT NULL, -- Minimum fiat amount per trade
    max_limit DECIMAL(18, 2) NOT NULL, -- Maximum fiat amount per trade
    payment_time_limit INT NOT NULL DEFAULT 15, -- Minutes allowed to complete fiat payment
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE', -- 'ACTIVE', 'PAUSED', 'COMPLETED', 'CANCELLED'
    terms_and_conditions TEXT,
    auto_reply TEXT,
    sandbox_mode BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_p2p_ads_user ON p2p_ads(user_id);
CREATE INDEX IF NOT EXISTS idx_p2p_ads_fiat_crypto ON p2p_ads(fiat_currency, crypto_type);
CREATE INDEX IF NOT EXISTS idx_p2p_ads_status ON p2p_ads(status);

-- Junction table linking an Ad to allowed Payment Methods
CREATE TABLE IF NOT EXISTS p2p_ad_payment_methods (
    ad_id BIGINT NOT NULL REFERENCES p2p_ads(id) ON DELETE CASCADE,
    payment_method_id BIGINT NOT NULL REFERENCES p2p_payment_methods(id) ON DELETE CASCADE,
    PRIMARY KEY (ad_id, payment_method_id)
);

-- 3. P2P Trades (The Escrow Engine)
CREATE TABLE IF NOT EXISTS p2p_trades (
    id BIGSERIAL PRIMARY KEY,
    trade_id VARCHAR(50) UNIQUE NOT NULL, -- Publicly shareable ID (e.g., TRD-123456)
    ad_id BIGINT NOT NULL REFERENCES p2p_ads(id) ON DELETE RESTRICT,
    maker_id BIGINT NOT NULL REFERENCES p2p_profiles(id) ON DELETE RESTRICT, -- The person who created the Ad
    taker_id BIGINT NOT NULL REFERENCES p2p_profiles(id) ON DELETE RESTRICT, -- The person who responded to the Ad
    crypto_amount DECIMAL(36, 18) NOT NULL,
    fiat_amount DECIMAL(18, 2) NOT NULL,
    price DECIMAL(18, 2) NOT NULL,
    status VARCHAR(30) NOT NULL, -- 'PENDING_PAYMENT', 'PAID', 'RELEASED', 'CANCELLED', 'DISPUTED'
    payment_method_id BIGINT NOT NULL REFERENCES p2p_payment_methods(id) ON DELETE RESTRICT,
    expires_at TIMESTAMPTZ NOT NULL, -- Timestamp when the trade auto-cancels if not marked PAID
    paid_at TIMESTAMPTZ, -- Timestamp when taker marked it as paid
    completed_at TIMESTAMPTZ, -- Timestamp when funds were released or trade cancelled
    disputed_at TIMESTAMPTZ,
    cancel_reason VARCHAR(255),
    sandbox_mode BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_p2p_trades_maker ON p2p_trades(maker_id);
CREATE INDEX IF NOT EXISTS idx_p2p_trades_taker ON p2p_trades(taker_id);
CREATE INDEX IF NOT EXISTS idx_p2p_trades_status ON p2p_trades(status);
CREATE INDEX IF NOT EXISTS idx_p2p_trades_trade_id ON p2p_trades(trade_id);

-- 4. P2P Trade Chat Messages
CREATE TABLE IF NOT EXISTS p2p_chat_messages (
    id BIGSERIAL PRIMARY KEY,
    trade_id BIGINT NOT NULL REFERENCES p2p_trades(id) ON DELETE CASCADE,
    sender_id BIGINT NOT NULL REFERENCES p2p_profiles(id) ON DELETE CASCADE,
    message TEXT NOT NULL,
    attachment_url VARCHAR(500), -- For proof of payment screenshots
    is_system_message BOOLEAN NOT NULL DEFAULT false, -- e.g., "Buyer marked order as paid"
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_p2p_chat_trade ON p2p_chat_messages(trade_id);
CREATE INDEX IF NOT EXISTS idx_p2p_chat_created ON p2p_chat_messages(created_at);

-- Comments
COMMENT ON TABLE p2p_ads IS 'Buy and Sell offers created by users';
COMMENT ON TABLE p2p_trades IS 'Active trades matching an Ad, acting as the Escrow lock mechanism';
COMMENT ON COLUMN p2p_trades.status IS 'PENDING_PAYMENT -> PAID -> RELEASED. Or DISPUTED/CANCELLED.';
