-- Migration: Unify Solana Wallets
-- Rename USDT_SOL to USDT_SPL and backfill missing SOL/WSOL/USDT_SPL records

-- 1. Rename any existing USDT_SOL to USDT_SPL in merchant_wallets
UPDATE merchant_wallets 
SET crypto_type = 'USDT_SPL' 
WHERE crypto_type = 'USDT_SOL';

-- 2. Rename in merchant_balances if present
UPDATE merchant_balances
SET crypto_type = 'USDT_SPL'
WHERE crypto_type = 'USDT_SOL';

-- 3. Backfill USDT_SPL for all merchants who have SOL but mission USDT_SPL
INSERT INTO merchant_wallets (merchant_id, crypto_type, network, address, is_active, sandbox_mode, created_at, updated_at)
SELECT 
    sol.merchant_id, 
    'USDT_SPL', 
    'SOLANA', 
    sol.address, 
    sol.is_active, 
    sol.sandbox_mode,
    NOW(), 
    NOW()
FROM merchant_wallets sol
WHERE sol.crypto_type = 'SOL'
AND NOT EXISTS (
    SELECT 1 FROM merchant_wallets usdt 
    WHERE usdt.merchant_id = sol.merchant_id 
    AND usdt.crypto_type = 'USDT_SPL'
    AND usdt.sandbox_mode = sol.sandbox_mode
)
ON CONFLICT (merchant_id, crypto_type) DO NOTHING;

-- 4. Backfill WSOL for all merchants who have SOL but missing WSOL
INSERT INTO merchant_wallets (merchant_id, crypto_type, network, address, is_active, sandbox_mode, created_at, updated_at)
SELECT 
    sol.merchant_id, 
    'WSOL', 
    'SOLANA', 
    sol.address, 
    sol.is_active, 
    sol.sandbox_mode,
    NOW(), 
    NOW()
FROM merchant_wallets sol
WHERE sol.crypto_type = 'SOL'
AND NOT EXISTS (
    SELECT 1 FROM merchant_wallets wsol 
    WHERE wsol.merchant_id = sol.merchant_id 
    AND wsol.crypto_type = 'WSOL'
    AND wsol.sandbox_mode = sol.sandbox_mode
)
ON CONFLICT (merchant_id, crypto_type) DO NOTHING;
