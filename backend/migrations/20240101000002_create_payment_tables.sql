-- Create payment_transactions table
-- Stores all payment requests and their status
CREATE TABLE payment_transactions (
    id BIGSERIAL PRIMARY KEY,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    payment_id VARCHAR(100) UNIQUE NOT NULL,  -- Public-facing ID (e.g., "pay_abc123")
    user_id BIGINT,  -- Optional: customer identifier from merchant's system
    subscription_id BIGINT,  -- Optional: subscription identifier
    
    -- Payment details
    description TEXT,
    metadata JSONB,
    
    -- Amount and fee information
    amount DECIMAL(20,8) NOT NULL,
    amount_usd DECIMAL(20,2) NOT NULL,
    fee_percentage DECIMAL(5,2) NOT NULL,
    fee_amount DECIMAL(20,8) NOT NULL,
    fee_amount_usd DECIMAL(20,2) NOT NULL,
    
    -- Cryptocurrency details
    crypto_type VARCHAR(50) NOT NULL,  -- "SOL", "USDT_SPL", "USDT_BEP20", "USDT_ARBITRUM", "USDT_POLYGON"
    network VARCHAR(50) NOT NULL,      -- "SOLANA", "BEP20", "ARBITRUM", "POLYGON"
    
    -- Blockchain transaction details
    transaction_hash VARCHAR(255),
    from_address VARCHAR(255),
    to_address VARCHAR(255) NOT NULL,  -- Merchant's wallet address
    
    -- Status and confirmations
    status VARCHAR(50) NOT NULL DEFAULT 'PENDING',  -- "PENDING", "CONFIRMING", "CONFIRMED", "FAILED", "REFUNDED", "EXPIRED"
    confirmations INT NOT NULL DEFAULT 0,
    required_confirmations INT NOT NULL,
    block_number BIGINT,
    
    -- Partial payments support
    partial_payments_enabled BOOLEAN NOT NULL DEFAULT false,
    total_paid DECIMAL(20,8) NOT NULL DEFAULT 0,
    remaining_balance DECIMAL(20,8),
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    confirmed_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL
);

-- Indexes for payment_transactions table
CREATE INDEX idx_payment_transactions_merchant ON payment_transactions(merchant_id);
CREATE INDEX idx_payment_transactions_payment_id ON payment_transactions(payment_id);
CREATE INDEX idx_payment_transactions_status ON payment_transactions(status);
CREATE INDEX idx_payment_transactions_transaction_hash ON payment_transactions(transaction_hash);
CREATE INDEX idx_payment_transactions_created_at ON payment_transactions(created_at);
CREATE INDEX idx_payment_transactions_expires_at ON payment_transactions(expires_at) 
    WHERE status IN ('PENDING', 'CONFIRMING');
CREATE INDEX idx_payment_transactions_user_id ON payment_transactions(user_id) 
    WHERE user_id IS NOT NULL;

-- Create payment_links table
-- Stores unique links for hosted payment pages
CREATE TABLE payment_links (
    id BIGSERIAL PRIMARY KEY,
    link_id VARCHAR(100) UNIQUE NOT NULL,  -- Short unique ID for URL (e.g., "lnk_abc123")
    payment_id BIGINT NOT NULL REFERENCES payment_transactions(id) ON DELETE CASCADE,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);

-- Indexes for payment_links table
CREATE INDEX idx_payment_links_link_id ON payment_links(link_id);
CREATE INDEX idx_payment_links_payment ON payment_links(payment_id);
CREATE INDEX idx_payment_links_merchant ON payment_links(merchant_id);
CREATE INDEX idx_payment_links_expires_at ON payment_links(expires_at);

