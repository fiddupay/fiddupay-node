// API Routes
// HTTP route definitions

use crate::api::{public_handlers, merchant_auth_handlers, payment_handlers, wallet_management, security_monitoring, status, blog, careers};
use crate::api::state::AppState;
use crate::api::middleware::{create_rate_limit_layer, rate_limit_middleware};
use crate::api::{merchant_routes, admin_routes, p2p_routes};
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
        .route("/", get(public_handlers::root_handler))
        .route("/health", get(public_handlers::health_check))
        .route("/:link_id", get(payment_handlers::payment_page))
        .route("/:link_id/status", get(payment_handlers::payment_status))
        .route("/:link_id/select", post(payment_handlers::finalize_payment_selection))
        .route("/api/v1/merchants/register", post(merchant_auth_handlers::register_merchant))
        .route("/api/v1/merchants/login", post(merchant_auth_handlers::login_merchant))
        .route("/api/v1/p2p/register", post(crate::api::p2p_auth_handlers::register_p2p_user))
        .route("/api/v1/currencies/supported", get(public_handlers::get_supported_currencies))
        .route("/:payment_id/cancel", post(public_handlers::public_cancel_payment));

    // Additional public routes
    let additional_public_routes = Router::new()
        .route("/api/v1/status", get(status::get_system_status))
        .route("/api/v1/blog", get(blog::get_blog_posts))
        .route("/api/v1/careers", get(careers::get_careers))
        .route("/api/v1/contact", post(public_handlers::submit_contact_form))
        .route("/api/v1/pricing", get(public_handlers::get_pricing_info));

    // Create modular routers
    let merchant_router = merchant_routes::create_merchant_router(state.clone());
    let admin_router = admin_routes::create_admin_router(state.clone());
    let p2p_router = p2p_routes::create_p2p_router(state.clone());

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
        .merge(p2p_router)
        // Apply global rate limiting to all routes
        .layer(axum_middleware::from_fn_with_state(rate_limiter, rate_limit_middleware))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
