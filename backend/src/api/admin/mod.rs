pub mod auth;
pub mod dashboard;
pub mod merchants;
pub mod config;
pub mod payments;
pub mod withdrawals;
pub mod wallets;
pub mod system;

// Re-export all handlers for easier access in routes
pub use auth::*;
pub use dashboard::*;
pub use merchants::*;
pub use config::*;
pub use payments::*;
pub use withdrawals::*;
pub use wallets::*;
pub use system::*;
