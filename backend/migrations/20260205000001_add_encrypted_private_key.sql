-- Add encrypted_private_key to merchant_wallets
ALTER TABLE merchant_wallets
ADD COLUMN IF NOT EXISTS encrypted_private_key TEXT;

COMMENT ON COLUMN merchant_wallets.encrypted_private_key IS 'AES-256-GCM encrypted private key for imported wallets';