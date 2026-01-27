// Comprehensive Admin API Test Suite
// Tests for admin-only endpoints and system management functionality

use axum::{
    body::Body,
    http::{Request, StatusCode, Method},
    Router,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::collections::HashMap;
use tower::ServiceExt;

// Test utilities and helpers
mod admin_test_utils {
    use super::*;
    use crate::api::state::AppState;
    use crate::config::Config;
    
    pub struct AdminTestContext {
        pub app: Router,
        pub db_pool: PgPool,
        pub super_admin_api_key: String,
        pub admin_api_key: String,
        pub merchant_api_key: String,
    }
    
    impl AdminTestContext {
        pub async fn new() -> Self {
            let config = Config::from_env().expect("Failed to load config");
            let db_pool = PgPool::connect(&config.database_url)
                .await
                .expect("Failed to connect to database");
            
            // Run migrations
            sqlx::migrate!("./migrations")
                .run(&db_pool)
                .await
                .expect("Failed to run migrations");
            
            let state = AppState::new(db_pool.clone(), config).await;
            let app = crate::api::routes::create_router(state);
            
            // Create test users with different roles
            let super_admin_api_key = create_test_user(&db_pool, "super_admin@test.com", "SUPER_ADMIN").await;
            let admin_api_key = create_test_user(&db_pool, "admin@test.com", "ADMIN").await;
            let merchant_api_key = create_test_user(&db_pool, "merchant@test.com", "MERCHANT").await;
            
            Self {
                app,
                db_pool,
                super_admin_api_key,
                admin_api_key,
                merchant_api_key,
            }
        }
        
        pub async fn make_request(&self, method: Method, path: &str, api_key: &str, body: Option<Value>) -> (StatusCode, Value) {
            let mut request = Request::builder()
                .method(method)
                .uri(path)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json");
            
            let request = if let Some(body) = body {
                request.body(Body::from(body.to_string())).unwrap()
            } else {
                request.body(Body::empty()).unwrap()
            };
            
            let response = self.app.clone().oneshot(request).await.unwrap();
            let status = response.status();
            
            let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let json: Value = serde_json::from_slice(&body).unwrap_or(json!({}));
            
            (status, json)
        }
        
        pub async fn cleanup(&self) {
            // Clean up test data
            sqlx::query!("DELETE FROM merchants WHERE email LIKE '%@test.com'")
                .execute(&self.db_pool)
                .await
                .ok();
        }
    }
    
    async fn create_test_user(db_pool: &PgPool, email: &str, role: &str) -> String {
        use sha2::{Sha256, Digest};
        
        let api_key = format!("test_{}_{}", role.to_lowercase(), nanoid::nanoid!(16));
        let mut hasher = Sha256::new();
        hasher.update(api_key.as_bytes());
        let api_key_hash = format!("{:x}", hasher.finalize());
        
        sqlx::query!(
            r#"
            INSERT INTO merchants (email, business_name, api_key_hash, role, is_active, sandbox_mode, created_at, updated_at)
            VALUES ($1, $2, $3, $4::user_role, $5, $6, NOW(), NOW())
            ON CONFLICT (email) DO UPDATE SET 
                api_key_hash = EXCLUDED.api_key_hash,
                role = EXCLUDED.role
            "#,
            email,
            format!("Test {} Business", role),
            api_key_hash,
            role,
            true,
            false
        )
        .execute(db_pool)
        .await
        .expect("Failed to create test user");
        
        api_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use admin_test_utils::AdminTestContext;
    
    #[tokio::test]
    async fn test_admin_context_setup() {
        let ctx = AdminTestContext::new().await;
        
        // Verify test context is properly set up
        assert!(!ctx.super_admin_api_key.is_empty());
        assert!(!ctx.admin_api_key.is_empty());
        assert!(!ctx.merchant_api_key.is_empty());
        
        ctx.cleanup().await;
    }
}