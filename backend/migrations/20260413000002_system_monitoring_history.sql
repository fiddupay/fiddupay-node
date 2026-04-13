-- Migration: System Monitoring History
-- Stores daily snapshots of service health for the status page charts

CREATE TABLE IF NOT EXISTS system_health_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    service_name VARCHAR(100) NOT NULL,
    status VARCHAR(20) NOT NULL, -- operational, degraded, outage
    latency_ms INTEGER,
    cpu_usage FLOAT,
    memory_usage_gb FLOAT,
    metadata JSONB -- Stores extra info like error messages or RPC provider name
);

-- Index for efficient data retrieval for the status page (last 90 days)
CREATE INDEX IF NOT EXISTS idx_system_health_timestamp ON system_health_history(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_system_health_service ON system_health_history(service_name);

-- Utility function to get daily uptime percentage for the last 90 days
-- This will be used by the frontend to render the bars accurately
CREATE OR REPLACE VIEW daily_uptime_summary AS
SELECT 
    date_trunc('day', timestamp) as day,
    service_name,
    COUNT(*) FILTER (WHERE status = 'operational')::FLOAT / COUNT(*)::FLOAT * 100 as uptime_percent
FROM system_health_history
GROUP BY 1, 2
ORDER BY 1 DESC;
