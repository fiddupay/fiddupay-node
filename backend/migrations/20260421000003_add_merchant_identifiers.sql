-- Migration: Add Merchant Identifiers for Ecosystem Interoperability
-- Enables universal search/pay via Username and PayID

ALTER TABLE merchants 
ADD COLUMN IF NOT EXISTS username VARCHAR(50) UNIQUE,
ADD COLUMN IF NOT EXISTS pay_id VARCHAR(20) UNIQUE;

-- Create indexes for fast resolution in the Hybrid Resolver
CREATE INDEX IF NOT EXISTS idx_merchants_username ON merchants(username);
CREATE INDEX IF NOT EXISTS idx_merchants_pay_id ON merchants(pay_id);

-- Documentation for the columns
COMMENT ON COLUMN merchants.username IS 'Merchant-claimed unique handle for P2P/Checkouts (@username)';
COMMENT ON COLUMN merchants.pay_id IS 'Unique platform identifier (FID-XXXX-XXXX)';
