-- Add payload_format to webhook_configs
ALTER TABLE webhook_configs
ADD COLUMN payload_format VARCHAR(50) NOT NULL DEFAULT 'standard';

COMMENT ON COLUMN webhook_configs.payload_format IS 'Formatting for webhook payload: "standard" or "discord"';