-- Consolidate crypto_type aliases in merchant_forwarding_wallets
-- Fix: The frontend sent aliases (USDT_SOL, USDT_BSC) instead of canonical names (USDT_SPL, USDT_BEP20)
-- This migration standardizes these remaining duplicates.

-- Step 1: Update old alias entries (USDT_SOL, USDT_BSC) to canonical naming 
-- IF the merchant doesn't already have the canonical entry.
UPDATE merchant_forwarding_wallets SET crypto_type = 'USDT_SPL'
WHERE crypto_type = 'USDT_SOL' AND merchant_id NOT IN (
    SELECT merchant_id FROM merchant_forwarding_wallets WHERE crypto_type = 'USDT_SPL'
);

UPDATE merchant_forwarding_wallets SET crypto_type = 'USDT_BEP20'
WHERE crypto_type = 'USDT_BSC' AND merchant_id NOT IN (
    SELECT merchant_id FROM merchant_forwarding_wallets WHERE crypto_type = 'USDT_BEP20'
);

-- Step 2: Delete remaining duplicates (the canonical underscore row already exists)
DELETE FROM merchant_forwarding_wallets WHERE crypto_type IN ('USDT_SOL', 'USDT_BSC');

-- Step 3: Also normalize merchant_wallets table if it has the same issue
UPDATE merchant_wallets SET crypto_type = 'USDT_SPL' WHERE crypto_type = 'USDT_SOL' AND merchant_id NOT IN (SELECT merchant_id FROM merchant_wallets WHERE crypto_type = 'USDT_SPL');
UPDATE merchant_wallets SET crypto_type = 'USDT_BEP20' WHERE crypto_type = 'USDT_BSC' AND merchant_id NOT IN (SELECT merchant_id FROM merchant_wallets WHERE crypto_type = 'USDT_BEP20');
DELETE FROM merchant_wallets WHERE crypto_type IN ('USDT_SOL', 'USDT_BSC');

-- Step 4: Normalize payment_transactions table
UPDATE payment_transactions SET crypto_type = 'USDT_SPL' WHERE crypto_type = 'USDT_SOL';
UPDATE payment_transactions SET crypto_type = 'USDT_BEP20' WHERE crypto_type = 'USDT_BSC';
