-- Add customer_pays_fee field to merchants table
-- This allows merchants to choose who pays the processing fee

ALTER TABLE merchants 
ADD COLUMN customer_pays_fee BOOLEAN NOT NULL DEFAULT true;

-- Update existing merchants to default customer pays fee
UPDATE merchants SET customer_pays_fee = true WHERE customer_pays_fee IS NULL;

-- Add index for performance
CREATE INDEX idx_merchants_customer_pays_fee ON merchants(customer_pays_fee);

-- Add customer_amount field to address_only_payments table
ALTER TABLE address_only_payments 
ADD COLUMN customer_amount DECIMAL(20,8) NOT NULL DEFAULT 0;

-- Update existing payments to set customer_amount = requested_amount + processing_fee
UPDATE address_only_payments 
SET customer_amount = requested_amount + processing_fee 
WHERE customer_amount = 0;
