-- Change default for customer_pays_fee to false (Merchant pays fee)
ALTER TABLE merchants ALTER COLUMN customer_pays_fee SET DEFAULT false;

-- Update existing records to match the new preference if desired, 
-- or leave them as is. Let's update all to ensure consistency with the new policy.
UPDATE merchants SET customer_pays_fee = false;
