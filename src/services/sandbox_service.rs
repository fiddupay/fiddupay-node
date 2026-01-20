// Sandbox Service
// Business logic for sandbox testing environment

use crate::error::ServiceError;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use chrono::Utc;
use nanoid::nanoid;
use serde::Serialize;
use sqlx::PgPool;

pub struct SandboxService {
    db_pool: PgPool,
}

impl SandboxService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
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
        let api_key = format!("test_{}", self.generate_sandbox_key());
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let api_key_hash = argon2
            .hash_password(api_key.as_bytes(), &salt)
            .map_err(|e| ServiceError::Internal(format!("Hash error: {}", e)))?
            .to_string();

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
        .fetch_one(&self.db_pool)
        .await?;

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

        sqlx::query!(
            "UPDATE payment_transactions SET status = $1, confirmed_at = $2 WHERE id = $3",
            new_status,
            confirmed_at,
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
        const ALPHABET: [char; 62] = [
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J',
            'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T',
            'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd',
            'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n',
            'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x',
            'y', 'z',
        ];
        nanoid!(32, &ALPHABET)
    }
}

#[derive(Debug, Serialize)]
pub struct SandboxCredentials {
    pub merchant_id: i64,
    pub sandbox_api_key: String,
    pub sandbox_mode: bool,
}
