-- Migration: Extended Wallet Safeguards
-- Add protection for customer wallets and enhance history archiving

-- 1. Add customer_wallets_locked to merchants table
ALTER TABLE merchants ADD COLUMN customer_wallets_locked BOOLEAN NOT NULL DEFAULT false;

-- 2. Modify merchant_wallet_history to support customer wallets and detailed state
-- First, add columns (some can be NULL initially for local backfill if needed, but here we can just add them)
ALTER TABLE merchant_wallet_history 
    ADD COLUMN owner_type VARCHAR(20) NOT NULL DEFAULT 'merchant',
    ADD COLUMN customer_id BIGINT REFERENCES merchant_customers(id) ON DELETE CASCADE,
    ADD COLUMN wallet_mode VARCHAR(20) DEFAULT 'address_only',
    ADD COLUMN encrypted_private_key TEXT,
    ADD COLUMN is_active BOOLEAN DEFAULT true;

-- 3. Add index for customer history
CREATE INDEX idx_wallet_history_customer ON merchant_wallet_history(customer_id);
CREATE INDEX idx_wallet_history_owner_type ON merchant_wallet_history(owner_type);

-- Add comments
COMMENT ON COLUMN merchants.customer_wallets_locked IS 'If true, merchant customer wallet addresses cannot be changed';
COMMENT ON COLUMN merchant_wallet_history.owner_type IS 'Distinguishes between merchant and customer wallets';
COMMENT ON COLUMN merchant_wallet_history.wallet_mode IS 'The wallet mode at the time of archiving (managed, address_only, imported)';
