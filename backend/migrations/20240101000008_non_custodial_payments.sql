-- Add non-custodial payment support
-- Merchants control their own wallets

-- Add column to track if payment goes directly to merchant wallet
ALTER TABLE payment_transactions 
ADD COLUMN is_non_custodial BOOLEAN NOT NULL DEFAULT true;

-- Add index for non-custodial payments
CREATE INDEX IF NOT EXISTS idx_payment_transactions_non_custodial 
ON payment_transactions(is_non_custodial, status);

-- Update existing payments to be non-custodial
UPDATE payment_transactions SET is_non_custodial = true WHERE is_non_custodial IS NULL;

-- Add comment
COMMENT ON COLUMN payment_transactions.is_non_custodial IS 'True if payment goes directly to merchant wallet (non-custodial)';
