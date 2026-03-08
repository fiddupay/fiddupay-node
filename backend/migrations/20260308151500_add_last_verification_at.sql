-- Add last_verification_at to payment_transactions to support verification cooldowns
ALTER TABLE payment_transactions ADD COLUMN last_verification_at TIMESTAMPTZ;
