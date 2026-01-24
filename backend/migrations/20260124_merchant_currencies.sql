-- Add merchant currency preferences
CREATE TABLE IF NOT EXISTS merchant_currencies (
    id BIGSERIAL PRIMARY KEY,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    currency_group VARCHAR(10) NOT NULL, -- 'USDT', 'ETH', 'SOL', 'BTC'
    network VARCHAR(20) NOT NULL, -- 'ETH', 'BSC', 'POLYGON', 'ARBITRUM', 'SOL'
    crypto_type VARCHAR(20) NOT NULL, -- 'USDT_ETH', 'USDT_BSC', etc.
    is_enabled BOOLEAN NOT NULL DEFAULT true,
    wallet_address VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(merchant_id, crypto_type)
);

-- Index for efficient lookups
CREATE INDEX IF NOT EXISTS idx_merchant_currencies_merchant_enabled 
ON merchant_currencies(merchant_id, is_enabled);

-- Add sample data for existing merchants
INSERT INTO merchant_currencies (merchant_id, currency_group, network, crypto_type, is_enabled)
SELECT 
    id as merchant_id,
    'USDT' as currency_group,
    'ETH' as network,
    'USDT_ETH' as crypto_type,
    true as is_enabled
FROM merchants 
WHERE id NOT IN (SELECT DISTINCT merchant_id FROM merchant_currencies)
ON CONFLICT (merchant_id, crypto_type) DO NOTHING;
