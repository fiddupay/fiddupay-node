// Crypto Payment Gateway Service
// Main entry point

use fiddupay::{
    api::{routes, state::AppState},
    background_tasks::BackgroundTasks,
    config::Config,
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "crypto_payment_gateway=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!(" Starting Crypto Payment Gateway Service");

    // Load configuration
    let config = Config::from_env()?;
    config.validate()?;
    tracing::info!(" Configuration loaded");

    // Initialize database connection pool using config values
    tracing::info!("📦 Connecting to database...");
    let db_pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .min_connections(5)  // Keep some connections warm
        .acquire_timeout(std::time::Duration::from_secs(config.database_timeout_seconds as u64))
        .idle_timeout(std::time::Duration::from_secs(600))
        .max_lifetime(std::time::Duration::from_secs(3600))
        .test_before_acquire(false)  // Disable for performance
        .connect(&config.database_url)
        .await?;
    tracing::info!(" Database pool connected");

    // Run migrations (unless skipped)
    if std::env::var("SKIP_MIGRATIONS").unwrap_or_default() != "true" {
        tracing::info!("🔄 Running database migrations...");
        sqlx::migrate!("./migrations")
            .run(&db_pool)
            .await?;
        tracing::info!("✅ Migrations complete");
    } else {
        tracing::info!("⏭️ Skipping database migrations (SKIP_MIGRATIONS=true)");
    }

    // Initialize application state
    let app_state = AppState::new(
        db_pool.clone(),
        config.clone(),
    );
    tracing::info!(" Application state initialized");

    // Start background tasks
    tracing::info!(" Starting background tasks...");
    let background_tasks = Arc::new(BackgroundTasks::new(
        db_pool.clone(),
        config.webhook_signing_key.clone(),
    ));
    background_tasks.start();
    tracing::info!(" Background tasks started");

    // Create router
    let app = routes::create_router(app_state);

    // Start HTTP server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!(" Starting HTTP server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(" Server listening on http://{}", addr);
    tracing::info!(" Health check: http://{}/health", addr);
    tracing::info!("📄 API endpoints: http://{}/api/v1/*", addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
