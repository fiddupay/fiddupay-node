-- Migration for Smart Fee Sweeping

-- Add fee_collected flag to payment_transactions to track if the platform fee has been swept
ALTER TABLE payment_transactions ADD COLUMN IF NOT EXISTS fee_collected BOOLEAN DEFAULT FALSE;

-- Create fee_sweep_settings table (Global settings for Super Admin)
CREATE TABLE IF NOT EXISTS fee_sweep_settings (
    id SERIAL PRIMARY KEY,
    is_auto_sweep_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    min_accumulated_usd DECIMAL(20, 8),           -- Threshold in USD equivalent
    schedule_cron VARCHAR(100),                   -- e.g., '0 2 * * 0' for Sunday at 2 AM
    discord_webhook_url VARCHAR(255),             -- Webhook for low gas alerts
    gas_alert_threshold_gwei DECIMAL(20, 8),      -- EVM networks low gas threshold
    gas_alert_threshold_lamports BIGINT,          -- Solana low gas threshold
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Insert a default settings row
INSERT INTO fee_sweep_settings (is_auto_sweep_enabled) 
SELECT FALSE 
WHERE NOT EXISTS (SELECT 1 FROM fee_sweep_settings);

-- Create gas_history table to track gas prices on different networks
CREATE TABLE IF NOT EXISTS gas_history (
    id SERIAL PRIMARY KEY,
    network VARCHAR(50) NOT NULL,                 -- e.g., 'ETHEREUM', 'POLYGON', 'SOLANA'
    base_fee_gwei DECIMAL(20, 8),
    base_fee_lamports BIGINT,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_gas_history_network_time ON gas_history(network, recorded_at DESC);

-- Allow fee_collections to track batch sweeps instead of individual payments
ALTER TABLE fee_collections ALTER COLUMN payment_id DROP NOT NULL;
