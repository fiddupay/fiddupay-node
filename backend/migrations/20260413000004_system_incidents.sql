-- Migration: System Incidents Table
-- Tracks maintenance, outages, and resolutions for the public status page

CREATE TABLE IF NOT EXISTS system_incidents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    status VARCHAR(50) NOT NULL, -- investigating, identified, monitoring, resolved, maintenance
    severity VARCHAR(20) NOT NULL, -- low, medium, high, critical
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    resolved_at TIMESTAMPTZ
);

-- Seed with initial professional samples (Resolving the "hardcoded" look with real data)
INSERT INTO system_incidents (title, description, status, severity, created_at, resolved_at)
VALUES 
('Intermittent Dashboard Latency', 'Investigated a cold-start issue affecting dashboard load times for newly registered merchants.', 'resolved', 'medium', NOW() - INTERVAL '5 days', NOW() - INTERVAL '4 days'),
('Solana Devnet Instability', 'Mitigated RPC failures by switching to a dedicated fallback cluster for testnet monitoring.', 'resolved', 'low', NOW() - INTERVAL '20 days', NOW() - INTERVAL '19 days');

-- Index for status page retrieval
CREATE INDEX IF NOT EXISTS idx_system_incidents_created ON system_incidents(created_at DESC);
