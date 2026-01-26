// Address-Only Mode Startup Service
// Initializes and coordinates all address-only components

use crate::config::Config;
use crate::error::ServiceError;
use crate::services::{
    address_only_service::AddressOnlyService,
    gas_fee_service::GasFeeService,
    payment_monitor_service::PaymentMonitorService,
    webhook_notification_service::WebhookNotificationService,
};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::task::JoinHandle;

pub struct AddressOnlyManager {
    pub address_service: Arc<AddressOnlyService>,
    pub monitor_service: Arc<PaymentMonitorService>,
    pub webhook_service: Arc<WebhookNotificationService>,
    db_pool: PgPool,
    monitor_handle: Option<JoinHandle<()>>,
}

impl AddressOnlyManager {
    /// Initialize address-only mode with all components
    pub async fn new(db_pool: PgPool, config: Config) -> Result<Self, ServiceError> {
        // Initialize services
        let gas_service = GasFeeService::new(config.clone());
        let address_service = Arc::new(AddressOnlyService::new(
            db_pool.clone(),
            gas_service,
            config.clone(),
        ));
        let webhook_service = Arc::new(WebhookNotificationService::new(db_pool.clone()));
        let monitor_service = Arc::new(PaymentMonitorService::new(
            db_pool.clone(),
            (*address_service).clone(),
            config.clone(),
        ));

        Ok(Self {
            address_service,
            monitor_service,
            webhook_service,
            db_pool,
            monitor_handle: None,
        })
    }

    /// Start payment monitoring in background
    pub async fn start_monitoring(&mut self) -> Result<(), ServiceError> {
        let monitor_service = Arc::clone(&self.monitor_service);
        
        let handle = tokio::spawn(async move {
            if let Err(e) = monitor_service.start_monitoring().await {
                tracing::error!("Payment monitoring failed: {}", e);
            }
        });

        self.monitor_handle = Some(handle);
        tracing::info!("Address-only payment monitoring started");
        
        Ok(())
    }

    /// Stop payment monitoring
    pub fn stop_monitoring(&mut self) {
        if let Some(handle) = self.monitor_handle.take() {
            handle.abort();
            tracing::info!("Address-only payment monitoring stopped");
        }
    }

    /// Get service for API usage
    pub fn get_address_service(&self) -> Arc<AddressOnlyService> {
        Arc::clone(&self.address_service)
    }

    /// Get webhook service for notifications
    pub fn get_webhook_service(&self) -> Arc<WebhookNotificationService> {
        Arc::clone(&self.webhook_service)
    }

    /// Health check for all components
    pub async fn health_check(&self) -> Result<AddressOnlyHealthStatus, ServiceError> {
        // Check database connectivity
        let db_healthy = self.check_database_health().await?;
        
        // Check monitoring status
        let monitoring_active = self.monitor_handle.as_ref()
            .map(|h| !h.is_finished())
            .unwrap_or(false);

        Ok(AddressOnlyHealthStatus {
            database_healthy: db_healthy,
            monitoring_active,
            supported_currencies: vec![
                "ETH".to_string(),
                "BNB".to_string(),
                "MATIC".to_string(),
                "ARB".to_string(),
                "SOL".to_string(),
            ],
        })
    }

    async fn check_database_health(&self) -> Result<bool, ServiceError> {
        // Simple query to check database connectivity using a test connection
        let result = sqlx::query!("SELECT 1 as health_check")
            .fetch_one(&self.db_pool)
            .await;

        Ok(result.is_ok())
    }
}

#[derive(Debug, serde::Serialize)]
pub struct AddressOnlyHealthStatus {
    pub database_healthy: bool,
    pub monitoring_active: bool,
    pub supported_currencies: Vec<String>,
}

impl Drop for AddressOnlyManager {
    fn drop(&mut self) {
        self.stop_monitoring();
    }
}
