-- Cross-Chain Payments via Delora Integration
-- Tracks every cross-chain quote, registration, and bridge completion

CREATE TABLE cross_chain_payments (
    id BIGSERIAL PRIMARY KEY,
    quote_id UUID NOT NULL UNIQUE,
    payment_transaction_id BIGINT REFERENCES payment_transactions(id) ON DELETE SET NULL,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id),
    invoice_id UUID,

    -- Quote snapshot (immutable record of what was shown to customer)
    origin_chain_id BIGINT NOT NULL,
    origin_currency_address VARCHAR(255) NOT NULL,
    origin_currency_symbol VARCHAR(50) NOT NULL,
    origin_currency_decimals INT NOT NULL,
    destination_chain_id BIGINT NOT NULL,
    destination_currency_address VARCHAR(255) NOT NULL,
    destination_currency_symbol VARCHAR(50) NOT NULL,
    destination_currency_decimals INT NOT NULL,
    input_amount VARCHAR(100) NOT NULL,
    output_amount VARCHAR(100) NOT NULL,
    min_output_amount VARCHAR(100) NOT NULL DEFAULT '0',

    -- Fee tracking
    delora_fee_amount VARCHAR(100),
    delora_fee_usd VARCHAR(100),
    integrator_fee_amount VARCHAR(100),
    integrator_fee_usd VARCHAR(100),
    integrator_fee_rate DECIMAL(5,4),

    -- Route info
    adapter VARCHAR(100) NOT NULL,
    route_id VARCHAR(255),
    route_snapshot JSONB,
    is_multistep BOOLEAN NOT NULL DEFAULT false,
    is_advanced BOOLEAN NOT NULL DEFAULT false,

    -- Execution tracking
    status VARCHAR(50) NOT NULL DEFAULT 'quote_requested',
    sender_address VARCHAR(255),
    merchant_destination_address VARCHAR(255) NOT NULL,
    calldata JSONB NOT NULL,
    calldata_to VARCHAR(255) NOT NULL,
    approval_address VARCHAR(255),

    -- On-chain tracking
    origin_tx_hash VARCHAR(255),
    destination_tx_hash VARCHAR(255),
    origin_block_number BIGINT,
    destination_block_number BIGINT,
    origin_confirmations INT DEFAULT 0,
    bridge_scan_metadata JSONB,

    -- Timing
    quote_expires_at TIMESTAMPTZ NOT NULL,
    tx_submitted_at TIMESTAMPTZ,
    origin_confirmed_at TIMESTAMPTZ,
    bridge_completed_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    failed_reason TEXT,

    -- Delora response metadata
    delora_warnings JSONB DEFAULT '[]',
    estimated_time_sec BIGINT,
    gas_info JSONB,

    -- Safety
    sandbox_mode BOOLEAN NOT NULL DEFAULT false,
    idempotency_key VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Lookup by quote_id (primary access pattern)
CREATE INDEX idx_ccp_quote_id ON cross_chain_payments(quote_id);
-- Lookup by merchant for dashboard
CREATE INDEX idx_ccp_merchant_id ON cross_chain_payments(merchant_id);
-- Link to payment_transactions
CREATE INDEX idx_ccp_payment_transaction_id ON cross_chain_payments(payment_transaction_id);
-- Status-based queries (bridge monitor)
CREATE INDEX idx_ccp_status ON cross_chain_payments(status);
-- Double-credit: composite unique on (tx_hash, chain) for origin side
CREATE UNIQUE INDEX idx_ccp_origin_tx_chain_unique 
    ON cross_chain_payments(origin_tx_hash, origin_chain_id) 
    WHERE origin_tx_hash IS NOT NULL AND deleted_at IS NULL;
-- Bridge monitor: find pending bridge completions
CREATE INDEX idx_ccp_bridge_pending ON cross_chain_payments(status, updated_at) 
    WHERE status IN ('tx_confirmed', 'bridge_pending');
-- Quote expiry cleanup
CREATE INDEX idx_ccp_quote_expires_at ON cross_chain_payments(quote_expires_at) 
    WHERE status = 'quote_requested';

-- Auto-update updated_at
CREATE TRIGGER update_cross_chain_payments_updated_at
    BEFORE UPDATE ON cross_chain_payments
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
