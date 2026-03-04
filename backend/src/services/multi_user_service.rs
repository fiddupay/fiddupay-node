use sqlx::{PgPool, Row};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use crate::error::ServiceError;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum UserRole {
    #[serde(rename = "SUPER_ADMIN")]
    SuperAdmin,
    #[serde(rename = "ADMIN")]
    Admin,
    #[serde(rename = "MODERATOR")]
    Moderator,
    #[serde(rename = "MERCHANT")]
    Merchant,
    #[serde(rename = "USER")]
    User,
}

impl UserRole {
    pub fn as_str(&self) -> &str {
        match self {
            UserRole::SuperAdmin => "SUPER_ADMIN",
            UserRole::Admin => "ADMIN",
            UserRole::Moderator => "MODERATOR",
            UserRole::Merchant => "MERCHANT",
            UserRole::User => "USER",
        }
    }

    pub fn can_manage_users(&self) -> bool {
        matches!(self, UserRole::SuperAdmin | UserRole::Admin | UserRole::Merchant)
    }

    pub fn can_approve_withdrawals(&self) -> bool {
        matches!(self, UserRole::SuperAdmin | UserRole::Admin)
    }

    pub fn can_view_analytics(&self) -> bool {
        !matches!(self, UserRole::User)
    }
}

#[derive(Debug, Serialize)]
pub struct MerchantUser {
    pub id: i32,
    pub merchant_id: i64,
    pub email: String,
    pub role: String,
    pub is_active: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub role: String,
}

pub struct MultiUserService {
    pool: PgPool,
}

impl MultiUserService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, merchant_id: i64, req: CreateUserRequest) -> Result<MerchantUser, ServiceError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(req.password.as_bytes(), &salt)
            .map_err(|e| ServiceError::InternalError(format!("Password hashing failed: {}", e)))?
            .to_string();

        let record = sqlx::query_as::<_, (i32, i64, String, String, bool, Option<DateTime<Utc>>, DateTime<Utc>)>(
            r#"INSERT INTO merchant_users (merchant_id, email, password_hash, role)
               VALUES ($1, $2, $3, $4::user_role)
               RETURNING id, merchant_id, email, role::text, is_active, last_login, created_at"#
        )
        .bind(merchant_id)
        .bind(&req.email)
        .bind(&password_hash)
        .bind(&req.role)
        .fetch_one(&self.pool)
        .await?;

        Ok(MerchantUser {
            id: record.0,
            merchant_id: record.1,
            email: record.2,
            role: record.3,
            is_active: record.4,
            last_login: record.5,
            created_at: record.6,
        })
    }

    pub async fn list_users(&self, merchant_id: i64) -> Result<Vec<MerchantUser>, ServiceError> {
        let records = sqlx::query(
            "SELECT id, merchant_id, email, role::text as role, is_active, last_login, created_at
             FROM merchant_users WHERE merchant_id = $1 ORDER BY created_at DESC"
        )
        .bind(merchant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(records.into_iter().map(|r| MerchantUser {
            id: r.get("id"),
            merchant_id: r.get("merchant_id"),
            email: r.get("email"),
            role: r.try_get::<Option<String>, _>("role").ok().flatten().unwrap_or("MERCHANT".to_string()),
            is_active: r.get("is_active"),
            last_login: r.get("last_login"),
            created_at: r.get("created_at"),
        }).collect())
    }

    pub async fn update_role(&self, merchant_id: i64, user_id: i32, new_role: &str) -> Result<(), ServiceError> {
        sqlx::query(
            "UPDATE merchant_users SET role = $3::user_role, updated_at = NOW()
             WHERE id = $1 AND merchant_id = $2"
        )
        .bind(user_id)
        .bind(merchant_id)
        .bind(new_role)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn deactivate_user(&self, merchant_id: i64, user_id: i32) -> Result<(), ServiceError> {
        sqlx::query(
            "UPDATE merchant_users SET is_active = false, updated_at = NOW()
             WHERE id = $1 AND merchant_id = $2"
        )
        .bind(user_id)
        .bind(merchant_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn authenticate(&self, email: &str, password: &str) -> Result<MerchantUser, ServiceError> {
        let record = sqlx::query(
            "SELECT id, merchant_id, email, password_hash, role::text as role, is_active, last_login, created_at
             FROM merchant_users WHERE email = $1 AND is_active = true"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ServiceError::Unauthorized("Invalid credentials".to_string()))?;

        let stored_hash: String = record.get("password_hash");
        let parsed_hash = PasswordHash::new(&stored_hash)
            .map_err(|e| ServiceError::InternalError(format!("Invalid hash: {}", e)))?;
        
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| ServiceError::Unauthorized("Invalid credentials".to_string()))?;

        let user_id: i32 = record.get("id");
        sqlx::query(
            "UPDATE merchant_users SET last_login = NOW() WHERE id = $1"
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(MerchantUser {
            id: user_id,
            merchant_id: record.get("merchant_id"),
            email: record.get("email"),
            role: record.try_get::<Option<String>, _>("role").ok().flatten().unwrap_or("MERCHANT".to_string()),
            is_active: record.get("is_active"),
            last_login: Some(Utc::now()),
            created_at: record.get("created_at"),
        })
    }
}
