pub mod auth;
pub mod config;
pub mod dashboard;
pub mod merchants;
pub mod payments;
pub mod system;
pub mod wallets;
pub mod withdrawals;

// Re-export all handlers for easier access in routes
pub use auth::*;
pub use config::*;
pub use dashboard::*;
pub use merchants::*;
pub use payments::*;
pub use system::*;
pub use wallets::*;
pub use withdrawals::*;
