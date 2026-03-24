-- Normalize network naming across all wallet tables
-- Standardizes inconsistent names (e.g., 'SOLANA_SPL', 'Solana (SPL)', 'BSC', 'Binance Smart Chain') 
-- to canonical UPPERCASE versions used by the backend.

-- 1. Normalize merchant_wallets
UPDATE merchant_wallets 
SET network = 'SOLANA' 
WHERE network IN ('SOLANA_SPL', 'Solana (SPL)', 'Solana', 'solana', 'SPL');

UPDATE merchant_wallets 
SET network = 'ETHEREUM' 
WHERE network IN ('Ethereum', 'ethereum', 'Ethereum (ERC-20)', 'ERC20');

UPDATE merchant_wallets 
SET network = 'BEP20' 
WHERE network IN ('Binance Smart Chain', 'Binance Smart Chain (BEP-20)', 'BSC', 'bsc', 'BEP-20');

UPDATE merchant_wallets 
SET network = 'POLYGON' 
WHERE network IN ('Polygon', 'polygon', 'Polygon (MATIC)', 'MATIC');

UPDATE merchant_wallets 
SET network = 'ARBITRUM' 
WHERE network IN ('Arbitrum', 'arbitrum', 'Arbitrum One', 'ARB');

UPDATE merchant_wallets 
SET network = 'BITCOIN' 
WHERE network IN ('Bitcoin', 'bitcoin', 'BTC');

-- 2. Normalize merchant_forwarding_wallets
UPDATE merchant_forwarding_wallets 
SET network = 'SOLANA' 
WHERE network IN ('SOLANA_SPL', 'Solana (SPL)', 'Solana', 'solana', 'SPL');

UPDATE merchant_forwarding_wallets 
SET network = 'ETHEREUM' 
WHERE network IN ('Ethereum', 'ethereum', 'Ethereum (ERC-20)', 'ERC20');

UPDATE merchant_forwarding_wallets 
SET network = 'BEP20' 
WHERE network IN ('Binance Smart Chain', 'Binance Smart Chain (BEP-20)', 'BSC', 'bsc', 'BEP-20');

UPDATE merchant_forwarding_wallets 
SET network = 'POLYGON' 
WHERE network IN ('Polygon', 'polygon', 'Polygon (MATIC)', 'MATIC');

UPDATE merchant_forwarding_wallets 
SET network = 'ARBITRUM' 
WHERE network IN ('Arbitrum', 'arbitrum', 'Arbitrum One', 'ARB');

UPDATE merchant_forwarding_wallets 
SET network = 'BITCOIN' 
WHERE network IN ('Bitcoin', 'bitcoin', 'BTC');

-- 3. Normalize merchant_customer_wallets
UPDATE merchant_customer_wallets 
SET network = 'SOLANA' 
WHERE network IN ('SOLANA_SPL', 'Solana (SPL)', 'Solana', 'solana', 'SPL');

UPDATE merchant_customer_wallets 
SET network = 'ETHEREUM' 
WHERE network IN ('Ethereum', 'ethereum', 'Ethereum (ERC-20)', 'ERC20');

UPDATE merchant_customer_wallets 
SET network = 'BEP20' 
WHERE network IN ('Binance Smart Chain', 'Binance Smart Chain (BEP-20)', 'BSC', 'bsc', 'BEP-20');

UPDATE merchant_customer_wallets 
SET network = 'POLYGON' 
WHERE network IN ('Polygon', 'polygon', 'Polygon (MATIC)', 'MATIC');

UPDATE merchant_customer_wallets 
SET network = 'ARBITRUM' 
WHERE network IN ('Arbitrum', 'arbitrum', 'Arbitrum One', 'ARB');

UPDATE merchant_customer_wallets 
SET network = 'BITCOIN' 
WHERE network IN ('Bitcoin', 'bitcoin', 'BTC');
