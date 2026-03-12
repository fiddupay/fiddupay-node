-- Migration: Wallet Safeguards and History
-- Add safeguards against accidental wallet changes and audit trail for old addresses

-- 1. Add wallets_locked to merchants table
ALTER TABLE merchants ADD COLUMN wallets_locked BOOLEAN NOT NULL DEFAULT false;

-- 2. Create merchant_wallet_history table
CREATE TABLE merchant_wallet_history (
    id BIGSERIAL PRIMARY KEY,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    crypto_type VARCHAR(50) NOT NULL,
    network VARCHAR(50) NOT NULL,
    old_address VARCHAR(255) NOT NULL,
    new_address VARCHAR(255) NOT NULL,
    changed_by VARCHAR(50) NOT NULL DEFAULT 'merchant', -- 'merchant', 'admin', 'system'
    reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 3. Add index for history lookups
CREATE INDEX idx_wallet_history_merchant ON merchant_wallet_history(merchant_id);
CREATE INDEX idx_wallet_history_crypto ON merchant_wallet_history(merchant_id, crypto_type);

-- Add comments
COMMENT ON COLUMN merchants.wallets_locked IS 'If true, merchant wallet addresses cannot be changed without unlocking first';
COMMENT ON TABLE merchant_wallet_history IS 'Audit trail for old merchant wallet addresses';