-- Create partial_payments table
-- Tracks individual payments toward a single order when partial payments are enabled
CREATE TABLE partial_payments (
    id BIGSERIAL PRIMARY KEY,
    payment_id BIGINT NOT NULL REFERENCES payment_transactions(id) ON DELETE CASCADE,
    transaction_hash VARCHAR(255) NOT NULL UNIQUE,
    amount DECIMAL(20,8) NOT NULL,
    amount_usd DECIMAL(20,2) NOT NULL,
    confirmations INT NOT NULL DEFAULT 0,
    status VARCHAR(50) NOT NULL DEFAULT 'PENDING',  -- "PENDING", "CONFIRMED"
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    confirmed_at TIMESTAMPTZ
);

-- Indexes for partial_payments table
CREATE INDEX idx_partial_payments_payment ON partial_payments(payment_id);
CREATE INDEX idx_partial_payments_transaction_hash ON partial_payments(transaction_hash);
CREATE INDEX idx_partial_payments_status ON partial_payments(status);
CREATE INDEX idx_partial_payments_created_at ON partial_payments(created_at);

-- Add comments for documentation
COMMENT ON TABLE payment_transactions IS 'Payment requests and their status, supporting both full and partial payments';
COMMENT ON TABLE payment_links IS 'Unique links for hosted payment pages with QR codes';
COMMENT ON TABLE partial_payments IS 'Individual transactions toward a payment when partial payments are enabled';

COMMENT ON COLUMN payment_transactions.payment_id IS 'Public-facing payment identifier (e.g., pay_abc123)';
COMMENT ON COLUMN payment_transactions.fee_percentage IS 'Platform fee percentage applied to this payment';
COMMENT ON COLUMN payment_transactions.fee_amount IS 'Platform fee amount in cryptocurrency';
COMMENT ON COLUMN payment_transactions.fee_amount_usd IS 'Platform fee amount in USD';
COMMENT ON COLUMN payment_transactions.partial_payments_enabled IS 'Whether this payment accepts multiple partial transactions';
COMMENT ON COLUMN payment_transactions.total_paid IS 'Total amount paid so far (for partial payments)';
COMMENT ON COLUMN payment_transactions.remaining_balance IS 'Remaining amount to be paid (for partial payments)';
COMMENT ON COLUMN payment_transactions.metadata IS 'Custom merchant metadata stored as JSON';

COMMENT ON COLUMN payment_links.link_id IS 'Short unique identifier for the payment link URL';

COMMENT ON COLUMN partial_payments.transaction_hash IS 'Blockchain transaction hash for this partial payment';
COMMENT ON COLUMN partial_payments.amount IS 'Amount of this partial payment in cryptocurrency';
COMMENT ON COLUMN partial_payments.amount_usd IS 'Amount of this partial payment in USD';

-- Add constraint to ensure transaction_hash is unique when not null
CREATE UNIQUE INDEX idx_payment_transactions_transaction_hash_unique 
    ON payment_transactions(transaction_hash) 
    WHERE transaction_hash IS NOT NULL;

-- Add constraint to ensure remaining_balance is set when partial payments are enabled
ALTER TABLE payment_transactions 
    ADD CONSTRAINT chk_partial_payments_balance 
    CHECK (
        (partial_payments_enabled = false) OR 
        (partial_payments_enabled = true AND remaining_balance IS NOT NULL)
    );

-- Add constraint to ensure fee_percentage is within valid range (0.1% - 5%)
ALTER TABLE payment_transactions 
    ADD CONSTRAINT chk_fee_percentage_range 
    CHECK (fee_percentage >= 0.1 AND fee_percentage <= 5.0);

-- Add constraint to ensure amounts are positive
ALTER TABLE payment_transactions 
    ADD CONSTRAINT chk_amount_positive 
    CHECK (amount > 0 AND amount_usd > 0);

ALTER TABLE partial_payments 
    ADD CONSTRAINT chk_partial_amount_positive 
    CHECK (amount > 0 AND amount_usd > 0);

-- Add constraint to ensure total_paid doesn't exceed amount
ALTER TABLE payment_transactions 
    ADD CONSTRAINT chk_total_paid_valid 
    CHECK (total_paid >= 0 AND total_paid <= amount);
