// System Management Tests
// Tests for system-level admin operations

use super::*;

#[cfg(test)]
mod system_management_tests {
    use super::*;
    use admin_test_utils::AdminTestContext;
    
    #[tokio::test]
    async fn test_system_status_access_control() {
        let ctx = AdminTestContext::new().await;
        
        // Super admin should have full access
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/system/status",
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["system_health"].is_object());
        
        // Admin should have read access
        let (status, _) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/system/status",
            &ctx.admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        
        // Merchant should be denied
        let (status, _) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/system/status",
            &ctx.merchant_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_system_configuration_management() {
        let ctx = AdminTestContext::new().await;
        
        // Test getting system configuration
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/system/config",
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["rate_limits"].is_object());
        assert!(response["fee_settings"].is_object());
        
        // Test updating system configuration
        let config_update = json!({
            "rate_limits": {
                "requests_per_minute": 120,
                "burst_limit": 10
            },
            "fee_settings": {
                "default_fee_percentage": "2.5",
                "minimum_fee_usd": "0.50"
            }
        });
        
        let (status, response) = ctx.make_request(
            Method::PUT,
            "/api/v1/admin/system/config",
            &ctx.super_admin_api_key,
            Some(config_update)
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["message"], "Configuration updated successfully");
        
        // Admin should not be able to update config
        let (status, _) = ctx.make_request(
            Method::PUT,
            "/api/v1/admin/system/config",
            &ctx.admin_api_key,
            Some(json!({"rate_limits": {"requests_per_minute": 100}}))
        ).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_system_maintenance_mode() {
        let ctx = AdminTestContext::new().await;
        
        // Enable maintenance mode
        let (status, response) = ctx.make_request(
            Method::POST,
            "/api/v1/admin/system/maintenance",
            &ctx.super_admin_api_key,
            Some(json!({
                "enabled": true,
                "message": "System maintenance in progress",
                "estimated_duration": "30 minutes"
            }))
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["maintenance_mode"], true);
        
        // Check maintenance status
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/system/maintenance",
            &ctx.admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["enabled"], true);
        assert_eq!(response["message"], "System maintenance in progress");
        
        // Disable maintenance mode
        let (status, _) = ctx.make_request(
            Method::POST,
            "/api/v1/admin/system/maintenance",
            &ctx.super_admin_api_key,
            Some(json!({"enabled": false}))
        ).await;
        assert_eq!(status, StatusCode::OK);
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_system_health_monitoring() {
        let ctx = AdminTestContext::new().await;
        
        // Get comprehensive system health
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/system/health",
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        
        // Verify health check components
        assert!(response["database"]["status"].is_string());
        assert!(response["redis"]["status"].is_string());
        assert!(response["blockchain_rpcs"].is_object());
        assert!(response["memory_usage"].is_object());
        assert!(response["disk_usage"].is_object());
        
        // Test health check with specific component
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/system/health/database",
            &ctx.admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["connection_pool"].is_object());
        assert!(response["query_performance"].is_object());
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_system_logs_access() {
        let ctx = AdminTestContext::new().await;
        
        // Get system logs
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/system/logs?level=error&limit=100",
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["logs"].is_array());
        assert!(response["total_count"].is_number());
        
        // Test log filtering
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/system/logs?component=payment_service&from=2024-01-01T00:00:00Z",
            &ctx.admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        
        // Merchant should not access system logs
        let (status, _) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/system/logs",
            &ctx.merchant_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_system_metrics_collection() {
        let ctx = AdminTestContext::new().await;
        
        // Get system metrics
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/system/metrics",
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        
        // Verify metrics structure
        assert!(response["performance"].is_object());
        assert!(response["usage"].is_object());
        assert!(response["errors"].is_object());
        assert!(response["blockchain_stats"].is_object());
        
        // Test metrics with time range
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/system/metrics?from=2024-01-01T00:00:00Z&to=2024-12-31T23:59:59Z&granularity=hour",
            &ctx.admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["time_series"].is_array());
        
        ctx.cleanup().await;
    }
}