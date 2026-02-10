-- Platform fee wallets: per-network addresses where fees are collected
CREATE TABLE IF NOT EXISTS platform_fee_wallets (
    id SERIAL PRIMARY KEY,
    network VARCHAR(50) UNIQUE NOT NULL,
    address VARCHAR(255) NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed with empty rows for each supported network
INSERT INTO
    platform_fee_wallets (network)
VALUES ('SOLANA'),
    ('ETHEREUM'),
    ('BSC'),
    ('POLYGON'),
    ('ARBITRUM') ON CONFLICT (network) DO NOTHING;