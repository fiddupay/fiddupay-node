// Rate Limiting Middleware
// Implements connection-key based rate limiting using governor

use crate::config::Config;
use std::sync::Arc;
use tower_governor::{governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer};

pub fn create_rate_limit_layer(config: &Config) -> GovernorLayer<SmartIpKeyExtractor, Box<dyn tower_governor::errors::DisplayError>> {
    // Configure rate limiting based on config values
    // Requests per minute is converted to requests per second period for simple configuration
    let requests_per_second = if config.rate_limit_requests_per_minute > 0 {
        config.rate_limit_requests_per_minute as u64 / 60
    } else {
        1 // Fallback to 1 request per second if 0
    };

    // Build the configuration
    let gov_conf = GovernorConfigBuilder::default()
        .per_second(requests_per_second)
        .burst_size(config.rate_limit_burst_size)
        // Extract key from IP header (SmartIpKeyExtractor handles X-Forwarded-For etc)
        .key_extractor(SmartIpKeyExtractor)
        .finish()
        .unwrap();

    GovernorLayer {
        config: Box::new(gov_conf),
    }
}
