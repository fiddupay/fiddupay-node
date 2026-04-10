-- Add public publishable keys to merchants table to support zero-code widgets
ALTER TABLE merchants
ADD COLUMN live_publishable_key VARCHAR(100),
ADD COLUMN test_publishable_key VARCHAR(100);

-- Create unique indexes for extremely fast lookup during public API requests
CREATE UNIQUE INDEX idx_merchants_live_publishable_key ON merchants(live_publishable_key) WHERE live_publishable_key IS NOT NULL;
CREATE UNIQUE INDEX idx_merchants_test_publishable_key ON merchants(test_publishable_key) WHERE test_publishable_key IS NOT NULL;
