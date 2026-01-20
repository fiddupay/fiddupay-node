use sqlx::PgPool;
use std::net::IpAddr;
use ipnetwork::IpNetwork;
use crate::error::ServiceError;

pub struct IpWhitelistService {
    pool: PgPool,
}

impl IpWhitelistService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn set_whitelist(&self, merchant_id: i32, ip_addresses: Vec<String>) -> Result<(), ServiceError> {
        if ip_addresses.len() > 10 {
            return Err(ServiceError::ValidationError("Maximum 10 IP addresses allowed".to_string()));
        }

        // Validate all IPs
        for ip in &ip_addresses {
            self.validate_ip(ip)?;
        }

        // Delete existing whitelist
        sqlx::query!("DELETE FROM ip_whitelist WHERE merchant_id = $1", merchant_id)
            .execute(&self.pool)
            .await?;

        // Insert new whitelist
        for ip in ip_addresses {
            sqlx::query!(
                "INSERT INTO ip_whitelist (merchant_id, ip_address) VALUES ($1, $2)",
                merchant_id,
                ip
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn get_whitelist(&self, merchant_id: i32) -> Result<Vec<String>, ServiceError> {
        let records = sqlx::query!("SELECT ip_address FROM ip_whitelist WHERE merchant_id = $1", merchant_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(records.into_iter().map(|r| r.ip_address).collect())
    }

    pub async fn is_ip_allowed(&self, merchant_id: i32, ip: &str) -> Result<bool, ServiceError> {
        let whitelist = self.get_whitelist(merchant_id).await?;
        
        // Empty whitelist = allow all
        if whitelist.is_empty() {
            return Ok(true);
        }

        let client_ip: IpAddr = ip.parse()
            .map_err(|_| ServiceError::ValidationError("Invalid IP address".to_string()))?;

        for entry in whitelist {
            if entry.contains('/') {
                // CIDR range
                let network: IpNetwork = entry.parse()
                    .map_err(|_| ServiceError::ValidationError("Invalid CIDR range".to_string()))?;
                if network.contains(client_ip) {
                    return Ok(true);
                }
            } else {
                // Single IP
                let allowed_ip: IpAddr = entry.parse()
                    .map_err(|_| ServiceError::ValidationError("Invalid IP address".to_string()))?;
                if client_ip == allowed_ip {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    pub async fn log_rejected_request(&self, merchant_id: i32, ip: &str, endpoint: &str) -> Result<(), ServiceError> {
        sqlx::query!(
            "INSERT INTO audit_logs (merchant_id, action_type, ip_address, details) VALUES ($1, $2, $3, $4)",
            merchant_id,
            "IP_REJECTED",
            ip,
            format!("Rejected request to {} from non-whitelisted IP", endpoint)
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    fn validate_ip(&self, ip: &str) -> Result<(), ServiceError> {
        if ip.contains('/') {
            // CIDR range
            ip.parse::<IpNetwork>()
                .map_err(|_| ServiceError::ValidationError(format!("Invalid CIDR range: {}", ip)))?;
        } else {
            // Single IP
            ip.parse::<IpAddr>()
                .map_err(|_| ServiceError::ValidationError(format!("Invalid IP address: {}", ip)))?;
        }
        Ok(())
    }
}
