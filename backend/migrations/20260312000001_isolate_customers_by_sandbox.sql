-- Migration: Isolate merchant customers by sandbox_mode
-- This ensures that sub-accounts and their wallets/balances are separated between Live and Sandbox

-- 1. Update merchant_customers
ALTER TABLE merchant_customers 
ADD COLUMN IF NOT EXISTS sandbox_mode BOOLEAN NOT NULL DEFAULT false;

-- Drop old unique constraint
ALTER TABLE merchant_customers 
DROP CONSTRAINT IF EXISTS merchant_customers_merchant_id_external_id_key;

-- Add new unique constraint including sandbox_mode
ALTER TABLE merchant_customers 
ADD CONSTRAINT merchant_customers_unique_merchant_external_sandbox UNIQUE (merchant_id, external_id, sandbox_mode);

-- Create index
CREATE INDEX IF NOT EXISTS idx_merchant_customers_sandbox ON merchant_customers (sandbox_mode);


-- 2. Update merchant_customer_wallets
ALTER TABLE merchant_customer_wallets 
ADD COLUMN IF NOT EXISTS sandbox_mode BOOLEAN NOT NULL DEFAULT false;

-- Drop old unique constraint
ALTER TABLE merchant_customer_wallets 
DROP CONSTRAINT IF EXISTS merchant_customer_wallets_customer_id_crypto_type_key;

-- Add new unique constraint
ALTER TABLE merchant_customer_wallets 
ADD CONSTRAINT merchant_cust_wallets_unique_cust_crypto_sandbox UNIQUE (customer_id, crypto_type, sandbox_mode);

-- Create index
CREATE INDEX IF NOT EXISTS idx_merchant_cust_wallets_sandbox ON merchant_customer_wallets (sandbox_mode);


-- 3. Update merchant_customer_balances
ALTER TABLE merchant_customer_balances 
ADD COLUMN IF NOT EXISTS sandbox_mode BOOLEAN NOT NULL DEFAULT false;

-- Drop old unique constraint
ALTER TABLE merchant_customer_balances 
DROP CONSTRAINT IF EXISTS merchant_customer_balances_customer_id_crypto_type_key;

-- Add new unique constraint
ALTER TABLE merchant_customer_balances 
ADD CONSTRAINT merchant_cust_balances_unique_cust_crypto_sandbox UNIQUE (customer_id, crypto_type, sandbox_mode);

-- Create index
CREATE INDEX IF NOT EXISTS idx_merchant_cust_balances_sandbox ON merchant_customer_balances (sandbox_mode);


-- 4. Update customer_transactions
ALTER TABLE customer_transactions 
ADD COLUMN IF NOT EXISTS sandbox_mode BOOLEAN NOT NULL DEFAULT false;

-- Create index
CREATE INDEX IF NOT EXISTS idx_customer_transactions_sandbox ON customer_transactions (sandbox_mode);

-- 5. Comments
COMMENT ON COLUMN merchant_customers.sandbox_mode IS 'Isolation: true for sandbox environment, false for production';
COMMENT ON COLUMN merchant_customer_wallets.sandbox_mode IS 'Isolation: true for sandbox environment, false for production';
COMMENT ON COLUMN merchant_customer_balances.sandbox_mode IS 'Isolation: true for sandbox environment, false for production';
COMMENT ON COLUMN customer_transactions.sandbox_mode IS 'Isolation: true for sandbox environment, false for production';
