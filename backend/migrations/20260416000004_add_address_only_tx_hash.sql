-- Add last_tx_hash to address_only_payments for better auditability
ALTER TABLE address_only_payments ADD COLUMN last_tx_hash TEXT;
