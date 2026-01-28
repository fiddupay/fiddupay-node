// API Routes
// HTTP route definitions

use crate::api::{handlers, merchant_handlers, merchant_routes, admin_routes, status, blog, careers};
use crate::api::state::AppState;
use crate::middleware::{auth, ip_whitelist, logging, rate_limit};
use axum::{
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    middleware as axum_middleware,
    routing::{get, post, put, delete},
    Router,
};
use tower_http::cors::CorsLayer;

pub fn create_router(state: AppState) -> Router {
    // Create rate limiter with config
    let rate_limiter = rate_limit::create_rate_limiter(state.config.rate_limit_requests_per_minute);

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/pay/:link_id", get(handlers::payment_page))
        .route("/pay/:link_id/status", get(handlers::payment_status))
        .route("/api/v1/merchant/register", post(merchant_handlers::register_merchant))
        .route("/api/v1/merchant/login", post(merchant_handlers::login_merchant))
        .route("/api/v1/currencies/supported", get(handlers::get_supported_currencies))
        .route("/api/v1/status", get(status::get_system_status))
        .route("/api/v1/blog", get(blog::get_blog_posts))
        .route("/api/v1/careers", get(careers::get_careers))
        .route("/api/v1/contact", post(handlers::submit_contact_form))
        .route("/api/v1/pricing", get(handlers::get_pricing_info));

    // Combine all routers with CORS
    let cors = CorsLayer::new()
        .allow_origin(
            std::env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string())
                .parse::<HeaderValue>()
                .unwrap()
        )
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_credentials(true);

    public_routes
        .merge(merchant_routes::create_merchant_router(state.clone()))
        .merge(admin_routes::create_admin_router(state.clone()))
        .layer(cors)
        .with_state(state)
}
