// Sandbox Service
// Business logic for sandbox testing environment

use crate::error::ServiceError;
use crate::services::merchant_service::MerchantService;
use crate::utils::api_keys::ApiKeyGenerator;
use chrono::Utc;
use nanoid::nanoid;
use serde::Serialize;
use sqlx::PgPool;

pub struct SandboxService {
    db_pool: PgPool,
    config: crate::config::Config,
}

impl SandboxService {
    pub fn new(db_pool: PgPool, config: crate::config::Config) -> Self {
        Self { db_pool, config }
    }

    /// Create sandbox credentials for a merchant
    /// 
    /// # Requirements
    /// * 10.1: Generate test API credentials when sandbox mode enabled
    /// * 10.4: Clearly distinguish sandbox from production
    pub async fn create_sandbox_credentials(
        &self,
        merchant_id: i64,
    ) -> Result<SandboxCredentials, ServiceError> {
        // Use searchable session key format for better security and performance
        let api_key = ApiKeyGenerator::generate_session_key(merchant_id, false);
        
        // Hash the API key using Argon2 (consistent with MerchantService)
        use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
        use rand::rngs::OsRng;
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let api_key_hash = argon2.hash_password(api_key.as_bytes(), &salt)
            .map_err(|_| ServiceError::InternalError("Failed to hash API key".to_string()))?
            .to_string();
        
        (api_key, api_key_hash)

        sqlx::query!(
            "UPDATE merchants SET sandbox_mode = true, api_key_hash = $1, updated_at = $2 WHERE id = $3",
            api_key_hash,
            Utc::now(),
            merchant_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(SandboxCredentials {
            merchant_id,
            sandbox_api_key: api_key,
            sandbox_mode: true,
        })
    }

    /// Check if an API key is a sandbox key
    /// 
    /// # Requirements
    /// * 10.4: Distinguish sandbox from production
    pub fn is_sandbox_key(&self, api_key: &str) -> bool {
        api_key.starts_with("test_")
    }

    /// Verify merchant is in sandbox mode
    pub async fn verify_sandbox_merchant(
        &self,
        merchant_id: i64,
    ) -> Result<bool, ServiceError> {
        let merchant = sqlx::query!(
            "SELECT sandbox_mode FROM merchants WHERE id = $1",
            merchant_id
        )
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| ServiceError::NotFound("Merchant not found".to_string()))?;

        Ok(merchant.sandbox_mode)
    }

    /// Simulate payment confirmation in sandbox mode
    /// 
    /// # Requirements
    /// * 10.2: Simulate payment confirmations without blockchain verification
    /// * 10.5: Allow manual payment status changes for testing
    pub async fn simulate_confirmation(
        &self,
        payment_id: &str,
        merchant_id: i64,
        success: bool,
        transaction_hash: Option<String>,
        from_address: Option<String>,
    ) -> Result<(), ServiceError> {
        if !self.verify_sandbox_merchant(merchant_id).await? {
            return Err(ServiceError::Forbidden(
                "Simulation only available in sandbox mode".to_string()
            ));
        }

        let payment = sqlx::query!(
            "SELECT id, merchant_id FROM payment_transactions WHERE payment_id = $1",
            payment_id
        )
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| ServiceError::NotFound("Payment not found".to_string()))?;

        if payment.merchant_id != merchant_id {
            return Err(ServiceError::Forbidden("Access denied".to_string()));
        }

        let new_status = if success { "CONFIRMED" } else { "FAILED" };
        let confirmed_at = if success { Some(Utc::now()) } else { None };
        
        // Use provided values or defaults for successful payments
        let tx_hash = if success { 
            Some(transaction_hash.unwrap_or_else(|| format!("sandbox_tx_{}", nanoid!(16))))
        } else { 
            None 
        };
        
        // Only set from_address if success (or failed?) - usually on transaction creation, but here we simulate receiving it
        let sender = if success {
            Some(from_address.unwrap_or_else(|| "0x_sandbox_mock_sender".to_string()))
        } else {
            None
        };

        sqlx::query!(
            "UPDATE payment_transactions SET status = $1, confirmed_at = $2, transaction_hash = $3, from_address = $4 WHERE id = $5",
            new_status,
            confirmed_at,
            tx_hash,
            sender,
            payment.id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Enforce sandbox data isolation
    /// 
    /// # Requirements
    /// * 10.6: Prevent sandbox keys from accessing production data
    pub async fn enforce_sandbox_isolation(
        &self,
        merchant_id: i64,
        api_key: &str,
    ) -> Result<(), ServiceError> {
        let is_sandbox_key = self.is_sandbox_key(api_key);
        let merchant_sandbox_mode = self.verify_sandbox_merchant(merchant_id).await?;

        if is_sandbox_key != merchant_sandbox_mode {
            return Err(ServiceError::Forbidden(
                "Sandbox/production mode mismatch".to_string()
            ));
        }

        Ok(())
    }

    fn generate_sandbox_key(&self) -> String {
        ApiKeyGenerator::generate_sandbox_key()
    }
}

#[derive(Debug, Serialize)]
pub struct SandboxCredentials {
    pub merchant_id: i64,
    pub sandbox_api_key: String,
    pub sandbox_mode: bool,
}
