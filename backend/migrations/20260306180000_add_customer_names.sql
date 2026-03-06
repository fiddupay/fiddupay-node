-- Migration: Add first_name and last_name to merchant_customers
-- Enhance user profiling for sub-account management

ALTER TABLE merchant_customers
ADD COLUMN first_name VARCHAR(100),
ADD COLUMN last_name VARCHAR(100);

-- Update existing records metadata to move name fields if they exist (optional but good practice)
-- Since it's a new feature, we assume new records will use these columns.
