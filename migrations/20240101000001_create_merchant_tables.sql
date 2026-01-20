-- Create merchants table
-- Stores merchant account information, API credentials, and fee configuration
CREATE TABLE merchants (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    business_name VARCHAR(255) NOT NULL,
    api_key_hash VARCHAR(255) UNIQUE NOT NULL,
    fee_percentage DECIMAL(5,2) NOT NULL DEFAULT 1.50,  -- 1.5% default fee
    is_active BOOLEAN NOT NULL DEFAULT true,
    sandbox_mode BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for merchants table
CREATE INDEX idx_merchants_api_key ON merchants(api_key_hash);
CREATE INDEX idx_merchants_email ON merchants(email);
CREATE INDEX idx_merchants_is_active ON merchants(is_active);

-- Create merchant_wallets table
-- Stores blockchain wallet addresses for each merchant (one per blockchain)
CREATE TABLE merchant_wallets (
    id BIGSERIAL PRIMARY KEY,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    crypto_type VARCHAR(50) NOT NULL,  -- "SOL", "USDT_SPL", "USDT_BEP20", "USDT_ARBITRUM", "USDT_POLYGON"
    network VARCHAR(50) NOT NULL,      -- "SOLANA", "BEP20", "ARBITRUM", "POLYGON"
    address VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(merchant_id, crypto_type)
);

-- Indexes for merchant_wallets table
CREATE INDEX idx_merchant_wallets_merchant ON merchant_wallets(merchant_id);
CREATE INDEX idx_merchant_wallets_crypto_type ON merchant_wallets(crypto_type);
CREATE INDEX idx_merchant_wallets_is_active ON merchant_wallets(is_active);

-- Create webhook_configs table
-- Stores webhook configuration for each merchant
CREATE TABLE webhook_configs (
    id BIGSERIAL PRIMARY KEY,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    url VARCHAR(500) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(merchant_id)
);

-- Indexes for webhook_configs table
CREATE INDEX idx_webhook_configs_merchant ON webhook_configs(merchant_id);
CREATE INDEX idx_webhook_configs_is_active ON webhook_configs(is_active);

-- Add comments for documentation
COMMENT ON TABLE merchants IS 'Merchant accounts with API credentials and fee configuration';
COMMENT ON TABLE merchant_wallets IS 'Blockchain wallet addresses for merchants (one per blockchain type)';
COMMENT ON TABLE webhook_configs IS 'Webhook notification URLs for merchants';

COMMENT ON COLUMN merchants.api_key_hash IS 'Bcrypt hash of the merchant API key';
COMMENT ON COLUMN merchants.fee_percentage IS 'Platform fee percentage (0.1% - 5.0%)';
COMMENT ON COLUMN merchants.sandbox_mode IS 'Whether merchant is in sandbox testing mode';
COMMENT ON COLUMN merchant_wallets.crypto_type IS 'Cryptocurrency type identifier';
COMMENT ON COLUMN merchant_wallets.network IS 'Blockchain network identifier';
COMMENT ON COLUMN webhook_configs.url IS 'HTTPS endpoint for webhook notifications';
