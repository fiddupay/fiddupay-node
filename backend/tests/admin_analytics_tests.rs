// Analytics and Monitoring Tests
// Tests for admin analytics, reporting, and monitoring functionality

use super::*;

#[cfg(test)]
mod analytics_monitoring_tests {
    use super::*;
    use admin_test_utils::AdminTestContext;
    
    #[tokio::test]
    async fn test_platform_analytics_overview() {
        let ctx = AdminTestContext::new().await;
        
        // Get platform-wide analytics
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/analytics/platform?period=30d",
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        
        // Verify analytics structure
        assert!(response["summary"].is_object());
        assert!(response["payment_volume"].is_object());
        assert!(response["merchant_growth"].is_object());
        assert!(response["revenue_metrics"].is_object());
        assert!(response["geographic_distribution"].is_object());
        
        // Check summary metrics
        let summary = &response["summary"];
        assert!(summary["total_merchants"].is_number());
        assert!(summary["active_merchants"].is_number());
        assert!(summary["total_payments"].is_number());
        assert!(summary["total_volume_usd"].is_string());
        assert!(summary["platform_revenue_usd"].is_string());
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_payment_analytics() {
        let ctx = AdminTestContext::new().await;
        
        // Get detailed payment analytics
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/analytics/payments?from=2024-01-01&to=2024-12-31&granularity=day",
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        
        // Verify payment analytics structure
        assert!(response["time_series"].is_array());
        assert!(response["by_currency"].is_object());
        assert!(response["by_network"].is_object());
        assert!(response["success_rates"].is_object());
        assert!(response["average_amounts"].is_object());
        
        // Test payment status breakdown
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/analytics/payments/status-breakdown",
            &ctx.admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["confirmed"].is_number());
        assert!(response["pending"].is_number());
        assert!(response["failed"].is_number());
        assert!(response["expired"].is_number());
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_revenue_analytics() {
        let ctx = AdminTestContext::new().await;
        
        // Get revenue analytics
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/analytics/revenue?period=90d",
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        
        // Verify revenue structure
        assert!(response["total_fees_collected"].is_string());
        assert!(response["fees_by_currency"].is_object());
        assert!(response["fees_by_merchant"].is_array());
        assert!(response["monthly_recurring_revenue"].is_string());
        assert!(response["growth_rate"].is_string());
        
        // Test revenue projections
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/analytics/revenue/projections",
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["next_month_projection"].is_string());
        assert!(response["confidence_interval"].is_object());
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_merchant_analytics() {
        let ctx = AdminTestContext::new().await;
        
        // Get merchant analytics
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/analytics/merchants",
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        
        // Verify merchant analytics
        assert!(response["total_merchants"].is_number());
        assert!(response["active_merchants"].is_number());
        assert!(response["new_registrations"].is_object());
        assert!(response["churn_rate"].is_string());
        assert!(response["top_merchants_by_volume"].is_array());
        
        // Test merchant segmentation
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/analytics/merchants/segmentation",
            &ctx.admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["by_volume"].is_object());
        assert!(response["by_activity"].is_object());
        assert!(response["by_geography"].is_object());
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_security_monitoring() {
        let ctx = AdminTestContext::new().await;
        
        // Get security events
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/security/events?severity=high&limit=100",
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["events"].is_array());
        assert!(response["total_count"].is_number());
        
        // Get security alerts
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/security/alerts?status=active",
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["alerts"].is_array());
        
        // Test fraud detection metrics
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/security/fraud-metrics",
            &ctx.admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["suspicious_transactions"].is_number());
        assert!(response["blocked_attempts"].is_number());
        assert!(response["false_positive_rate"].is_string());
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_performance_monitoring() {
        let ctx = AdminTestContext::new().await;
        
        // Get performance metrics
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/monitoring/performance",
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        
        // Verify performance metrics
        assert!(response["api_response_times"].is_object());
        assert!(response["database_performance"].is_object());
        assert!(response["blockchain_rpc_latency"].is_object());
        assert!(response["error_rates"].is_object());
        
        // Test specific endpoint performance
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/monitoring/performance/endpoints",
            &ctx.admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["endpoints"].is_array());
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_blockchain_monitoring() {
        let ctx = AdminTestContext::new().await;
        
        // Get blockchain network status
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/monitoring/blockchain",
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        
        // Verify blockchain monitoring
        assert!(response["networks"].is_object());
        assert!(response["rpc_health"].is_object());
        assert!(response["gas_prices"].is_object());
        assert!(response["confirmation_times"].is_object());
        
        // Test specific network monitoring
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/monitoring/blockchain/solana",
            &ctx.admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["network_status"].is_string());
        assert!(response["current_slot"].is_number());
        assert!(response["tps"].is_number());
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_alert_management() {
        let ctx = AdminTestContext::new().await;
        
        // Create a custom alert
        let (status, response) = ctx.make_request(
            Method::POST,
            "/api/v1/admin/alerts",
            &ctx.super_admin_api_key,
            Some(json!({
                "name": "High Error Rate Alert",
                "description": "Alert when error rate exceeds 5%",
                "conditions": {
                    "metric": "error_rate",
                    "operator": "greater_than",
                    "threshold": 0.05,
                    "duration": "5m"
                },
                "notifications": {
                    "email": ["admin@fiddupay.com"],
                    "webhook": "https://hooks.slack.com/services/..."
                }
            }))
        ).await;
        assert_eq!(status, StatusCode::CREATED);
        let alert_id = response["alert_id"].as_str().unwrap();
        
        // List all alerts
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/alerts",
            &ctx.admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["alerts"].is_array());
        
        // Update alert
        let (status, _) = ctx.make_request(
            Method::PUT,
            &format!("/api/v1/admin/alerts/{}", alert_id),
            &ctx.super_admin_api_key,
            Some(json!({
                "conditions": {
                    "threshold": 0.03
                }
            }))
        ).await;
        assert_eq!(status, StatusCode::OK);
        
        // Delete alert
        let (status, _) = ctx.make_request(
            Method::DELETE,
            &format!("/api/v1/admin/alerts/{}", alert_id),
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_report_generation() {
        let ctx = AdminTestContext::new().await;
        
        // Generate comprehensive platform report
        let (status, response) = ctx.make_request(
            Method::POST,
            "/api/v1/admin/reports/generate",
            &ctx.super_admin_api_key,
            Some(json!({
                "report_type": "platform_summary",
                "period": {
                    "from": "2024-01-01T00:00:00Z",
                    "to": "2024-12-31T23:59:59Z"
                },
                "format": "pdf",
                "sections": [
                    "executive_summary",
                    "payment_analytics",
                    "merchant_metrics",
                    "revenue_analysis",
                    "security_overview"
                ]
            }))
        ).await;
        assert_eq!(status, StatusCode::ACCEPTED);
        let report_id = response["report_id"].as_str().unwrap();
        
        // Check report status
        let (status, response) = ctx.make_request(
            Method::GET,
            &format!("/api/v1/admin/reports/{}/status", report_id),
            &ctx.admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["status"].is_string());
        
        // List available reports
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/reports",
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["reports"].is_array());
        
        ctx.cleanup().await;
    }
}