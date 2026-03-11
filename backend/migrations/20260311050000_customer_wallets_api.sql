-- Migration for Customer Wallet API
-- Adds status, permissions, and the customer_transactions ledger table

-- 1. Add status and permission fields to merchant_customers
ALTER TABLE merchant_customers 
  ADD COLUMN IF NOT EXISTS status VARCHAR(20) DEFAULT 'active',
  ADD COLUMN IF NOT EXISTS status_reason TEXT,
  ADD COLUMN IF NOT EXISTS can_withdraw BOOLEAN DEFAULT TRUE,
  ADD COLUMN IF NOT EXISTS withdrawal_limit NUMERIC;

-- 2. Backfill existing customers to active status
UPDATE merchant_customers SET status = 'active' WHERE status IS NULL;
UPDATE merchant_customers SET can_withdraw = TRUE WHERE can_withdraw IS NULL;

-- 3. Create the customer transactions ledger
CREATE TABLE IF NOT EXISTS customer_transactions (
  id BIGSERIAL PRIMARY KEY,
  customer_id BIGINT NOT NULL REFERENCES merchant_customers(id) ON DELETE CASCADE,
  merchant_id BIGINT NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
  type VARCHAR(30) NOT NULL, -- WITHDRAWAL, MERCHANT_PAYMENT, SWEEP
  crypto_type VARCHAR(30) NOT NULL,
  amount NUMERIC NOT NULL,
  fee NUMERIC DEFAULT 0,
  status VARCHAR(20) DEFAULT 'PENDING',
  destination_address TEXT,
  transaction_hash TEXT,
  reference_id TEXT,
  description TEXT,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 4. Create indexes for quick querying
CREATE INDEX IF NOT EXISTS idx_customer_transactions_customer_id ON customer_transactions(customer_id);
CREATE INDEX IF NOT EXISTS idx_customer_transactions_merchant_id ON customer_transactions(merchant_id);
CREATE INDEX IF NOT EXISTS idx_customer_transactions_crypto_type ON customer_transactions(crypto_type);
CREATE INDEX IF NOT EXISTS idx_customer_transactions_created_at ON customer_transactions(created_at);
