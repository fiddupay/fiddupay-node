-- Add sandbox_mode column to payment_transactions
ALTER TABLE payment_transactions
ADD COLUMN IF NOT EXISTS sandbox_mode BOOLEAN NOT NULL DEFAULT false;

-- Add sandbox_mode column to refunds
ALTER TABLE refunds
ADD COLUMN IF NOT EXISTS sandbox_mode BOOLEAN NOT NULL DEFAULT false;

-- Add sandbox_mode column to withdrawals
ALTER TABLE withdrawals
ADD COLUMN IF NOT EXISTS sandbox_mode BOOLEAN NOT NULL DEFAULT false;

-- Add sandbox_mode column to merchant_balances
ALTER TABLE merchant_balances
ADD COLUMN IF NOT EXISTS sandbox_mode BOOLEAN NOT NULL DEFAULT false;

-- Add sandbox_mode column to balance_history
ALTER TABLE balance_history
ADD COLUMN IF NOT EXISTS sandbox_mode BOOLEAN NOT NULL DEFAULT false;

-- Update unique constraint on merchant_balances to include sandbox_mode
-- First drop the old unique constraint
ALTER TABLE merchant_balances
DROP CONSTRAINT IF EXISTS merchant_balances_merchant_id_crypto_type_key;

-- Then add the new one
ALTER TABLE merchant_balances
ADD CONSTRAINT merchant_balances_unique_merchant_crypto_sandbox UNIQUE (
    merchant_id,
    crypto_type,
    sandbox_mode
);

-- Add indexes for filtering
CREATE INDEX IF NOT EXISTS idx_payment_transactions_sandbox ON payment_transactions (sandbox_mode);

CREATE INDEX IF NOT EXISTS idx_refunds_sandbox ON refunds (sandbox_mode);

CREATE INDEX IF NOT EXISTS idx_withdrawals_sandbox ON withdrawals (sandbox_mode);

CREATE INDEX IF NOT EXISTS idx_merchant_balances_sandbox ON merchant_balances (sandbox_mode);