// Merchant Management Tests
// Tests for admin operations on merchant accounts

use super::*;

#[cfg(test)]
mod merchant_management_tests {
    use super::*;
    use admin_test_utils::AdminTestContext;
    
    #[tokio::test]
    async fn test_list_all_merchants() {
        let ctx = AdminTestContext::new().await;
        
        // Super admin can list all merchants
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/merchants?limit=50&offset=0",
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["merchants"].is_array());
        assert!(response["total_count"].is_number());
        assert!(response["pagination"].is_object());
        
        // Admin can also list merchants
        let (status, _) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/merchants",
            &ctx.admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        
        // Merchant cannot list other merchants
        let (status, _) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/merchants",
            &ctx.merchant_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_merchant_detailed_view() {
        let ctx = AdminTestContext::new().await;
        
        // Create a test merchant first
        let merchant_id = create_test_merchant(&ctx.db_pool).await;
        
        // Get detailed merchant information
        let (status, response) = ctx.make_request(
            Method::GET,
            &format!("/api/v1/admin/merchants/{}", merchant_id),
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        
        // Verify detailed merchant data
        assert!(response["merchant_info"].is_object());
        assert!(response["payment_stats"].is_object());
        assert!(response["wallet_configs"].is_array());
        assert!(response["recent_activity"].is_array());
        assert!(response["security_settings"].is_object());
        
        // Check specific fields
        let merchant_info = &response["merchant_info"];
        assert!(merchant_info["id"].is_number());
        assert!(merchant_info["email"].is_string());
        assert!(merchant_info["business_name"].is_string());
        assert!(merchant_info["role"].is_string());
        assert!(merchant_info["is_active"].is_boolean());
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_merchant_account_management() {
        let ctx = AdminTestContext::new().await;
        let merchant_id = create_test_merchant(&ctx.db_pool).await;
        
        // Test account suspension
        let (status, response) = ctx.make_request(
            Method::POST,
            &format!("/api/v1/admin/merchants/{}/suspend", merchant_id),
            &ctx.super_admin_api_key,
            Some(json!({
                "reason": "Suspicious activity detected",
                "duration_days": 7,
                "notify_merchant": true
            }))
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["status"], "suspended");
        
        // Test account reactivation
        let (status, response) = ctx.make_request(
            Method::POST,
            &format!("/api/v1/admin/merchants/{}/reactivate", merchant_id),
            &ctx.super_admin_api_key,
            Some(json!({
                "reason": "Issue resolved",
                "notify_merchant": true
            }))
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["status"], "active");
        
        // Test role change (super admin only)
        let (status, response) = ctx.make_request(
            Method::PUT,
            &format!("/api/v1/admin/merchants/{}/role", merchant_id),
            &ctx.super_admin_api_key,
            Some(json!({"role": "ADMIN"}))
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["new_role"], "ADMIN");
        
        // Admin cannot change roles
        let (status, _) = ctx.make_request(
            Method::PUT,
            &format!("/api/v1/admin/merchants/{}/role", merchant_id),
            &ctx.admin_api_key,
            Some(json!({"role": "MERCHANT"}))
        ).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_merchant_fee_management() {
        let ctx = AdminTestContext::new().await;
        let merchant_id = create_test_merchant(&ctx.db_pool).await;
        
        // Update merchant fee structure
        let (status, response) = ctx.make_request(
            Method::PUT,
            &format!("/api/v1/admin/merchants/{}/fees", merchant_id),
            &ctx.super_admin_api_key,
            Some(json!({
                "fee_percentage": "1.5",
                "minimum_fee_usd": "0.25",
                "customer_pays_fee": false,
                "custom_rates": {
                    "SOL": "1.0",
                    "USDT_SPL": "1.2"
                }
            }))
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["fee_percentage"], "1.5");
        
        // Get merchant fee structure
        let (status, response) = ctx.make_request(
            Method::GET,
            &format!("/api/v1/admin/merchants/{}/fees", merchant_id),
            &ctx.admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["current_fees"].is_object());
        assert!(response["fee_history"].is_array());
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_merchant_limits_management() {
        let ctx = AdminTestContext::new().await;
        let merchant_id = create_test_merchant(&ctx.db_pool).await;
        
        // Set merchant limits
        let (status, response) = ctx.make_request(
            Method::PUT,
            &format!("/api/v1/admin/merchants/{}/limits", merchant_id),
            &ctx.super_admin_api_key,
            Some(json!({
                "daily_limit_usd": "10000.00",
                "monthly_limit_usd": "250000.00",
                "single_transaction_limit_usd": "5000.00",
                "rate_limit_requests_per_minute": 100
            }))
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["daily_limit_usd"], "10000.00");
        
        // Get merchant limits
        let (status, response) = ctx.make_request(
            Method::GET,
            &format!("/api/v1/admin/merchants/{}/limits", merchant_id),
            &ctx.admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["current_limits"].is_object());
        assert!(response["usage_stats"].is_object());
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_merchant_api_key_management() {
        let ctx = AdminTestContext::new().await;
        let merchant_id = create_test_merchant(&ctx.db_pool).await;
        
        // Force API key rotation
        let (status, response) = ctx.make_request(
            Method::POST,
            &format!("/api/v1/admin/merchants/{}/rotate-api-key", merchant_id),
            &ctx.super_admin_api_key,
            Some(json!({
                "reason": "Security audit requirement",
                "notify_merchant": true
            }))
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["new_api_key"].is_string());
        assert!(response["old_key_revoked_at"].is_string());
        
        // List merchant API keys
        let (status, response) = ctx.make_request(
            Method::GET,
            &format!("/api/v1/admin/merchants/{}/api-keys", merchant_id),
            &ctx.admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["active_keys"].is_array());
        assert!(response["revoked_keys"].is_array());
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_merchant_search_and_filtering() {
        let ctx = AdminTestContext::new().await;
        
        // Search merchants by email
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/merchants/search?q=test@example.com",
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["results"].is_array());
        
        // Filter merchants by status
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/merchants?status=active&role=MERCHANT",
            &ctx.admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["merchants"].is_array());
        
        // Filter by registration date
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/merchants?registered_after=2024-01-01&registered_before=2024-12-31",
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        
        ctx.cleanup().await;
    }
    
    async fn create_test_merchant(db_pool: &PgPool) -> i64 {
        let result = sqlx::query!(
            r#"
            INSERT INTO merchants (email, business_name, api_key_hash, role, is_active, sandbox_mode, created_at, updated_at)
            VALUES ($1, $2, $3, $4::user_role, $5, $6, NOW(), NOW())
            RETURNING id
            "#,
            "test_merchant@example.com",
            "Test Merchant Business",
            "test_hash",
            "MERCHANT",
            true,
            false
        )
        .fetch_one(db_pool)
        .await
        .expect("Failed to create test merchant");
        
        result.id
    }
}