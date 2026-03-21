// API State
// Shared application state

use crate::config::Config;
use crate::services::{
    analytics_service::AnalyticsService,
    merchant_service::MerchantService,
    payment_service::PaymentService,
    refund_service::RefundService,
    sandbox_service::SandboxService,
    admin_service::AdminService,
    webhook_service::WebhookService,
    ip_whitelist_service::IpWhitelistService,
    audit_service::AuditService,
    balance_service::BalanceService,
    withdrawal_service::WithdrawalService,
    wallet_config_service::WalletConfigService,
    currency_service::CurrencyService,
    price_service::PriceService,
    volume_tracking_service::VolumeTrackingService,
    invoice_service::InvoiceService,
    p2p_service::P2pService,
};
use sqlx::PgPool;
use std::sync::Arc;
use redis::Client as RedisClient;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub config: Config,
    pub merchant_service: Arc<MerchantService>,
    pub payment_service: Arc<PaymentService>,
    pub refund_service: Arc<RefundService>,
    pub analytics_service: Arc<AnalyticsService>,
    pub sandbox_service: Arc<SandboxService>,
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
    pub redis_client: RedisClient,
}

impl AppState {
    pub fn new(
        db_pool: PgPool,
        config: Config,
        redis_client: RedisClient,
    ) -> Self {
        let webhook_service = Arc::new(WebhookService::new(db_pool.clone(), config.webhook_signing_key.clone()));
        
        let price_service = Arc::new(PriceService::new(config.clone()));
        price_service.start_background_polling();
        
        let invoice_service = Arc::new(InvoiceService::new(db_pool.clone()));
        let balance_service = Arc::new(BalanceService::new(db_pool.clone(), price_service.clone()));

        let audit_service = Arc::new(AuditService::new(db_pool.clone()));
        let merchant_service = Arc::new(MerchantService::new(db_pool.clone(), config.clone(), audit_service.clone()));

        Self {
            merchant_service,
            payment_service: Arc::new(PaymentService::new(db_pool.clone(), &config.payment_page_base_url, price_service.clone(), invoice_service.clone(), audit_service.clone(), &config.webhook_signing_key, config.clone(), redis_client.clone())),
            refund_service: Arc::new(RefundService::new(db_pool.clone(), webhook_service.clone())),
            analytics_service: Arc::new(AnalyticsService::new(db_pool.clone(), price_service.clone())),
            sandbox_service: Arc::new(SandboxService::new(db_pool.clone(), config.clone())),
            admin_service: Arc::new(AdminService::new(db_pool.clone())),
            webhook_service: webhook_service.clone(),
            ip_whitelist_service: Arc::new(IpWhitelistService::new(db_pool.clone())),
            audit_service,
            balance_service: balance_service.clone(),
            withdrawal_service: Arc::new(WithdrawalService::new(db_pool.clone(), price_service.clone())),
            wallet_config_service: Arc::new(WalletConfigService::new(db_pool.clone())),
            currency_service: Arc::new(CurrencyService::new(db_pool.clone())),
            price_service,
            volume_tracking_service: Arc::new(VolumeTrackingService::new(db_pool.clone())),
            invoice_service,
            p2p_service: Arc::new(P2pService::new(db_pool.clone())),
            config,
            db_pool,
            redis_client,
        }
    }
}
