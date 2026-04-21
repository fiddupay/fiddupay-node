-- Migration: Add Merchant Compliance & Intelligence Fields
-- Adds NIN/BVN hashing and social handles for Nigerian regulatory alignment

ALTER TABLE merchants
ADD COLUMN IF NOT EXISTS nin_bvn_hash VARCHAR(128),
ADD COLUMN IF NOT EXISTS social_handles JSONB DEFAULT '{}'::jsonb,
ADD COLUMN IF NOT EXISTS kyc_tier INTEGER NOT NULL DEFAULT 0,
ADD COLUMN IF NOT EXISTS compliance_status VARCHAR(20) NOT NULL DEFAULT 'PENDING';

-- High-performance index for identifying duplicate ID users
CREATE INDEX IF NOT EXISTS idx_merchants_nin_bvn_hash ON merchants(nin_bvn_hash);

-- Role/Compliance commentary
COMMENT ON COLUMN merchants.nin_bvn_hash IS 'SHA-256 hash of NIN or BVN to prevent duplicate account creation without storing raw IDs';
COMMENT ON COLUMN merchants.social_handles IS 'JSON storage for Twitter and Instagram handles (e.g., {"twitter": "@user"})';
COMMENT ON COLUMN merchants.kyc_tier IS '0: Unverified, 1: ID Verified, 2: Business Verified';
