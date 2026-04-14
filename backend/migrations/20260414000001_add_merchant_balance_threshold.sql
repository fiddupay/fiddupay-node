-- Add low_balance_threshold_usd to merchants
ALTER TABLE merchants ADD COLUMN low_balance_threshold_usd DECIMAL(20, 8) DEFAULT 0;

-- Index to optimize background monitoring tasks that fetch merchants with active thresholds
CREATE INDEX idx_merchants_threshold ON merchants(low_balance_threshold_usd) WHERE low_balance_threshold_usd > 0;
