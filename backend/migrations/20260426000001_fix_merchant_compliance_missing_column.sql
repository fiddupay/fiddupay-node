-- Migration: Add missing business_license_update_count column
-- This column was added to models but missed in previous migrations, causing Auth failures.

ALTER TABLE merchants ADD COLUMN IF NOT EXISTS business_license_update_count INTEGER NOT NULL DEFAULT 0;
