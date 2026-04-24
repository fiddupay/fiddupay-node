-- Migration: Ensure Merchant Compliance & Identity Columns
-- Fixes ColumnNotFound errors in authentication by ensuring all required fields exist

ALTER TABLE merchants
ADD COLUMN IF NOT EXISTS nin_bvn_hash VARCHAR(128),
ADD COLUMN IF NOT EXISTS social_handles JSONB DEFAULT '{}'::jsonb,
ADD COLUMN IF NOT EXISTS kyc_tier INTEGER NOT NULL DEFAULT 0,
ADD COLUMN IF NOT EXISTS compliance_status VARCHAR(20) NOT NULL DEFAULT 'PENDING',
ADD COLUMN IF NOT EXISTS username VARCHAR(100),
ADD COLUMN IF NOT EXISTS pay_id VARCHAR(50);

-- Ensure UNIQUE constraints only if they don't already exist
DO $$ 
BEGIN 
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'merchants_username_key') THEN
        ALTER TABLE merchants ADD CONSTRAINT merchants_username_key UNIQUE (username);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'merchants_pay_id_key') THEN
        ALTER TABLE merchants ADD CONSTRAINT merchants_pay_id_key UNIQUE (pay_id);
    END IF;
END $$;

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_merchants_nin_bvn_hash ON merchants(nin_bvn_hash);
CREATE INDEX IF NOT EXISTS idx_merchants_pay_id ON merchants(pay_id);
