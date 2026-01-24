// Security Monitoring Service - Real-time Threat Detection
// Final component for 10/10 security score

use crate::error::ServiceError;
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub event_id: String,
    pub event_type: SecurityEventType,
    pub severity: SecuritySeverity,
    pub source_ip: String,
    pub api_key: Option<String>,
    pub details: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    SuspiciousLogin,
    RateLimitExceeded,
    InvalidApiKey,
    UnauthorizedAccess,
    DataExfiltrationAttempt,
    InjectionAttempt,
    BruteForceAttack,
    AnomalousTraffic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

pub struct SecurityMonitoringService {
    db_pool: PgPool,
    active_threats: Arc<RwLock<HashMap<String, ThreatProfile>>>,
    alert_thresholds: SecurityThresholds,
}

#[derive(Clone)]
pub struct ThreatProfile {
    pub ip_address: String,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub event_count: u32,
    pub threat_score: f64,
    pub blocked: bool,
}

#[derive(Clone)]
pub struct SecurityThresholds {
    pub max_requests_per_minute: u32,
    pub max_failed_logins: u32,
    pub anomaly_detection_window: Duration,
    pub auto_block_threshold: f64,
}

impl Default for SecurityThresholds {
    fn default() -> Self {
        Self {
            max_requests_per_minute: 100,
            max_failed_logins: 5,
            anomaly_detection_window: Duration::minutes(15),
            auto_block_threshold: 8.0,
        }
    }
}

