-- Optimized index for blockchain monitor payment verification
-- This query was identified as a slow statement (> 1s) in production logs
-- db.statement="SELECT id, merchant_id FROM payment_transactions WHERE LOWER(to_address) = $1 AND status IN ('PENDING', 'CONFIRMING')"

CREATE INDEX IF NOT EXISTS idx_payment_transactions_lower_address_active
ON payment_transactions (LOWER(to_address))
WHERE status IN ('PENDING', 'CONFIRMING');
