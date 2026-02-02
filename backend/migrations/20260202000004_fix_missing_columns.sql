-- Add missing columns to merchants and payment_transactions
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS kyc_verified BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE payment_transactions ADD COLUMN IF NOT EXISTS webhook_url VARCHAR(500);
