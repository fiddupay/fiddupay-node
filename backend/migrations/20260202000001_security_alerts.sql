-- Security alerts table for persistent alert storage and acknowledgement tracking
CREATE TABLE IF NOT EXISTS security_alerts (
    id SERIAL PRIMARY KEY,
    alert_id VARCHAR(100) UNIQUE NOT NULL,
    alert_type VARCHAR(50) NOT NULL,
    severity VARCHAR(20) NOT NULL CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    message TEXT NOT NULL,
    merchant_id BIGINT REFERENCES merchants(id),
    acknowledged BOOLEAN NOT NULL DEFAULT FALSE,
    acknowledged_by BIGINT REFERENCES merchants(id),
    acknowledged_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    metadata JSONB
);

-- Index for quick lookups
CREATE INDEX idx_security_alerts_alert_id ON security_alerts(alert_id);
CREATE INDEX idx_security_alerts_created_at ON security_alerts(created_at DESC);
CREATE INDEX idx_security_alerts_acknowledged ON security_alerts(acknowledged) WHERE acknowledged = FALSE;
CREATE INDEX idx_security_alerts_merchant_id ON security_alerts(merchant_id);

-- Security events table for audit logging
CREATE TABLE IF NOT EXISTS security_events (
    id SERIAL PRIMARY KEY,
    event_id VARCHAR(100) UNIQUE NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    severity VARCHAR(20) NOT NULL CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    description TEXT NOT NULL,
    merchant_id BIGINT REFERENCES merchants(id),
    ip_address INET,
    user_agent TEXT,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for quick lookups
CREATE INDEX idx_security_events_event_id ON security_events(event_id);
CREATE INDEX idx_security_events_created_at ON security_events(created_at DESC);
CREATE INDEX idx_security_events_merchant_id ON security_events(merchant_id);
CREATE INDEX idx_security_events_event_type ON security_events(event_type);
