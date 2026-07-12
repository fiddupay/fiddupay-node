// Delora Constants
// Known router contracts, chain IDs, and adapter keys

use std::collections::HashMap;

/// Delora router contracts per chain (must be kept in sync with Delora docs)
pub fn known_router_contracts() -> HashMap<u64, &'static str> {
    HashMap::from([
        // Ethereum mainnet
        (1, "0x1111111111111111111111111111111111111111"),
        // Base
        (8453, "0x1111111111111111111111111111111111111111"),
        // Arbitrum
        (42161, "0x1111111111111111111111111111111111111111"),
        // Optimism
        (10, "0x1111111111111111111111111111111111111111"),
        // Polygon
        (137, "0x1111111111111111111111111111111111111111"),
        // BSC
        (56, "0x1111111111111111111111111111111111111111"),
        // Avalanche
        (43114, "0x1111111111111111111111111111111111111111"),
    ])
}

/// Common chain names for display
pub fn chain_name(chain_id: u64) -> &'static str {
    match chain_id {
        1 => "Ethereum",
        10 => "Optimism",
        56 => "BNB Smart Chain",
        137 => "Polygon",
        8453 => "Base",
        42161 => "Arbitrum",
        43114 => "Avalanche",
        59144 => "Linea",
        534352 => "Scroll",
        _ => "Unknown Chain",
    }
}

/// Delora native asset markers
pub const NATIVE_ETH: &str = "0x0000000000000000000000000000000000000000";
pub const NATIVE_SOL: &str = "So11111111111111111111111111111111111111112";

/// Delora base URL
pub const DELORA_BASE_URL: &str = "https://api.delora.build";

/// Cache TTLs (seconds)
pub const METADATA_CACHE_TTL: u64 = 3600; // chains, tokens, tools
pub const PRICE_CACHE_TTL: u64 = 60; // token prices
pub const QUOTE_CACHE_TTL: u64 = 15; // live quotes
pub const QUOTE_SNAPSHOT_TTL: u64 = 300; // stored quote snapshots

/// Cache key prefixes
pub const CACHE_PREFIX_CHAINS: &str = "delora:chains";
pub const CACHE_PREFIX_TOKENS: &str = "delora:tokens";
pub const CACHE_PREFIX_TOOLS: &str = "delora:tools";
pub const CACHE_PREFIX_PRICES: &str = "delora:prices";
pub const CACHE_PREFIX_QUOTE: &str = "delora:quote";
pub const CACHE_PREFIX_TOKEN: &str = "delora:token";

/// In-memory LRU cache size
pub const MEMORY_CACHE_SIZE: usize = 512;

/// Default quote expiry window (seconds)
pub const QUOTE_EXPIRY_SECONDS: i64 = 30;

/// Grace period after quote expiry for tx registration (seconds)
pub const REGISTRATION_GRACE_SECONDS: i64 = 60;
