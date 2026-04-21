-- Create policy_agreements table for NDPC compliance
-- Tracks granular merchant/user consent for privacy policies and terms

CREATE TABLE IF NOT EXISTS policy_agreements (
    id BIGSERIAL PRIMARY KEY,
    merchant_id BIGINT REFERENCES merchants(id) ON DELETE CASCADE,
    policy_type VARCHAR(50) NOT NULL, -- e.g., 'PRIVACY_POLICY', 'TERMS_OF_SERVICE'
    version VARCHAR(20) NOT NULL,    -- e.g., '2024.1'
    accepted_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ip_address VARCHAR(45),           -- Supports IPv4 and IPv6
    user_agent TEXT
);

-- Index for quick lookups by merchant
CREATE INDEX idx_policy_agreements_merchant ON policy_agreements(merchant_id);

-- Commentary for documentation
COMMENT ON TABLE policy_agreements IS 'Audit log of user/merchant consent to specific legal policy versions';
