-- Migration: Add redirect_url to merchants table
-- This allows merchants to specify where customers are sent after a successful checkout.

ALTER TABLE merchants ADD COLUMN redirect_url VARCHAR(500);

COMMENT ON COLUMN merchants.redirect_url IS 'External URL to redirect customers to after payment is CONFIRMED';