-- Add customer_pays_fee column to merchants table
ALTER TABLE merchants ADD COLUMN customer_pays_fee BOOLEAN NOT NULL DEFAULT true;

-- Add customer_amount column to address_only_payments table
ALTER TABLE address_only_payments ADD COLUMN customer_amount DECIMAL(20,8);

-- Add comment
COMMENT ON COLUMN merchants.customer_pays_fee IS 'Whether customer pays the fee (true) or merchant absorbs it (false)';
COMMENT ON COLUMN address_only_payments.customer_amount IS 'Amount customer pays including fees';
