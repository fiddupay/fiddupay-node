-- Migration: Isolate wallets by sandbox_mode
-- This adds sandbox_mode to allow separate configurations for Live and Sandbox environments

-- 1. Update merchant_wallets
ALTER TABLE merchant_wallets 
ADD COLUMN IF NOT EXISTS sandbox_mode BOOLEAN NOT NULL DEFAULT false;

-- Drop existing unique constraint
ALTER TABLE merchant_wallets 
DROP CONSTRAINT IF EXISTS merchant_wallets_merchant_id_crypto_type_key;

-- Add new unique constraint including sandbox_mode
ALTER TABLE merchant_wallets 
ADD CONSTRAINT merchant_wallets_unique_merchant_crypto_sandbox UNIQUE (merchant_id, crypto_type, sandbox_mode);

-- Create index for filtering
CREATE INDEX IF NOT EXISTS idx_merchant_wallets_sandbox ON merchant_wallets (sandbox_mode);


-- 2. Update merchant_forwarding_wallets
ALTER TABLE merchant_forwarding_wallets 
ADD COLUMN IF NOT EXISTS sandbox_mode BOOLEAN NOT NULL DEFAULT false;

-- Drop existing unique constraint
ALTER TABLE merchant_forwarding_wallets 
DROP CONSTRAINT IF EXISTS merchant_forwarding_wallets_merchant_id_crypto_type_key;

-- Add new unique constraint including sandbox_mode
ALTER TABLE merchant_forwarding_wallets 
ADD CONSTRAINT merchant_fwd_wallets_unique_merchant_crypto_sandbox UNIQUE (merchant_id, crypto_type, sandbox_mode);

-- Create index for filtering
CREATE INDEX IF NOT EXISTS idx_merchant_fwd_wallets_sandbox ON merchant_forwarding_wallets (sandbox_mode);

-- 3. Comment for documentation
COMMENT ON COLUMN merchant_wallets.sandbox_mode IS 'Isolation: true for sandbox environment, false for production';
COMMENT ON COLUMN merchant_forwarding_wallets.sandbox_mode IS 'Isolation: true for sandbox environment, false for production';
