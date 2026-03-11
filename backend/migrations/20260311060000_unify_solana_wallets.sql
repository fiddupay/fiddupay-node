-- Migration: Unify Solana Wallets
-- Rename USDT_SOL to USDT_SPL and backfill missing SOL/WSOL/USDT_SPL records

-- 1. Remove USDT_SOL records if a USDT_SPL record already exists for the same merchant/sandbox
DELETE FROM merchant_wallets mw1
WHERE crypto_type = 'USDT_SOL'
AND EXISTS (
    SELECT 1 FROM merchant_wallets mw2
    WHERE mw2.merchant_id = mw1.merchant_id
    AND mw2.sandbox_mode = mw1.sandbox_mode
    AND mw2.crypto_type = 'USDT_SPL'
);

-- 2. Rename remaining USDT_SOL to USDT_SPL
UPDATE merchant_wallets 
SET crypto_type = 'USDT_SPL' 
WHERE crypto_type = 'USDT_SOL';

-- 3. Same for balances: Remove USDT_SOL if USDT_SPL exists
DELETE FROM merchant_balances mb1
WHERE crypto_type = 'USDT_SOL'
AND EXISTS (
    SELECT 1 FROM merchant_balances mb2
    WHERE mb2.merchant_id = mb1.merchant_id
    AND mb2.sandbox_mode = mb1.sandbox_mode
    AND mb2.crypto_type = 'USDT_SPL'
);

-- 4. Rename remaining USDT_SOL to USDT_SPL in merchant_balances
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
ON CONFLICT (merchant_id, crypto_type, sandbox_mode) DO NOTHING;

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
ON CONFLICT (merchant_id, crypto_type, sandbox_mode) DO NOTHING;
