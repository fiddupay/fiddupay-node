-- Add amount_usd column to customer_transactions table for USD value tracking

ALTER TABLE customer_transactions 
ADD COLUMN IF NOT EXISTS amount_usd DECIMAL(20,2) DEFAULT 0;

-- Add comment for clarity
COMMENT ON COLUMN customer_transactions.amount_usd IS 'Amount in USD at the time of transaction';

-- Optional: Add index if we need to filter by USD amount
CREATE INDEX IF NOT EXISTS idx_customer_transactions_amount_usd ON customer_transactions(amount_usd);
