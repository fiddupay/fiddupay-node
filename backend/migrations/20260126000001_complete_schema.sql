-- Complete FidduPay Database Schema
-- Creates all tables, columns, indices, and relations

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Merchants table
CREATE TABLE IF NOT EXISTS merchants (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    business_name VARCHAR(255) NOT NULL,
    api_key VARCHAR(255) UNIQUE NOT NULL,
    webhook_url TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Merchant wallets table
CREATE TABLE IF NOT EXISTS merchant_wallets (
    id BIGSERIAL PRIMARY KEY,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id),
    crypto_type VARCHAR(50) NOT NULL,
    network VARCHAR(50) NOT NULL,
    address VARCHAR(255) NOT NULL,
    wallet_mode VARCHAR(50) DEFAULT 'address_only',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(merchant_id, crypto_type)
);

-- Merchant balances table
CREATE TABLE IF NOT EXISTS merchant_balances (
    id BIGSERIAL PRIMARY KEY,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id),
    crypto_type VARCHAR(50) NOT NULL,
    available_balance DECIMAL(20,8) DEFAULT 0,
    reserved_balance DECIMAL(20,8) DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(merchant_id, crypto_type)
);

-- Payment transactions table
CREATE TABLE IF NOT EXISTS payment_transactions (
    id BIGSERIAL PRIMARY KEY,
    payment_id VARCHAR(255) UNIQUE NOT NULL,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id),
    amount DECIMAL(20,8) NOT NULL,
    crypto_type VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    to_address VARCHAR(255) NOT NULL,
    from_address VARCHAR(255),
    tx_hash VARCHAR(255),
    block_number BIGINT,
    confirmations INTEGER DEFAULT 0,
    expires_at TIMESTAMP WITH TIME ZONE,
    webhook_url TEXT,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Address-only payments table (new)
CREATE TABLE IF NOT EXISTS address_only_payments (
    id BIGSERIAL PRIMARY KEY,
    payment_id VARCHAR(255) UNIQUE NOT NULL,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id),
    crypto_type VARCHAR(50) NOT NULL,
    gateway_deposit_address VARCHAR(255) NOT NULL,
    merchant_destination_address VARCHAR(255) NOT NULL,
    requested_amount DECIMAL(20,8) NOT NULL,
    processing_fee DECIMAL(20,8) NOT NULL,
    forwarding_amount DECIMAL(20,8) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'PendingPayment',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Address-only forwarding transactions table (new)
CREATE TABLE IF NOT EXISTS address_only_forwarding_txs (
    id BIGSERIAL PRIMARY KEY,
    payment_id VARCHAR(255) NOT NULL REFERENCES address_only_payments(payment_id),
    destination_address VARCHAR(255) NOT NULL,
    amount DECIMAL(20,8) NOT NULL,
    gas_fee DECIMAL(20,8) NOT NULL,
    tx_hash VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    confirmed_at TIMESTAMP WITH TIME ZONE
);

-- Deposit keypairs table (new)
CREATE TABLE IF NOT EXISTS deposit_keypairs (
    id BIGSERIAL PRIMARY KEY,
    payment_id VARCHAR(255) NOT NULL,
    address VARCHAR(255) UNIQUE NOT NULL,
    encrypted_private_key TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Webhook logs table (new)
CREATE TABLE IF NOT EXISTS webhook_logs (
    id BIGSERIAL PRIMARY KEY,
    payment_id VARCHAR(255) NOT NULL,
    webhook_url TEXT NOT NULL,
    status_code INTEGER NOT NULL,
    attempted_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Withdrawals table
CREATE TABLE IF NOT EXISTS withdrawals (
    id BIGSERIAL PRIMARY KEY,
    withdrawal_id VARCHAR(255) UNIQUE NOT NULL,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id),
    crypto_type VARCHAR(50) NOT NULL,
    amount DECIMAL(20,8) NOT NULL,
    destination_address VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'PENDING',
    tx_hash VARCHAR(255),
    rejection_reason TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE
);

-- Deposit addresses table (existing)
CREATE TABLE IF NOT EXISTS deposit_addresses (
    id BIGSERIAL PRIMARY KEY,
    payment_id VARCHAR(255) UNIQUE NOT NULL,
    crypto_type VARCHAR(50) NOT NULL,
    deposit_address VARCHAR(255) UNIQUE NOT NULL,
    private_key_encrypted TEXT NOT NULL,
    merchant_destination VARCHAR(255) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    status VARCHAR(50) DEFAULT 'ACTIVE',
    forwarded_at TIMESTAMP WITH TIME ZONE,
    forward_tx_hash VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create all indexes for performance
CREATE INDEX IF NOT EXISTS idx_merchants_email ON merchants(email);
CREATE INDEX IF NOT EXISTS idx_merchants_api_key ON merchants(api_key);

CREATE INDEX IF NOT EXISTS idx_merchant_wallets_merchant_id ON merchant_wallets(merchant_id);
CREATE INDEX IF NOT EXISTS idx_merchant_wallets_crypto_type ON merchant_wallets(crypto_type);

CREATE INDEX IF NOT EXISTS idx_merchant_balances_merchant_id ON merchant_balances(merchant_id);
CREATE INDEX IF NOT EXISTS idx_merchant_balances_crypto_type ON merchant_balances(crypto_type);

CREATE INDEX IF NOT EXISTS idx_payment_transactions_payment_id ON payment_transactions(payment_id);
CREATE INDEX IF NOT EXISTS idx_payment_transactions_merchant_id ON payment_transactions(merchant_id);
CREATE INDEX IF NOT EXISTS idx_payment_transactions_status ON payment_transactions(status);
CREATE INDEX IF NOT EXISTS idx_payment_transactions_tx_hash ON payment_transactions(tx_hash);

CREATE INDEX IF NOT EXISTS idx_address_only_payments_merchant_id ON address_only_payments(merchant_id);
CREATE INDEX IF NOT EXISTS idx_address_only_payments_status ON address_only_payments(status);
CREATE INDEX IF NOT EXISTS idx_address_only_payments_deposit_address ON address_only_payments(gateway_deposit_address);

CREATE INDEX IF NOT EXISTS idx_address_only_forwarding_payment_id ON address_only_forwarding_txs(payment_id);

CREATE INDEX IF NOT EXISTS idx_deposit_keypairs_address ON deposit_keypairs(address);
CREATE INDEX IF NOT EXISTS idx_deposit_keypairs_payment_id ON deposit_keypairs(payment_id);

CREATE INDEX IF NOT EXISTS idx_webhook_logs_payment_id ON webhook_logs(payment_id);

CREATE INDEX IF NOT EXISTS idx_withdrawals_merchant_id ON withdrawals(merchant_id);
CREATE INDEX IF NOT EXISTS idx_withdrawals_status ON withdrawals(status);

CREATE INDEX IF NOT EXISTS idx_deposit_addresses_payment_id ON deposit_addresses(payment_id);
CREATE INDEX IF NOT EXISTS idx_deposit_addresses_deposit_address ON deposit_addresses(deposit_address);
CREATE INDEX IF NOT EXISTS idx_deposit_addresses_status ON deposit_addresses(status);

-- Insert sample data for testing
INSERT INTO merchants (email, business_name, api_key) VALUES 
('test@fiddupay.com', 'Test Business', 'test_api_key_123') 
ON CONFLICT (email) DO NOTHING;

-- Success message
SELECT 'Database schema created successfully!' as result;
