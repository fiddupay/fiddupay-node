-- Fee Collections Table
-- Records all platform fee collection transactions

CREATE TABLE IF NOT EXISTS fee_collections (
    id SERIAL PRIMARY KEY,
    payment_id BIGINT NOT NULL,
    merchant_id BIGINT NOT NULL,
    network VARCHAR(50) NOT NULL,
    fee_amount NUMERIC(20, 8) NOT NULL,
    from_address VARCHAR(255) NOT NULL,
    to_address VARCHAR(255) NOT NULL,
    transaction_hash VARCHAR(255),
    status VARCHAR(50) NOT NULL DEFAULT 'PENDING',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_fee_collections_payment ON fee_collections (payment_id);

CREATE INDEX IF NOT EXISTS idx_fee_collections_merchant ON fee_collections (merchant_id);