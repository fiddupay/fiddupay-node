-- Create merchant_forwarding_wallets table
-- Stores forwarding destination addresses separately from managed/imported wallets
-- This ensures forwarding mode addresses never mix with custodial wallet data
CREATE TABLE IF NOT EXISTS merchant_forwarding_wallets (
    id BIGSERIAL PRIMARY KEY,
    merchant_id BIGINT NOT NULL REFERENCES merchants (id) ON DELETE CASCADE,
    crypto_type VARCHAR(50) NOT NULL,
    network VARCHAR(50) NOT NULL,
    address VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (merchant_id, crypto_type)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_forwarding_wallets_merchant ON merchant_forwarding_wallets (merchant_id);

CREATE INDEX IF NOT EXISTS idx_forwarding_wallets_crypto ON merchant_forwarding_wallets (crypto_type);

CREATE INDEX IF NOT EXISTS idx_forwarding_wallets_active ON merchant_forwarding_wallets (is_active);

COMMENT ON
TABLE merchant_forwarding_wallets IS 'Forwarding-mode destination addresses, isolated from managed/imported wallet configs';