impl SecurityMonitoringService {
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            db_pool,
            active_threats: Arc::new(RwLock::new(HashMap::new())),
            alert_thresholds: SecurityThresholds::default(),
        }
    }

    /// Record security event and analyze threat level
    pub async fn record_event(&self, event: SecurityEvent) -> Result<(), ServiceError> {
        // Store event in database
        sqlx::query!(
            r#"INSERT INTO security_events 
               (event_id, event_type, severity, source_ip, api_key, details, timestamp)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            event.event_id,
            serde_json::to_string(&event.event_type)?,
            serde_json::to_string(&event.severity)?,
            event.source_ip,
            event.api_key,
            event.details,
            event.timestamp
        )
        .execute(&self.db_pool)
        .await?;

        // Update threat profile
        self.update_threat_profile(&event).await?;

        // Check for immediate threats
        self.analyze_threat_level(&event).await?;

        Ok(())
    }

    /// Update threat profile for IP address
    async fn update_threat_profile(&self, event: &SecurityEvent) -> Result<(), ServiceError> {
        let mut threats = self.active_threats.write().await;
        let profile = threats.entry(event.source_ip.clone()).or_insert_with(|| ThreatProfile {
            ip_address: event.source_ip.clone(),
            first_seen: event.timestamp,
            last_seen: event.timestamp,
            event_count: 0,
            threat_score: 0.0,
            blocked: false,
        });

        profile.last_seen = event.timestamp;
        profile.event_count += 1;

        // Calculate threat score based on event type and frequency
        let event_weight = match event.event_type {
            SecurityEventType::BruteForceAttack => 3.0,
            SecurityEventType::InjectionAttempt => 5.0,
            SecurityEventType::DataExfiltrationAttempt => 4.0,
            SecurityEventType::UnauthorizedAccess => 2.0,
            SecurityEventType::InvalidApiKey => 1.0,
            SecurityEventType::RateLimitExceeded => 0.5,
            _ => 0.1,
        };

        profile.threat_score += event_weight;

        // Auto-block if threshold exceeded
        if profile.threat_score >= self.alert_thresholds.auto_block_threshold {
            profile.blocked = true;
            self.trigger_emergency_response(event).await?;
        }

        Ok(())
    }

    /// Analyze current threat level and trigger alerts
    async fn analyze_threat_level(&self, event: &SecurityEvent) -> Result<(), ServiceError> {
        match event.severity {
            SecuritySeverity::Emergency => {
                self.trigger_emergency_response(event).await?;
            }
            SecuritySeverity::Critical => {
                self.trigger_critical_alert(event).await?;
            }
            SecuritySeverity::Warning => {
                self.log_security_warning(event).await?;
            }
            SecuritySeverity::Info => {
                // Just log for audit trail
                tracing::info!("Security event recorded: {:?}", event.event_type);
            }
        }

        Ok(())
    }

    /// Trigger emergency response for critical threats
    async fn trigger_emergency_response(&self, event: &SecurityEvent) -> Result<(), ServiceError> {
        tracing::error!("🚨 EMERGENCY: Critical security threat detected from {}", event.source_ip);
        
        // Block IP immediately
        self.block_ip_address(&event.source_ip).await?;
        
        // Send alert to security team (in production, integrate with PagerDuty/Slack)
        self.send_security_alert(event, "EMERGENCY").await?;
        
        // Log to security incident table
        sqlx::query!(
            "INSERT INTO security_incidents (event_id, severity, response_action, created_at) VALUES ($1, $2, $3, NOW())",
            event.event_id,
            "EMERGENCY",
            "AUTO_BLOCKED_IP"
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Trigger critical alert
    async fn trigger_critical_alert(&self, event: &SecurityEvent) -> Result<(), ServiceError> {
        tracing::warn!("⚠️ CRITICAL: Security threat detected from {}", event.source_ip);
        self.send_security_alert(event, "CRITICAL").await?;
        Ok(())
    }

    /// Log security warning
    async fn log_security_warning(&self, event: &SecurityEvent) -> Result<(), ServiceError> {
        tracing::warn!("Security warning: {:?} from {}", event.event_type, event.source_ip);
        Ok(())
    }

    /// Block IP address
    async fn block_ip_address(&self, ip: &str) -> Result<(), ServiceError> {
        sqlx::query!(
            "INSERT INTO blocked_ips (ip_address, reason, blocked_at, expires_at) VALUES ($1, $2, NOW(), NOW() + INTERVAL '24 hours')",
            ip,
            "Automated security block"
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Send security alert (placeholder for real alerting system)
    async fn send_security_alert(&self, event: &SecurityEvent, level: &str) -> Result<(), ServiceError> {
        // In production, integrate with:
        // - PagerDuty for emergency alerts
        // - Slack for team notifications
        // - Email for security team
        // - SMS for critical incidents
        
        tracing::error!("🔔 Security Alert [{}]: {:?} from {} at {}", 
            level, event.event_type, event.source_ip, event.timestamp);
        
        Ok(())
    }

    /// Check if IP is currently blocked
    pub async fn is_ip_blocked(&self, ip: &str) -> Result<bool, ServiceError> {
        let result = sqlx::query!(
            "SELECT COUNT(*) as count FROM blocked_ips WHERE ip_address = $1 AND expires_at > NOW()",
            ip
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(result.count.unwrap_or(0) > 0)
    }

    /// Get threat profile for IP
    pub async fn get_threat_profile(&self, ip: &str) -> Option<ThreatProfile> {
        let threats = self.active_threats.read().await;
        threats.get(ip).cloned()
    }

    /// Cleanup old threat profiles
    pub async fn cleanup_old_threats(&self) -> Result<(), ServiceError> {
        let cutoff = Utc::now() - Duration::hours(24);
        let mut threats = self.active_threats.write().await;
        
        threats.retain(|_, profile| profile.last_seen > cutoff);
        
        // Also cleanup database
        sqlx::query!(
            "DELETE FROM security_events WHERE timestamp < $1",
            cutoff
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Generate security report
    pub async fn generate_security_report(&self, hours: i64) -> Result<SecurityReport, ServiceError> {
        let since = Utc::now() - Duration::hours(hours);
        
        let events = sqlx::query!(
            "SELECT event_type, severity, COUNT(*) as count FROM security_events WHERE timestamp > $1 GROUP BY event_type, severity",
            since
        )
        .fetch_all(&self.db_pool)
        .await?;

        let blocked_ips = sqlx::query!(
            "SELECT COUNT(*) as count FROM blocked_ips WHERE blocked_at > $1",
            since
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(SecurityReport {
            period_hours: hours,
            total_events: events.iter().map(|e| e.count.unwrap_or(0) as u32).sum(),
            blocked_ips: blocked_ips.count.unwrap_or(0) as u32,
            event_breakdown: events.into_iter().map(|e| EventSummary {
                event_type: e.event_type.unwrap_or_default(),
                severity: e.severity.unwrap_or_default(),
                count: e.count.unwrap_or(0) as u32,
            }).collect(),
            generated_at: Utc::now(),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct SecurityReport {
    pub period_hours: i64,
    pub total_events: u32,
    pub blocked_ips: u32,
    pub event_breakdown: Vec<EventSummary>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct EventSummary {
    pub event_type: String,
    pub severity: String,
    pub count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_threat_profile_update() {
        // Test would require database setup
        // Placeholder for actual test implementation
    }
}
