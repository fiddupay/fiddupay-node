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
                .unwrap_or_else(|_| "fiddupay=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!(" Starting Crypto Payment Gateway Service");

    // Load configuration
    let config = Config::from_env()?;
    config.validate()?;
    tracing::info!(" Configuration loaded");

    // Initialize database connection pool using config values
    tracing::info!(" Connecting to database...");
    let db_pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .min_connections(10) // Increase warm connections for high concurrency
        .acquire_timeout(std::time::Duration::from_secs(
            config.database_timeout_seconds,
        ))
        .idle_timeout(std::time::Duration::from_secs(120)) // Reduce idle timeout to cycle connections faster
        .max_lifetime(std::time::Duration::from_secs(1800)) // 30 minutes max life
        .test_before_acquire(false) // Disable test_before_acquire for better performance (rely on robust error handling)
        .connect(&config.database_url)
        .await?;
    tracing::info!(" Database pool connected");

    // Run migrations
    tracing::info!(" Running database migrations...");
    sqlx::migrate::Migrator::new(std::path::Path::new("./migrations"))
        .await?
        .run(&db_pool)
        .await?;
    tracing::info!(" Migrations complete");

    // Load dynamic settings from database
    tracing::info!(" Loading dynamic settings from database...");
    let mut config = config;
    if let Err(e) = config.load_from_db(&db_pool).await {
        tracing::error!("Failed to load dynamic settings: {}", e);
        // Continue with env defaults, but log error
    } else {
        tracing::info!(" Dynamic settings loaded");
    }

    // Sync platform fee wallets from environment
    tracing::info!(" Syncing platform fee wallets from configuration...");
    if let Err(e) = config.sync_fee_wallets(&db_pool).await {
        tracing::error!("Failed to sync platform fee wallets: {}", e);
    } else {
        tracing::info!(" Platform fee wallets synced");
    }

    // Initialize Redis connection
    tracing::info!(" Connecting to Redis...");
    let redis_client = match redis::Client::open(config.redis_url.clone()) {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Failed to initialize Redis client: {}", e);
            return Err(e.into());
        }
    };
    tracing::info!(" Redis client initialized");

    // Initialize application state
    let app_state = AppState::new(db_pool.clone(), config.clone(), redis_client);
    tracing::info!(" Application state initialized");

    // Start background tasks
    tracing::info!(" Starting background tasks...");
    let background_tasks = Arc::new(BackgroundTasks::new(
        db_pool.clone(),
        config.clone(),
        app_state.price_service.clone(),
        app_state.redis_client.clone(),
        app_state.notification_service.clone(),
        app_state.blockchain_sender.clone(),
    ));
    background_tasks.start();
    tracing::info!(" Background tasks started");

    // Create router
    let app = routes::create_router(app_state);

    // Start HTTP server with graceful shutdown
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!(" Starting HTTP server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(" Server listening on http://{}", addr);
    tracing::info!(" Health check: http://{}/health", addr);
    tracing::info!(" API endpoints: http://{}/api/v1/*", addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    tracing::info!("Server shut down gracefully");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { tracing::info!("Received Ctrl+C, starting graceful shutdown..."); },
        _ = terminate => { tracing::info!("Received SIGTERM, starting graceful shutdown..."); },
    }
}
