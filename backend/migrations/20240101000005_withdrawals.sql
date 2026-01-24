-- Create withdrawals table
CREATE TABLE withdrawals (
    id SERIAL PRIMARY KEY,
    withdrawal_id VARCHAR(100) UNIQUE NOT NULL,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id),
    crypto_type VARCHAR(50) NOT NULL,
    amount DECIMAL(36, 18) NOT NULL,
    destination_address VARCHAR(255) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'PENDING', -- PENDING, APPROVED, PROCESSING, COMPLETED, REJECTED, CANCELLED
    fee DECIMAL(36, 18) NOT NULL DEFAULT 0,
    net_amount DECIMAL(36, 18) NOT NULL,
    transaction_hash VARCHAR(255),
    rejection_reason TEXT,
    requires_approval BOOLEAN NOT NULL DEFAULT false,
    approved_by INTEGER REFERENCES merchants(id),
    approved_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_withdrawals_merchant ON withdrawals(merchant_id);
CREATE INDEX idx_withdrawals_status ON withdrawals(status);
CREATE INDEX idx_withdrawals_created ON withdrawals(created_at);
CREATE INDEX idx_withdrawals_withdrawal_id ON withdrawals(withdrawal_id);
