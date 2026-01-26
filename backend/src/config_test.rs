use std::env;

fn main() {
    // Load .env file
    dotenvy::dotenv().ok();
    
    println!(" Environment variables loaded:");
    
    // Test core variables
    println!("  DATABASE_URL: {}", env::var("DATABASE_URL").unwrap_or("NOT SET".to_string()));
    println!("  SERVER_PORT: {}", env::var("SERVER_PORT").unwrap_or("NOT SET".to_string()));
    println!("  ENVIRONMENT: {}", env::var("ENVIRONMENT").unwrap_or("NOT SET".to_string()));
    println!("  DEBUG_MODE: {}", env::var("DEBUG_MODE").unwrap_or("NOT SET".to_string()));
    
    // Test security variables
    println!("  ENCRYPTION_KEY: {}", if env::var("ENCRYPTION_KEY").is_ok() { "SET" } else { "NOT SET" });
    println!("  WEBHOOK_SIGNING_KEY: {}", if env::var("WEBHOOK_SIGNING_KEY").is_ok() { "SET" } else { "NOT SET" });
    
    // Test blockchain variables
    println!("  SOLANA_RPC_URL: {}", env::var("SOLANA_RPC_URL").unwrap_or("NOT SET".to_string()));
    println!("  ETHEREUM_RPC_URL: {}", env::var("ETHEREUM_RPC_URL").unwrap_or("NOT SET".to_string()));
    
    // Test feature flags
    println!("  TWO_FACTOR_ENABLED: {}", env::var("TWO_FACTOR_ENABLED").unwrap_or("NOT SET".to_string()));
    println!("  MAINTENANCE_MODE: {}", env::var("MAINTENANCE_MODE").unwrap_or("NOT SET".to_string()));
    
    // Test payment settings
    println!("  DEFAULT_FEE_PERCENTAGE: {}", env::var("DEFAULT_FEE_PERCENTAGE").unwrap_or("NOT SET".to_string()));
    println!("  MIN_PAYMENT_USD: {}", env::var("MIN_PAYMENT_USD").unwrap_or("NOT SET".to_string()));
    
    println!("\n Configuration test completed successfully!");
}
