-- Migration to allow nullable fields in payment_transactions for multi-currency selection
-- This allows creating a payment request in USD without fixing the cryptocurrency yet

ALTER TABLE payment_transactions ALTER COLUMN amount DROP NOT NULL;

ALTER TABLE payment_transactions
ALTER COLUMN crypto_type
DROP NOT NULL;

ALTER TABLE payment_transactions ALTER COLUMN network DROP NOT NULL;

ALTER TABLE payment_transactions
ALTER COLUMN to_address
DROP NOT NULL;

ALTER TABLE payment_transactions
ALTER COLUMN fee_amount
DROP NOT NULL;

-- Also update the constraint for positive amount to only apply when amount is not null
ALTER TABLE payment_transactions DROP CONSTRAINT chk_amount_positive;

ALTER TABLE payment_transactions
ADD CONSTRAINT chk_amount_positive CHECK (
    (
        amount IS NULL
        OR (amount > 0)
    )
    AND (amount_usd > 0)
);

-- Update chk_total_paid_valid to handle NULL amount
ALTER TABLE payment_transactions
DROP CONSTRAINT chk_total_paid_valid;

ALTER TABLE payment_transactions
ADD CONSTRAINT chk_total_paid_valid CHECK (
    (amount IS NULL)
    OR (
        total_paid >= 0
        AND total_paid <= amount
    )
);