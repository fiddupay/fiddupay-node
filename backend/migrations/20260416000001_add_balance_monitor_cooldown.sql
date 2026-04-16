-- Add last_low_balance_alert_at to merchant_wallets for notification cooldown tracking
ALTER TABLE merchant_wallets ADD COLUMN last_low_balance_alert_at TIMESTAMPTZ;

-- Index to optimize fetching wallets that haven't been notified recently
CREATE INDEX idx_merchant_wallets_last_alert ON merchant_wallets(last_low_balance_alert_at);
