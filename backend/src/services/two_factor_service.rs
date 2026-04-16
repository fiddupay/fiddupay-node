use crate::error::ServiceError;
use crate::utils::encryption::Encryption;
use serde::Serialize;
use sqlx::{PgPool, Row};
use std::time::{SystemTime, UNIX_EPOCH};
use totp_lite::{totp_custom, Sha1};

#[derive(Debug, Serialize)]
pub struct TwoFactorSetup {
    pub secret: String,
    pub qr_code_url: String,
    pub recovery_codes: Vec<String>,
}

pub struct TwoFactorService {
    pool: PgPool,
    enabled: bool,
    encryption: Encryption,
}

impl TwoFactorService {
    pub fn new(pool: PgPool, enabled: bool) -> Result<Self, ServiceError> {
        let encryption = Encryption::new()
            .map_err(|e| ServiceError::InternalError(format!("Encryption init failed: {}", e)))?;

        Ok(Self {
            pool,
            enabled,
            encryption,
        })
    }

    pub async fn setup_2fa(
        &self,
        merchant_id: i64,
        merchant_email: &str,
    ) -> Result<TwoFactorSetup, ServiceError> {
        if !self.enabled {
            return Err(ServiceError::ValidationError("2FA is disabled".to_string()));
        }

        let secret = self.generate_secret();
        let recovery_codes = self.generate_recovery_codes(10);

        let secret_encrypted = self
            .encryption
            .encrypt(&secret)
            .map_err(|e| ServiceError::InternalError(format!("Encryption failed: {}", e)))?;
        let codes_json = serde_json::to_string(&recovery_codes)?;
        let codes_encrypted = self
            .encryption
            .encrypt(&codes_json)
            .map_err(|e| ServiceError::InternalError(format!("Encryption failed: {}", e)))?;

        sqlx::query(
            r#"INSERT INTO two_factor_auth (merchant_id, secret_encrypted, recovery_codes_encrypted)
               VALUES ($1, $2, $3)
               ON CONFLICT (merchant_id) DO UPDATE 
               SET secret_encrypted = $2, recovery_codes_encrypted = $3, is_enabled = false"#,
        )
        .bind(merchant_id)
        .bind(&secret_encrypted)
        .bind(&codes_encrypted)
        .execute(&self.pool)
        .await?;

        let qr_code_url = format!(
            "otpauth://totp/CryptoGateway:{}?secret={}&issuer=CryptoGateway",
            merchant_email, secret
        );

        Ok(TwoFactorSetup {
            secret,
            qr_code_url,
            recovery_codes,
        })
    }

    pub async fn enable_2fa(&self, merchant_id: i64, code: &str) -> Result<(), ServiceError> {
        if !self.enabled {
            return Err(ServiceError::ValidationError("2FA is disabled".to_string()));
        }

        if !self.verify_code(merchant_id, code).await? {
            return Err(ServiceError::ValidationError(
                "Invalid 2FA code".to_string(),
            ));
        }

        sqlx::query(
            "UPDATE two_factor_auth SET is_enabled = true, enabled_at = NOW() WHERE merchant_id = $1"
        )
        .bind(merchant_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn disable_2fa(&self, merchant_id: i64, code: &str) -> Result<(), ServiceError> {
        if !self.verify_code(merchant_id, code).await? {
            return Err(ServiceError::ValidationError(
                "Invalid 2FA code".to_string(),
            ));
        }

        sqlx::query("UPDATE two_factor_auth SET is_enabled = false WHERE merchant_id = $1")
            .bind(merchant_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn verify_code(&self, merchant_id: i64, code: &str) -> Result<bool, ServiceError> {
        if !self.enabled {
            return Ok(true);
        }

        let record = sqlx::query(
            "SELECT secret_encrypted, is_enabled FROM two_factor_auth WHERE merchant_id = $1",
        )
        .bind(merchant_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(record) = record else {
            return Ok(false);
        };

        let is_enabled: bool = record.get("is_enabled");
        if !is_enabled {
            return Ok(true);
        }

        let secret_encrypted: String = record.get("secret_encrypted");
        let secret = self
            .encryption
            .decrypt(&secret_encrypted)
            .map_err(|e| ServiceError::InternalError(format!("Decryption failed: {}", e)))?;

        self.verify_totp(&secret, code)
    }

    pub async fn is_enabled(&self, merchant_id: i64) -> Result<bool, ServiceError> {
        if !self.enabled {
            return Ok(false);
        }

        let result: Option<bool> =
            sqlx::query_scalar("SELECT is_enabled FROM two_factor_auth WHERE merchant_id = $1")
                .bind(merchant_id)
                .fetch_optional(&self.pool)
                .await?;

        Ok(result.unwrap_or(false))
    }

    fn generate_secret(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..20).map(|_| rng.gen()).collect();

        base32::encode(base32::Alphabet::RFC4648 { padding: false }, &bytes)
    }

    fn generate_recovery_codes(&self, count: usize) -> Vec<String> {
        (0..count)
            .map(|_| {
                format!(
                    "{:04}-{:04}",
                    rand::random::<u16>() % 10000,
                    rand::random::<u16>() % 10000
                )
            })
            .collect()
    }

    fn verify_totp(&self, secret: &str, code: &str) -> Result<bool, ServiceError> {
        let secret_bytes = base32::decode(base32::Alphabet::RFC4648 { padding: false }, secret)
            .ok_or_else(|| ServiceError::InternalError("Invalid secret".to_string()))?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ServiceError::InternalError(format!("Time error: {}", e)))?
            .as_secs();

        let time_step: u64 = 30;
        let digits = 6;

        for offset in [-1, 0, 1] {
            let time = (timestamp as i64 + (offset * time_step as i64)) as u64 / time_step;
            let expected = totp_custom::<Sha1>(time_step, digits, &secret_bytes, time);

            if code == expected {
                return Ok(true);
            }
        }

        Ok(false)
    }
}
