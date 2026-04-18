-- Migration: Make merchant customers global across environments
-- This allows a merchant to reuse the same customer profile in both Live and Sandbox

-- 1. Identify and consolidate duplicate customer registrations
-- For any (merchant_id, external_id) that exists in both Live (sandbox_mode=false) and Sandbox (sandbox_mode=true),
-- we will redirect all Sandbox-linked data (wallets, balances, transactions) to the Live profile ID,
-- then delete the redundant Sandbox profile.

DO $$ 
DECLARE 
    rec RECORD;
BEGIN
    FOR rec IN (
        -- Find customers that exist in both live and sandbox
        SELECT live.id as live_id, sand.id as sand_id, live.merchant_id, live.external_id
        FROM merchant_customers live
        JOIN merchant_customers sand ON live.merchant_id = sand.merchant_id 
            AND live.external_id = sand.external_id
        WHERE live.sandbox_mode = false AND sand.sandbox_mode = true
    ) LOOP
        -- Update wallets
        UPDATE merchant_customer_wallets SET customer_id = rec.live_id WHERE customer_id = rec.sand_id;
        -- Update balances
        UPDATE merchant_customer_balances SET customer_id = rec.live_id WHERE customer_id = rec.sand_id;
        -- Update transactions
        UPDATE customer_transactions SET customer_id = rec.live_id WHERE customer_id = rec.sand_id;
        -- Update history
        UPDATE merchant_wallet_history SET customer_id = rec.live_id WHERE customer_id = rec.sand_id;
        
        -- Delete the redundant sandbox customer
        DELETE FROM merchant_customers WHERE id = rec.sand_id;
    END LOOP;
END $$;

-- 2. Modify merchant_customers table
-- Drop unique constraint that includes sandbox_mode
ALTER TABLE merchant_customers 
DROP CONSTRAINT IF EXISTS merchant_customers_unique_merchant_external_sandbox;

-- Drop index
DROP INDEX IF EXISTS idx_merchant_customers_sandbox;

-- Drop the column
ALTER TABLE merchant_customers DROP COLUMN IF EXISTS sandbox_mode;

-- Add new unique constraint (merchant_id, external_id)
-- Using IF NOT EXISTS pattern is not directly supported for constraints in standard ALTER TABLE, 
-- but we already dropped the old one.
ALTER TABLE merchant_customers 
ADD CONSTRAINT merchant_customers_unique_merchant_external UNIQUE (merchant_id, external_id);

-- Update comments
COMMENT ON TABLE merchant_customers IS 'Merchant customer profiles (Shared across Live and Sandbox)';
