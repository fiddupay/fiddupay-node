// Security and Compliance Tests
// Tests for admin security operations, compliance, and audit functionality

use super::*;

#[cfg(test)]
mod security_compliance_tests {
    use super::*;
    use admin_test_utils::AdminTestContext;
    
    #[tokio::test]
    async fn test_audit_log_management() {
        let ctx = AdminTestContext::new().await;
        
        // Get comprehensive audit logs
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/audit/logs?from=2024-01-01&to=2024-12-31&limit=1000",
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        
        // Verify audit log structure
        assert!(response["logs"].is_array());
        assert!(response["total_count"].is_number());
        assert!(response["filters_applied"].is_object());
        
        // Test audit log filtering by action type
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/audit/logs?action_type=MERCHANT_SUSPENDED&user_role=SUPER_ADMIN",
            &ctx.admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        
        // Test audit log export
        let (status, response) = ctx.make_request(
            Method::POST,
            "/api/v1/admin/audit/export",
            &ctx.super_admin_api_key,
            Some(json!({
                "format": "csv",
                "filters": {
                    "from": "2024-01-01T00:00:00Z",
                    "to": "2024-12-31T23:59:59Z",
                    "action_types": ["PAYMENT_CREATED", "MERCHANT_REGISTERED", "API_KEY_ROTATED"]
                }
            }))
        ).await;
        assert_eq!(status, StatusCode::ACCEPTED);
        assert!(response["export_id"].is_string());
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_compliance_monitoring() {
        let ctx = AdminTestContext::new().await;
        
        // Get compliance dashboard
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/compliance/dashboard",
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        
        // Verify compliance metrics
        assert!(response["kyc_status"].is_object());
        assert!(response["aml_alerts"].is_object());
        assert!(response["transaction_monitoring"].is_object());
        assert!(response["regulatory_reports"].is_object());
        
        // Test AML transaction screening
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/compliance/aml/alerts?status=active&risk_level=high",
            &ctx.admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["alerts"].is_array());
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_kyc_management() {
        let ctx = AdminTestContext::new().await;
        let merchant_id = create_test_merchant(&ctx.db_pool).await;
        
        // Get KYC status for merchant
        let (status, response) = ctx.make_request(
            Method::GET,
            &format!("/api/v1/admin/compliance/kyc/merchant/{}", merchant_id),
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["kyc_status"].is_string());
        assert!(response["documents"].is_array());
        assert!(response["verification_history"].is_array());
        
        // Update KYC status
        let (status, response) = ctx.make_request(
            Method::PUT,
            &format!("/api/v1/admin/compliance/kyc/merchant/{}", merchant_id),
            &ctx.super_admin_api_key,
            Some(json!({
                "status": "VERIFIED",
                "verification_level": "ENHANCED",
                "notes": "All documents verified successfully",
                "verified_by": "admin@fiddupay.com"
            }))
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["status"], "VERIFIED");
        
        // List pending KYC reviews
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/compliance/kyc/pending",
            &ctx.admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["pending_reviews"].is_array());
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_transaction_monitoring() {
        let ctx = AdminTestContext::new().await;
        
        // Get suspicious transaction alerts
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/compliance/transactions/suspicious",
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["suspicious_transactions"].is_array());
        assert!(response["risk_scores"].is_object());
        
        // Review suspicious transaction
        let (status, response) = ctx.make_request(
            Method::POST,
            "/api/v1/admin/compliance/transactions/review",
            &ctx.super_admin_api_key,
            Some(json!({
                "transaction_id": "test_tx_123",
                "review_status": "CLEARED",
                "notes": "Transaction appears legitimate after review",
                "reviewer": "compliance_officer@fiddupay.com"
            }))
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["review_status"], "CLEARED");
        
        // Get transaction risk analysis
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/compliance/transactions/risk-analysis?period=30d",
            &ctx.admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["risk_distribution"].is_object());
        assert!(response["flagged_patterns"].is_array());
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_regulatory_reporting() {
        let ctx = AdminTestContext::new().await;
        
        // Generate regulatory report
        let (status, response) = ctx.make_request(
            Method::POST,
            "/api/v1/admin/compliance/reports/generate",
            &ctx.super_admin_api_key,
            Some(json!({
                "report_type": "SAR", // Suspicious Activity Report
                "period": {
                    "from": "2024-01-01T00:00:00Z",
                    "to": "2024-03-31T23:59:59Z"
                },
                "jurisdiction": "US",
                "include_sections": [
                    "suspicious_transactions",
                    "merchant_activities",
                    "risk_assessments"
                ]
            }))
        ).await;
        assert_eq!(status, StatusCode::ACCEPTED);
        let report_id = response["report_id"].as_str().unwrap();
        
        // Check report generation status
        let (status, response) = ctx.make_request(
            Method::GET,
            &format!("/api/v1/admin/compliance/reports/{}/status", report_id),
            &ctx.admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["status"].is_string());
        
        // List available compliance reports
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/compliance/reports",
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["reports"].is_array());
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_security_incident_management() {
        let ctx = AdminTestContext::new().await;
        
        // Create security incident
        let (status, response) = ctx.make_request(
            Method::POST,
            "/api/v1/admin/security/incidents",
            &ctx.super_admin_api_key,
            Some(json!({
                "title": "Suspicious API Activity Detected",
                "description": "Unusual API request patterns from merchant account",
                "severity": "HIGH",
                "category": "API_ABUSE",
                "affected_merchant_id": 12345,
                "initial_response": "Account temporarily suspended pending investigation"
            }))
        ).await;
        assert_eq!(status, StatusCode::CREATED);
        let incident_id = response["incident_id"].as_str().unwrap();
        
        // Update incident status
        let (status, response) = ctx.make_request(
            Method::PUT,
            &format!("/api/v1/admin/security/incidents/{}", incident_id),
            &ctx.super_admin_api_key,
            Some(json!({
                "status": "INVESTIGATING",
                "assigned_to": "security_team@fiddupay.com",
                "notes": "Initial investigation shows potential false positive"
            }))
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["status"], "INVESTIGATING");
        
        // List security incidents
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/security/incidents?status=OPEN&severity=HIGH",
            &ctx.admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["incidents"].is_array());
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_access_control_management() {
        let ctx = AdminTestContext::new().await;
        
        // Get role permissions matrix
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/security/permissions",
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["roles"].is_object());
        assert!(response["permissions"].is_array());
        
