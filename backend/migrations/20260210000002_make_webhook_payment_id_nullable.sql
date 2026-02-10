-- Make payment_id nullable in webhook_deliveries
-- This is required to support test webhooks that don't have a corresponding payment record
ALTER TABLE webhook_deliveries ALTER COLUMN payment_id DROP NOT NULL;

-- Remove the foreign key constraint that requires a valid payment_id
-- We first need to find the constraint name, which is standard SQL:
-- ALTER TABLE webhook_deliveries DROP CONSTRAINT webhook_deliveries_payment_id_fkey;
-- However, since it was defined with "payment_id BIGINT NOT NULL REFERENCES payment_transactions(id) ON DELETE CASCADE",
-- PostgreSQL usually names it "webhook_deliveries_payment_id_fkey".

ALTER TABLE webhook_deliveries
DROP CONSTRAINT IF EXISTS webhook_deliveries_payment_id_fkey;

-- Add it back as an optional reference
ALTER TABLE webhook_deliveries
ADD CONSTRAINT webhook_deliveries_payment_id_fkey FOREIGN KEY (payment_id) REFERENCES payment_transactions (id) ON DELETE CASCADE;