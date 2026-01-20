-- Test Fixtures for PayFlow Integration Tests

-- Insert test merchant
INSERT INTO merchants (business_name, email, api_key_hash, fee_percentage, is_active, sandbox_mode)
VALUES 
  ('Test Merchant 1', 'test1@example.com', '$argon2id$v=19$m=19456,t=2,p=1$test_salt$test_hash', 1.50, true, true),
  ('Test Merchant 2', 'test2@example.com', '$argon2id$v=19$m=19456,t=2,p=1$test_salt2$test_hash2', 2.00, true, false);

-- Get merchant IDs
DO $$
DECLARE
  merchant1_id BIGINT;
  merchant2_id BIGINT;
BEGIN
  SELECT id INTO merchant1_id FROM merchants WHERE email = 'test1@example.com';
  SELECT id INTO merchant2_id FROM merchants WHERE email = 'test2@example.com';

  -- Insert merchant wallets
  INSERT INTO merchant_wallets (merchant_id, crypto_type, wallet_address, network)
  VALUES 
    (merchant1_id, 'SOL', 'TestSolanaAddress1111111111111111111', 'SOLANA'),
    (merchant1_id, 'USDT_BEP20', '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0', 'BEP20'),
    (merchant2_id, 'SOL', 'TestSolanaAddress2222222222222222222', 'SOLANA');

  -- Insert test payments
  INSERT INTO payment_transactions (
    payment_id, merchant_id, amount, amount_usd, crypto_type, network,
    status, customer_email, fee_percentage, fee_amount, fee_amount_usd
  )
  VALUES 
    ('PAY-TEST-001', merchant1_id, 1.5, 150.00, 'SOL', 'SOLANA', 'PENDING', 'customer1@test.com', 1.50, 0.0225, 2.25),
    ('PAY-TEST-002', merchant1_id, 100.0, 100.00, 'USDT_BEP20', 'BEP20', 'CONFIRMED', 'customer2@test.com', 1.50, 1.5, 1.50),
    ('PAY-TEST-003', merchant2_id, 2.0, 200.00, 'SOL', 'SOLANA', 'EXPIRED', 'customer3@test.com', 2.00, 0.04, 4.00);

  -- Insert merchant balances
  INSERT INTO merchant_balances (merchant_id, crypto_type, available_balance, reserved_balance)
  VALUES 
    (merchant1_id, 'SOL', 10.5, 1.5),
    (merchant1_id, 'USDT_BEP20', 500.0, 0.0),
    (merchant2_id, 'SOL', 5.0, 0.0);

END $$;

-- Verify data
SELECT 'Merchants:', COUNT(*) FROM merchants;
SELECT 'Wallets:', COUNT(*) FROM merchant_wallets;
SELECT 'Payments:', COUNT(*) FROM payment_transactions;
SELECT 'Balances:', COUNT(*) FROM merchant_balances;
