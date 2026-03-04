-- Migration: Add KYC fields to merchants and p2p_profiles
-- This collects detailed information for regulatory compliance

-- 1. Update merchants table
ALTER TABLE merchants 
ADD COLUMN first_name VARCHAR(100),
ADD COLUMN last_name VARCHAR(100),
ADD COLUMN gender VARCHAR(20),
ADD COLUMN phone_number VARCHAR(30),
ADD COLUMN country VARCHAR(100),
ADD COLUMN applicant_role VARCHAR(100),
ADD COLUMN business_country VARCHAR(100),
ADD COLUMN business_license_number VARCHAR(100),
ADD COLUMN business_certificate_url VARCHAR(500),
ADD COLUMN terms_accepted BOOLEAN NOT NULL DEFAULT false;

-- Add comments for documentation
COMMENT ON COLUMN merchants.first_name IS 'Personal first name of the applicant';
COMMENT ON COLUMN merchants.last_name IS 'Personal last name of the applicant';
COMMENT ON COLUMN merchants.applicant_role IS 'Role of the person registering (e.g., Founder, CTO)';
COMMENT ON COLUMN merchants.business_certificate_url IS 'URL to the uploaded business incorporation/CAC certificate';

-- 2. Update p2p_profiles table
ALTER TABLE p2p_profiles
ADD COLUMN first_name VARCHAR(100),
ADD COLUMN last_name VARCHAR(100),
ADD COLUMN gender VARCHAR(20),
ADD COLUMN phone_number VARCHAR(30),
ADD COLUMN country VARCHAR(100),
ADD COLUMN terms_accepted BOOLEAN NOT NULL DEFAULT false;

-- Add comments for documentation
COMMENT ON COLUMN p2p_profiles.first_name IS 'Legal first name of the P2P user';
COMMENT ON COLUMN p2p_profiles.last_name IS 'Legal last name of the P2P user';
