// API Routes
// HTTP route definitions

use crate::api::{handlers, admin_handlers, wallet_management, security_monitoring, status, blog, careers};
use crate::api::state::AppState;
use crate::api::middleware::{create_rate_limit_layer, rate_limit_middleware};
use crate::api::{merchant_routes, admin_routes};
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
use tower_http::trace::TraceLayer;

pub fn create_router(state: AppState) -> Router {
    // Create rate limiter
    let rate_limiter = create_rate_limit_layer(&state.config);

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/", get(handlers::root_handler))
        .route("/health", get(handlers::health_check))
        // .route("/test-auth/:api_key", get(handlers::debug_auth)) // DEBUG ENDPOINT - REMOVED FOR SECURITY
        .route("/:link_id", get(handlers::payment_page))
        .route("/:link_id/status", get(handlers::payment_status))
        .route("/:link_id/select", post(handlers::finalize_payment_selection))
        .route("/api/v1/merchants/register", post(handlers::register_merchant))
        .route("/api/v1/merchants/login", post(handlers::login_merchant))
        .route("/api/v1/currencies/supported", get(handlers::get_supported_currencies));

    // Additional public routes
    let additional_public_routes = Router::new()
        .route("/api/v1/status", get(status::get_system_status))
        .route("/api/v1/blog", get(blog::get_blog_posts))
        .route("/api/v1/careers", get(careers::get_careers))
        .route("/api/v1/contact", post(handlers::submit_contact_form))
        .route("/api/v1/pricing", get(handlers::get_pricing_info));

    // Create modular routers
    let merchant_router = merchant_routes::create_merchant_router(state.clone());
    let admin_router = admin_routes::create_admin_router(state.clone());

    // Combine routes with CORS
    let cors = CorsLayer::new()
        .allow_origin(
            state.config.allowed_origins
                .iter()
                .map(|origin| origin.parse::<HeaderValue>().unwrap())
                .collect::<Vec<HeaderValue>>()
        )
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE, Method::OPTIONS])
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_credentials(true);

    public_routes
        .merge(additional_public_routes)
        .merge(merchant_router)
        .merge(admin_router)
        // Apply global rate limiting to all routes
        .layer(axum_middleware::from_fn_with_state(rate_limiter, rate_limit_middleware))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
