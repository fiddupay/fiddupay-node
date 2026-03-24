-- Remove Transaction PIN fields from merchant_customers
-- These fields are now only applicable to merchants.
ALTER TABLE merchant_customers 
DROP COLUMN IF EXISTS transaction_pin_hash,
DROP COLUMN IF EXISTS pin_setup_at;
