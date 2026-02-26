-- Migration: Create tables for Merchant Customer Designated Wallets
-- Allows merchants to provision unique wallets for their own platform users

-- 1. Merchant Customers table
-- Links a merchant to their own external users/customers
CREATE TABLE merchant_customers (
    id BIGSERIAL PRIMARY KEY,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    external_id VARCHAR(100) NOT NULL, -- The user-id from the merchant's platform
    email VARCHAR(255),
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(merchant_id, external_id)
);

-- 2. Merchant Customer Wallets
-- Tracks unique blockchain addresses and encrypted keys for each customer
CREATE TABLE merchant_customer_wallets (
    id BIGSERIAL PRIMARY KEY,
    customer_id BIGINT NOT NULL REFERENCES merchant_customers(id) ON DELETE CASCADE,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    crypto_type VARCHAR(50) NOT NULL,
    network VARCHAR(50) NOT NULL,
    address VARCHAR(255) NOT NULL,
    encrypted_private_key TEXT NOT NULL, -- Encrypted using platform master key
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(customer_id, crypto_type)
);

-- 3. Merchant Customer Balances
-- Tracks real-time balances for these customer wallets
CREATE TABLE merchant_customer_balances (
    id BIGSERIAL PRIMARY KEY,
    customer_id BIGINT NOT NULL REFERENCES merchant_customers(id) ON DELETE CASCADE,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    crypto_type VARCHAR(50) NOT NULL,
    available_balance DECIMAL(20,8) NOT NULL DEFAULT 0,
    locked_balance DECIMAL(20,8) NOT NULL DEFAULT 0,
    total_balance DECIMAL(20,8) NOT NULL DEFAULT 0,
    last_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(customer_id, crypto_type)
);

-- Indexes for performance
CREATE INDEX idx_merchant_customers_external_id ON merchant_customers(external_id);
CREATE INDEX idx_merchant_customers_merchant ON merchant_customers(merchant_id);
CREATE INDEX idx_merchant_customer_wallets_address ON merchant_customer_wallets(address);
CREATE INDEX idx_merchant_customer_wallets_customer ON merchant_customer_wallets(customer_id);
CREATE INDEX idx_merchant_customer_balances_customer ON merchant_customer_balances(customer_id);

-- Trigger for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_merchant_customers_updated_at
    BEFORE UPDATE ON merchant_customers
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_merchant_customer_wallets_updated_at
    BEFORE UPDATE ON merchant_customer_wallets
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Add comments for documentation
COMMENT ON TABLE merchant_customers IS 'Tracks users/customers belonging to a specific merchant for sub-account features';
COMMENT ON TABLE merchant_customer_wallets IS 'Designated deposit wallets generated for merchant customers';
COMMENT ON TABLE merchant_customer_balances IS 'Balances for merchant customer wallets, updated by the monitoring service';
