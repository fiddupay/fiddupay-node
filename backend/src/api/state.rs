// API State
// Shared application state

use crate::config::Config;
use crate::services::{
    account_lockout_service::AccountLockoutService, admin_service::AdminService,
    analytics_service::AnalyticsService, audit_service::AuditService,
    balance_monitor::BalanceMonitor, balance_service::BalanceService,
    blockchain_transaction_sender::BlockchainTransactionSender, currency_service::CurrencyService,
    invoice_service::InvoiceService, ip_whitelist_service::IpWhitelistService,
    merchant_customer_service::MerchantCustomerService, merchant_service::MerchantService,
    monitoring_service::MonitoringService, notification_service::NotificationService,
    p2p_service::P2pService, payment_service::PaymentService, price_service::PriceService,
    refund_service::RefundService, report_service::ReportService,
    security_monitoring_service::SecurityMonitoringService,
    volume_tracking_service::VolumeTrackingService, wallet_config_service::WalletConfigService,
    webhook_notification_service::WebhookNotificationService, webhook_service::WebhookService,
    withdrawal_service::WithdrawalService,
};
use redis::Client as RedisClient;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub config: Config,
    pub merchant_service: Arc<MerchantService>,
    pub merchant_customer_service: Arc<MerchantCustomerService>,
    pub payment_service: Arc<PaymentService>,
    pub refund_service: Arc<RefundService>,
    pub analytics_service: Arc<AnalyticsService>,
    pub admin_service: Arc<AdminService>,
    pub webhook_service: Arc<WebhookService>,
    pub ip_whitelist_service: Arc<IpWhitelistService>,
    pub audit_service: Arc<AuditService>,
    pub balance_service: Arc<BalanceService>,
    pub withdrawal_service: Arc<WithdrawalService>,
    pub wallet_config_service: Arc<WalletConfigService>,
    pub currency_service: Arc<CurrencyService>,
    pub price_service: Arc<PriceService>,
    pub volume_tracking_service: Arc<VolumeTrackingService>,
    pub invoice_service: Arc<InvoiceService>,
    pub p2p_service: Arc<P2pService>,
    pub report_service: Arc<ReportService>,
    pub notification_service: Arc<NotificationService>,
    pub monitoring_service: Arc<MonitoringService>,
    pub balance_monitor: Arc<BalanceMonitor>,
    pub account_lockout_service: Arc<AccountLockoutService>,
    pub security_monitoring_service: Arc<SecurityMonitoringService>,
    pub blockchain_sender: Arc<BlockchainTransactionSender>,
    pub redis_client: RedisClient,
}

impl AppState {
    pub fn new(db_pool: PgPool, config: Config, redis_client: RedisClient) -> Self {
        let webhook_service = Arc::new(WebhookService::new(
            db_pool.clone(),
            config.webhook_signing_key.clone(),
        ));

        let price_service = Arc::new(PriceService::new(config.clone()));
        price_service.start_background_polling();

        let invoice_service = Arc::new(InvoiceService::new(db_pool.clone()));
        let balance_service = Arc::new(BalanceService::new(
            db_pool.clone(),
            price_service.clone(),
            redis_client.clone(),
        ));

        let audit_service = Arc::new(AuditService::new(db_pool.clone()));
        let volume_tracking_service = Arc::new(VolumeTrackingService::new(db_pool.clone()));
        let merchant_service = Arc::new(MerchantService::new(
            db_pool.clone(),
            config.clone(),
            audit_service.clone(),
            volume_tracking_service.clone(),
        ));

        let notification_service = Arc::new(NotificationService::new(db_pool.clone()));

        let merchant_customer_service = Arc::new(
            crate::services::merchant_customer_service::MerchantCustomerService::new(
                db_pool.clone(),
                price_service.clone(),
                volume_tracking_service.clone(),
                notification_service.clone(),
                balance_service.clone(),
                Arc::new(config.clone()),
            ),
        );

        let monitoring_service = Arc::new(MonitoringService::new(
            db_pool.clone(),
            config.clone(),
            redis_client.clone(),
        ));
        monitoring_service.clone().start_polling();

        let blockchain_sender = Arc::new(BlockchainTransactionSender::new(config.clone()));
        let webhook_notif = Arc::new(WebhookNotificationService::new(db_pool.clone()));
        let balance_monitor = Arc::new(BalanceMonitor::new(
            db_pool.clone(),
            blockchain_sender.clone(),
            price_service.clone(),
            notification_service.clone(),
            webhook_notif,
            balance_service.clone(),
        ));

        Self {
            merchant_service,
            payment_service: Arc::new(PaymentService::new(
                crate::services::payment_service::PaymentServiceConfig {
                    db_pool: db_pool.clone(),
                    payment_page_base_url: config.payment_page_base_url.clone(),
                    price_service: price_service.clone(),
                    invoice_service: invoice_service.clone(),
                    audit_service: audit_service.clone(),
                    webhook_signing_key: config.webhook_signing_key.clone(),
                    config: config.clone(),
                    redis_client: redis_client.clone(),
                    volume_tracking: volume_tracking_service.clone(),
                    notification_service: notification_service.clone(),
                    balance_service: balance_service.clone(),
                },
            )),
            refund_service: Arc::new(RefundService::new(db_pool.clone(), webhook_service.clone())),
            analytics_service: Arc::new(AnalyticsService::new(
                db_pool.clone(),
                price_service.clone(),
            )),
            admin_service: Arc::new(AdminService::new(db_pool.clone())),
            webhook_service: webhook_service.clone(),
            ip_whitelist_service: Arc::new(IpWhitelistService::new(db_pool.clone())),
            audit_service,
            balance_service: balance_service.clone(),
            withdrawal_service: Arc::new(WithdrawalService::new(
                db_pool.clone(),
                price_service.clone(),
                volume_tracking_service.clone(),
                config.clone(),
            )),
            wallet_config_service: Arc::new(WalletConfigService::new(db_pool.clone())),
            currency_service: Arc::new(CurrencyService::new(db_pool.clone())),
            price_service,
            volume_tracking_service,
            merchant_customer_service,
            invoice_service,
            p2p_service: Arc::new(P2pService::new(db_pool.clone())),
            report_service: Arc::new(ReportService::new(db_pool.clone())),
            notification_service,
            monitoring_service,
            balance_monitor,
            config,
            db_pool: db_pool.clone(),
            account_lockout_service: Arc::new(AccountLockoutService::new(db_pool.clone(), 5, 15)),
            security_monitoring_service: Arc::new(SecurityMonitoringService::new(db_pool.clone())),
            blockchain_sender: blockchain_sender.clone(),
            redis_client,
        }
    }
}
