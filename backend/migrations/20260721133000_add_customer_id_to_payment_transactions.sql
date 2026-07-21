-- Add customer_id to payment_transactions to explicitly link checkout payments to merchant_customers
ALTER TABLE payment_transactions 
ADD COLUMN IF NOT EXISTS customer_id BIGINT REFERENCES merchant_customers(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_payment_transactions_customer_id 
ON payment_transactions(customer_id) 
WHERE customer_id IS NOT NULL;
