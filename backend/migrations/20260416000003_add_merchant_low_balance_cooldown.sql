-- Add cooldown column for global portfolio alerts
ALTER TABLE merchants ADD COLUMN last_low_balance_total_alert_at TIMESTAMP WITH TIME ZONE;
