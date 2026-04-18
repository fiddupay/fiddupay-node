-- Idempotency Keys Table
-- Prevents duplicate processing of the same request (e.g., double-spend on payment confirmation)
CREATE TABLE IF NOT EXISTS idempotency_keys (
    key_hash       TEXT        PRIMARY KEY,
    merchant_id    BIGINT      NOT NULL,
    endpoint       TEXT        NOT NULL,
    response_code  SMALLINT,
    response_body  JSONB,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at     TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '24 hours')
);

CREATE INDEX IF NOT EXISTS idx_idempotency_expires ON idempotency_keys (expires_at);

-- Cleanup job: automatically purge expired keys (run daily via pg_cron or background task)
-- DELETE FROM idempotency_keys WHERE expires_at < NOW();
