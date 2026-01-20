// Unit tests for payment listing and filtering functionality
// Tests for task 6.7: Implement payment listing and filtering

#[cfg(test)]
mod payment_listing_tests {
    use chrono::{Duration, Utc};
    use crypto_payment_gateway::payment::models::{
        PaymentFilters, PaymentList, PaymentStatus,
    };
    use crypto_payment_gateway::services::payment_service::PaymentService;
    use rust_decimal::Decimal;
    use sqlx::PgPool;

    // Helper function to setup test database
    async fn setup_test_db() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost/crypto_gateway_test".to_string());
        
        PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    // Helper function to create test merchant
    async fn create_test_merchant(pool: &PgPool) -> i64 {
        let result = sqlx::query!(
            "INSERT INTO merchants (email, business_name, api_key_hash, fee_percentage) 
             VALUES ($1, $2, $3, $4) RETURNING id",
            "test@example.com",
            "Test Business",
            "test_hash",
            Decimal::new(150, 2)
        )
        .fetch_one(pool)
        .await
        .expect("Failed to create test merchant");

        result.id
    }

    // Helper function to create test payment
    async fn create_test_payment(
        pool: &PgPool,
        merchant_id: i64,
        status: &str,
        network: &str,
        created_at: chrono::DateTime<Utc>,
    ) -> i64 {
        let payment_id = format!("pay_test_{}", uuid::Uuid::new_v4());
        let expires_at = created_at + Duration::minutes(15);

        let result = sqlx::query!(
            r#"
            INSERT INTO payment_transactions (
                merchant_id, payment_id, amount, amount_usd, fee_percentage,
                fee_amount, fee_amount_usd, crypto_type, network, to_address,
                status, confirmations, required_confirmations, partial_payments_enabled,
                total_paid, created_at, expires_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            RETURNING id
            "#,
            merchant_id,
            payment_id,
            Decimal::new(100, 0),
            Decimal::new(10000, 2),
            Decimal::new(150, 2),
            Decimal::new(150, 2),
            Decimal::new(150, 2),
            "USDT_BEP20",
            network,
            "0x123456789",
            status,
            0,
            15,
            false,
            Decimal::ZERO,
            created_at,
            expires_at
        )
        .fetch_one(pool)
        .await
        .expect("Failed to create test payment");

        result.id
    }

    #[tokio::test]
    async fn test_list_payments_default_pagination() {
        let pool = setup_test_db().await;
        let merchant_id = create_test_merchant(&pool).await;
        let service = PaymentService::new(pool.clone());

        // Create 5 test payments
        for i in 0..5 {
            let created_at = Utc::now() - Duration::hours(i);
            create_test_payment(&pool, merchant_id, "PENDING", "BEP20", created_at).await;
        }

        // List payments with default filters
        let filters = PaymentFilters::default();
        let result = service.list_payments(merchant_id, filters).await;

        assert!(result.is_ok());
        let payment_list = result.unwrap();
        assert_eq!(payment_list.payments.len(), 5);
        assert_eq!(payment_list.total, 5);
        assert_eq!(payment_list.page, 1);
        assert_eq!(payment_list.page_size, 20);
        assert_eq!(payment_list.total_pages, 1);
    }

    #[tokio::test]
    async fn test_list_payments_with_status_filter() {
        let pool = setup_test_db().await;
        let merchant_id = create_test_merchant(&pool).await;
        let service = PaymentService::new(pool.clone());

        // Create payments with different statuses
        create_test_payment(&pool, merchant_id, "PENDING", "BEP20", Utc::now()).await;
        create_test_payment(&pool, merchant_id, "PENDING", "BEP20", Utc::now()).await;
        create_test_payment(&pool, merchant_id, "CONFIRMED", "BEP20", Utc::now()).await;
        create_test_payment(&pool, merchant_id, "FAILED", "BEP20", Utc::now()).await;

        // Filter by PENDING status
        let filters = PaymentFilters {
            status: Some(PaymentStatus::Pending),
            ..Default::default()
        };
        let result = service.list_payments(merchant_id, filters).await;

        assert!(result.is_ok());
        let payment_list = result.unwrap();
        assert_eq!(payment_list.payments.len(), 2);
        assert_eq!(payment_list.total, 2);
        
        // Verify all returned payments have PENDING status
        for payment in payment_list.payments {
            assert_eq!(payment.status, PaymentStatus::Pending);
        }
    }

    #[tokio::test]
    async fn test_list_payments_with_blockchain_filter() {
        let pool = setup_test_db().await;
        let merchant_id = create_test_merchant(&pool).await;
        let service = PaymentService::new(pool.clone());

        // Create payments on different blockchains
        create_test_payment(&pool, merchant_id, "PENDING", "BEP20", Utc::now()).await;
        create_test_payment(&pool, merchant_id, "PENDING", "BEP20", Utc::now()).await;
        create_test_payment(&pool, merchant_id, "PENDING", "SOLANA", Utc::now()).await;
        create_test_payment(&pool, merchant_id, "PENDING", "ARBITRUM", Utc::now()).await;

        // Filter by BEP20 blockchain
        let filters = PaymentFilters {
            blockchain: Some("BEP20".to_string()),
            ..Default::default()
        };
        let result = service.list_payments(merchant_id, filters).await;

        assert!(result.is_ok());
        let payment_list = result.unwrap();
        assert_eq!(payment_list.payments.len(), 2);
        assert_eq!(payment_list.total, 2);
        
        // Verify all returned payments are on BEP20
        for payment in payment_list.payments {
            assert_eq!(payment.network, "BEP20");
        }
    }

    #[tokio::test]
    async fn test_list_payments_with_date_range_filter() {
        let pool = setup_test_db().await;
        let merchant_id = create_test_merchant(&pool).await;
        let service = PaymentService::new(pool.clone());

        let now = Utc::now();
        let yesterday = now - Duration::days(1);
        let two_days_ago = now - Duration::days(2);
        let three_days_ago = now - Duration::days(3);

        // Create payments at different times
        create_test_payment(&pool, merchant_id, "PENDING", "BEP20", now).await;
        create_test_payment(&pool, merchant_id, "PENDING", "BEP20", yesterday).await;
        create_test_payment(&pool, merchant_id, "PENDING", "BEP20", two_days_ago).await;
        create_test_payment(&pool, merchant_id, "PENDING", "BEP20", three_days_ago).await;

        // Filter by date range (last 2 days)
        let filters = PaymentFilters {
            from_date: Some(two_days_ago),
            to_date: Some(now),
            ..Default::default()
        };
        let result = service.list_payments(merchant_id, filters).await;

        assert!(result.is_ok());
        let payment_list = result.unwrap();
        assert_eq!(payment_list.payments.len(), 3); // now, yesterday, two_days_ago
        assert_eq!(payment_list.total, 3);
    }

    #[tokio::test]
    async fn test_list_payments_with_pagination() {
        let pool = setup_test_db().await;
        let merchant_id = create_test_merchant(&pool).await;
        let service = PaymentService::new(pool.clone());

        // Create 25 test payments
        for i in 0..25 {
            let created_at = Utc::now() - Duration::hours(i);
            create_test_payment(&pool, merchant_id, "PENDING", "BEP20", created_at).await;
        }

        // Get first page (10 items per page)
        let filters = PaymentFilters {
            page: Some(1),
            page_size: Some(10),
            ..Default::default()
        };
        let result = service.list_payments(merchant_id, filters).await;

        assert!(result.is_ok());
        let payment_list = result.unwrap();
        assert_eq!(payment_list.payments.len(), 10);
        assert_eq!(payment_list.total, 25);
        assert_eq!(payment_list.page, 1);
        assert_eq!(payment_list.page_size, 10);
        assert_eq!(payment_list.total_pages, 3); // 25 / 10 = 2.5, ceil = 3

        // Get second page
        let filters = PaymentFilters {
            page: Some(2),
            page_size: Some(10),
            ..Default::default()
        };
        let result = service.list_payments(merchant_id, filters).await;

        assert!(result.is_ok());
        let payment_list = result.unwrap();
        assert_eq!(payment_list.payments.len(), 10);
        assert_eq!(payment_list.page, 2);

        // Get third page (only 5 items)
        let filters = PaymentFilters {
            page: Some(3),
            page_size: Some(10),
            ..Default::default()
        };
        let result = service.list_payments(merchant_id, filters).await;

        assert!(result.is_ok());
        let payment_list = result.unwrap();
        assert_eq!(payment_list.payments.len(), 5);
        assert_eq!(payment_list.page, 3);
    }

    #[tokio::test]
    async fn test_list_payments_with_combined_filters() {
        let pool = setup_test_db().await;
        let merchant_id = create_test_merchant(&pool).await;
        let service = PaymentService::new(pool.clone());

        let now = Utc::now();
        let yesterday = now - Duration::days(1);

        // Create various payments
        create_test_payment(&pool, merchant_id, "CONFIRMED", "BEP20", now).await;
        create_test_payment(&pool, merchant_id, "CONFIRMED", "BEP20", yesterday).await;
        create_test_payment(&pool, merchant_id, "PENDING", "BEP20", now).await;
        create_test_payment(&pool, merchant_id, "CONFIRMED", "SOLANA", now).await;

        // Filter by status=CONFIRMED, blockchain=BEP20, from yesterday
        let filters = PaymentFilters {
            status: Some(PaymentStatus::Confirmed),
            blockchain: Some("BEP20".to_string()),
            from_date: Some(yesterday),
            ..Default::default()
        };
        let result = service.list_payments(merchant_id, filters).await;

        assert!(result.is_ok());
        let payment_list = result.unwrap();
        assert_eq!(payment_list.payments.len(), 2); // Both CONFIRMED BEP20 payments
        assert_eq!(payment_list.total, 2);
        
        // Verify all returned payments match filters
        for payment in payment_list.payments {
            assert_eq!(payment.status, PaymentStatus::Confirmed);
            assert_eq!(payment.network, "BEP20");
        }
    }

    #[tokio::test]
    async fn test_list_payments_max_page_size() {
        let pool = setup_test_db().await;
        let merchant_id = create_test_merchant(&pool).await;
        let service = PaymentService::new(pool.clone());

        // Create 150 test payments
        for i in 0..150 {
            let created_at = Utc::now() - Duration::hours(i);
            create_test_payment(&pool, merchant_id, "PENDING", "BEP20", created_at).await;
        }

        // Try to request 200 items per page (should be capped at 100)
        let filters = PaymentFilters {
            page: Some(1),
            page_size: Some(200),
            ..Default::default()
        };
        let result = service.list_payments(merchant_id, filters).await;

        assert!(result.is_ok());
        let payment_list = result.unwrap();
        assert_eq!(payment_list.page_size, 100); // Capped at max
        assert_eq!(payment_list.payments.len(), 100);
    }

    #[tokio::test]
    async fn test_list_payments_empty_result() {
        let pool = setup_test_db().await;
        let merchant_id = create_test_merchant(&pool).await;
        let service = PaymentService::new(pool.clone());

        // Don't create any payments

        // List payments
        let filters = PaymentFilters::default();
        let result = service.list_payments(merchant_id, filters).await;

        assert!(result.is_ok());
        let payment_list = result.unwrap();
        assert_eq!(payment_list.payments.len(), 0);
        assert_eq!(payment_list.total, 0);
        assert_eq!(payment_list.total_pages, 0);
    }

    #[tokio::test]
    async fn test_list_payments_ordering() {
        let pool = setup_test_db().await;
        let merchant_id = create_test_merchant(&pool).await;
        let service = PaymentService::new(pool.clone());

        let now = Utc::now();
        
        // Create payments in specific order
        let payment1_time = now - Duration::hours(3);
        let payment2_time = now - Duration::hours(2);
        let payment3_time = now - Duration::hours(1);

        create_test_payment(&pool, merchant_id, "PENDING", "BEP20", payment1_time).await;
        create_test_payment(&pool, merchant_id, "PENDING", "BEP20", payment2_time).await;
        create_test_payment(&pool, merchant_id, "PENDING", "BEP20", payment3_time).await;

        // List payments (should be ordered by created_at DESC)
        let filters = PaymentFilters::default();
        let result = service.list_payments(merchant_id, filters).await;

        assert!(result.is_ok());
        let payment_list = result.unwrap();
        assert_eq!(payment_list.payments.len(), 3);
        
        // Verify ordering (newest first)
        assert!(payment_list.payments[0].created_at > payment_list.payments[1].created_at);
        assert!(payment_list.payments[1].created_at > payment_list.payments[2].created_at);
    }

    #[tokio::test]
    async fn test_list_payments_different_merchants() {
        let pool = setup_test_db().await;
        let merchant1_id = create_test_merchant(&pool).await;
        let merchant2_id = create_test_merchant(&pool).await;
        let service = PaymentService::new(pool.clone());

        // Create payments for both merchants
        create_test_payment(&pool, merchant1_id, "PENDING", "BEP20", Utc::now()).await;
        create_test_payment(&pool, merchant1_id, "PENDING", "BEP20", Utc::now()).await;
        create_test_payment(&pool, merchant2_id, "PENDING", "BEP20", Utc::now()).await;

        // List payments for merchant 1
        let filters = PaymentFilters::default();
        let result = service.list_payments(merchant1_id, filters).await;

        assert!(result.is_ok());
        let payment_list = result.unwrap();
        assert_eq!(payment_list.payments.len(), 2);
        assert_eq!(payment_list.total, 2);

        // List payments for merchant 2
        let filters = PaymentFilters::default();
        let result = service.list_payments(merchant2_id, filters).await;

        assert!(result.is_ok());
        let payment_list = result.unwrap();
        assert_eq!(payment_list.payments.len(), 1);
        assert_eq!(payment_list.total, 1);
    }
}
