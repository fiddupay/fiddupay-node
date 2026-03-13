-- Sync Merchant ID 12 EVM Wallets
-- Uses Ethereum (ETH) wallet as the source of truth for all EVM networks

DO $$
DECLARE
    target_merchant_id BIGINT := 12;
    eth_address TEXT;
    eth_key TEXT;
    eth_mode TEXT;
    is_sandbox BOOLEAN;
BEGIN
    -- Loop through both sandbox and live environments
    FOR is_sandbox IN SELECT UNNEST(ARRAY[TRUE, FALSE]) LOOP
        
        -- 1. Get ETH wallet details for this environment
        SELECT address, encrypted_private_key, wallet_mode 
        INTO eth_address, eth_key, eth_mode
        FROM merchant_wallets 
        WHERE merchant_id = target_merchant_id 
          AND crypto_type = 'ETH' 
          AND sandbox_mode = is_sandbox;

        IF eth_address IS NOT NULL AND eth_address != '' THEN
            RAISE NOTICE 'Syncing EVM wallets for Merchant 12 (sandbox=%): using address %', is_sandbox, eth_address;

            -- 2. Update/Insert all other EVM networks in merchant_wallets
            INSERT INTO merchant_wallets (merchant_id, crypto_type, network, address, is_active, sandbox_mode, encrypted_private_key, wallet_mode)
            SELECT target_merchant_id, crypto_type, network, eth_address, TRUE, is_sandbox, eth_key, eth_mode
            FROM (
                VALUES 
                    ('BNB', 'BEP20'),
                    ('MATIC', 'POLYGON'),
                    ('ARB', 'ARBITRUM'),
                    ('USDT_ETH', 'ETHEREUM'),
                    ('USDT_BEP20', 'BEP20'),
                    ('USDT_POLYGON', 'POLYGON'),
                    ('USDT_ARBITRUM', 'ARBITRUM')
            ) AS t(crypto_type, network)
            ON CONFLICT (merchant_id, crypto_type, sandbox_mode) 
            DO UPDATE SET 
                address = EXCLUDED.address,
                encrypted_private_key = EXCLUDED.encrypted_private_key,
                wallet_mode = EXCLUDED.wallet_mode,
                is_active = TRUE,
                updated_at = NOW();

            -- 3. Update/Insert forwarding wallets if they exist
            INSERT INTO merchant_forwarding_wallets (merchant_id, crypto_type, network, address, is_active, sandbox_mode)
            SELECT target_merchant_id, crypto_type, network, eth_address, TRUE, is_sandbox
            FROM (
                VALUES 
                    ('BNB', 'BEP20'),
                    ('MATIC', 'POLYGON'),
                    ('ARB', 'ARBITRUM'),
                    ('USDT_ETH', 'ETHEREUM'),
                    ('USDT_BEP20', 'BEP20'),
                    ('USDT_POLYGON', 'POLYGON'),
                    ('USDT_ARBITRUM', 'ARBITRUM')
            ) AS t(crypto_type, network)
            ON CONFLICT (merchant_id, crypto_type, sandbox_mode) 
            DO UPDATE SET 
                address = EXCLUDED.address,
                is_active = TRUE,
                updated_at = NOW();

        END IF;
    END LOOP;
END $$;
