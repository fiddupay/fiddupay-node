-- Add from_address to payment_transactions for refund auditing
ALTER TABLE payment_transactions ADD COLUMN IF NOT EXISTS from_address VARCHAR(255);

-- Add daily_limit_usd to merchants (default 1000.00, overrides system default if set)
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS daily_limit_usd DECIMAL(20,2);
