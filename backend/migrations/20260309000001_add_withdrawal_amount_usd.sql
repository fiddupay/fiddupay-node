-- Add amount_usd column to withdrawals table
ALTER TABLE withdrawals
ADD COLUMN amount_usd DECIMAL(20,2) NOT NULL DEFAULT 0;

-- Comment for documentation
COMMENT ON COLUMN withdrawals.amount_usd IS 'Amount of the withdrawal in USD at the time of request';
