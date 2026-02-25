-- Normalize crypto_type naming in merchant_forwarding_wallets
-- Fix: Old Display impl used hyphens (USDT-Arbitrum), new uses underscores (USDT_ARBITRUM)
-- This migration consolidates duplicates and standardizes naming

-- Step 1: For each merchant, if both old (hyphenated) and new (underscored) entries exist,
-- copy the address from the active one to the other, then delete the old one.

-- Update old hyphenated entries to new underscore naming (keeping the active address)
UPDATE merchant_forwarding_wallets SET crypto_type = 'USDT_ARBITRUM'
WHERE crypto_type = 'USDT-Arbitrum' AND id NOT IN (
    SELECT id FROM merchant_forwarding_wallets WHERE crypto_type = 'USDT_ARBITRUM'
);

UPDATE merchant_forwarding_wallets SET crypto_type = 'USDT_POLYGON'
WHERE crypto_type = 'USDT-Polygon' AND id NOT IN (
    SELECT id FROM merchant_forwarding_wallets WHERE crypto_type = 'USDT_POLYGON'
);

UPDATE merchant_forwarding_wallets SET crypto_type = 'USDT_ETH'
WHERE crypto_type = 'USDT-ERC20' AND id NOT IN (
    SELECT id FROM merchant_forwarding_wallets WHERE crypto_type = 'USDT_ETH'
);

UPDATE merchant_forwarding_wallets SET crypto_type = 'USDT_BEP20'
WHERE crypto_type = 'USDT-BEP20' AND id NOT IN (
    SELECT id FROM merchant_forwarding_wallets WHERE crypto_type = 'USDT_BEP20'
);

-- Step 2: Delete remaining old hyphenated duplicates (the new underscore row already exists)
DELETE FROM merchant_forwarding_wallets WHERE crypto_type IN ('USDT-Arbitrum', 'USDT-Polygon', 'USDT-ERC20', 'USDT-BEP20');

-- Step 3: Clean up stale deactivated entries with empty addresses
DELETE FROM merchant_forwarding_wallets WHERE address = '' AND is_active = false;

-- Step 4: Also normalize merchant_wallets table if it has the same issue
UPDATE merchant_wallets SET crypto_type = 'USDT_ARBITRUM' WHERE crypto_type = 'USDT-Arbitrum';
UPDATE merchant_wallets SET crypto_type = 'USDT_POLYGON' WHERE crypto_type = 'USDT-Polygon';
UPDATE merchant_wallets SET crypto_type = 'USDT_ETH' WHERE crypto_type = 'USDT-ERC20';
UPDATE merchant_wallets SET crypto_type = 'USDT_BEP20' WHERE crypto_type = 'USDT-BEP20';

-- Step 5: Normalize payment_transactions table
UPDATE payment_transactions SET crypto_type = 'USDT_ARBITRUM' WHERE crypto_type = 'USDT-Arbitrum';
UPDATE payment_transactions SET crypto_type = 'USDT_POLYGON' WHERE crypto_type = 'USDT-Polygon';
UPDATE payment_transactions SET crypto_type = 'USDT_ETH' WHERE crypto_type = 'USDT-ERC20';
UPDATE payment_transactions SET crypto_type = 'USDT_BEP20' WHERE crypto_type = 'USDT-BEP20';
