-- Address-Only Mode Database Schema (Phase 1)
-- Supports native currencies with auto-forwarding

-- Main address-only payments table
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

-- Forwarding transactions tracking
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

-- Webhook logs for debugging
CREATE TABLE IF NOT EXISTS webhook_logs (
    id BIGSERIAL PRIMARY KEY,
    payment_id VARCHAR(255) NOT NULL,
    webhook_url TEXT NOT NULL,
    status_code INTEGER NOT NULL,
    attempted_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Add webhook URL to merchants table if not exists
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS webhook_url TEXT;

-- Deposit keypairs for secure key storage
CREATE TABLE IF NOT EXISTS deposit_keypairs (
    id BIGSERIAL PRIMARY KEY,
    payment_id VARCHAR(255) NOT NULL,
    address VARCHAR(255) UNIQUE NOT NULL,
    encrypted_private_key TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_address_only_payments_merchant_id ON address_only_payments(merchant_id);
CREATE INDEX IF NOT EXISTS idx_address_only_payments_status ON address_only_payments(status);
CREATE INDEX IF NOT EXISTS idx_address_only_payments_deposit_address ON address_only_payments(gateway_deposit_address);
CREATE INDEX IF NOT EXISTS idx_address_only_forwarding_payment_id ON address_only_forwarding_txs(payment_id);

-- Add address-only mode to wallet configurations
ALTER TABLE merchant_wallets ADD COLUMN IF NOT EXISTS wallet_mode VARCHAR(50) DEFAULT 'address_only';

-- Update existing wallet configurations
UPDATE merchant_wallets SET wallet_mode = 'address_only' WHERE wallet_mode IS NULL;