        // Update role permissions (super admin only)
        let (status, response) = ctx.make_request(
            Method::PUT,
            "/api/v1/admin/security/permissions/role/ADMIN",
            &ctx.super_admin_api_key,
            Some(json!({
                "permissions": [
                    "READ_MERCHANTS",
                    "READ_ANALYTICS",
                    "READ_AUDIT_LOGS",
                    "MANAGE_ALERTS"
                ]
            }))
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["role"], "ADMIN");
        
        // Admin cannot modify permissions
        let (status, _) = ctx.make_request(
            Method::PUT,
            "/api/v1/admin/security/permissions/role/MERCHANT",
            &ctx.admin_api_key,
            Some(json!({"permissions": ["READ_PAYMENTS"]}))
        ).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        
        ctx.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_data_retention_management() {
        let ctx = AdminTestContext::new().await;
        
        // Get data retention policies
        let (status, response) = ctx.make_request(
            Method::GET,
            "/api/v1/admin/compliance/data-retention",
            &ctx.super_admin_api_key,
            None
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response["policies"].is_object());
        assert!(response["scheduled_deletions"].is_array());
        
        // Update retention policy
        let (status, response) = ctx.make_request(
            Method::PUT,
            "/api/v1/admin/compliance/data-retention/policy",
            &ctx.super_admin_api_key,
            Some(json!({
                "data_type": "AUDIT_LOGS",
                "retention_period_days": 2555, // 7 years
                "auto_delete": true,
                "archive_before_delete": true
            }))
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["retention_period_days"], 2555);
        
        // Schedule data deletion
        let (status, response) = ctx.make_request(
            Method::POST,
            "/api/v1/admin/compliance/data-retention/schedule-deletion",
            &ctx.super_admin_api_key,
            Some(json!({
                "data_type": "EXPIRED_PAYMENTS",
                "criteria": {
                    "older_than_days": 365,
                    "status": "EXPIRED"
                },
                "scheduled_for": "2024-12-31T02:00:00Z"
            }))
        ).await;
        assert_eq!(status, StatusCode::ACCEPTED);
        assert!(response["deletion_job_id"].is_string());
        
        ctx.cleanup().await;
    }
    
    async fn create_test_merchant(db_pool: &PgPool) -> i64 {
        let result = sqlx::query!(
            r#"
            INSERT INTO merchants (email, business_name, api_key_hash, role, is_active, sandbox_mode, created_at, updated_at)
            VALUES ($1, $2, $3, $4::user_role, $5, $6, NOW(), NOW())
            RETURNING id
            "#,
            "compliance_test@example.com",
            "Compliance Test Merchant",
            "test_hash_compliance",
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