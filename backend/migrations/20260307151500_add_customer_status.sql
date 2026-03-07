-- Migration: Add is_active status to merchant_customers
-- Allows for deactivation instead of permanent deletion

ALTER TABLE merchant_customers
ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT TRUE;

COMMENT ON COLUMN merchant_customers.is_active IS 'Indicates if the customer is active or deactivated. Deactivated customers cannot perform new operations.';
