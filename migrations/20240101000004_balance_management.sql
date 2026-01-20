-- Create merchant_balances table
CREATE TABLE merchant_balances (
    id SERIAL PRIMARY KEY,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id),
    crypto_type VARCHAR(50) NOT NULL,
    available_balance DECIMAL(36, 18) NOT NULL DEFAULT 0,
    reserved_balance DECIMAL(36, 18) NOT NULL DEFAULT 0,
    total_balance DECIMAL(36, 18) GENERATED ALWAYS AS (available_balance + reserved_balance) STORED,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(merchant_id, crypto_type)
);

-- Create balance_history table
CREATE TABLE balance_history (
    id SERIAL PRIMARY KEY,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id),
    crypto_type VARCHAR(50) NOT NULL,
    amount DECIMAL(36, 18) NOT NULL,
    balance_type VARCHAR(20) NOT NULL, -- 'AVAILABLE' or 'RESERVED'
    change_type VARCHAR(20) NOT NULL, -- 'CREDIT' or 'DEBIT'
    reason VARCHAR(100) NOT NULL, -- 'PAYMENT_CONFIRMED', 'REFUND', 'WITHDRAWAL', etc.
    reference_id VARCHAR(100), -- payment_id, refund_id, withdrawal_id
    balance_before DECIMAL(36, 18) NOT NULL,
    balance_after DECIMAL(36, 18) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_merchant_balances_merchant ON merchant_balances(merchant_id);
CREATE INDEX idx_balance_history_merchant ON balance_history(merchant_id);
CREATE INDEX idx_balance_history_created ON balance_history(created_at);
CREATE INDEX idx_balance_history_reference ON balance_history(reference_id);
