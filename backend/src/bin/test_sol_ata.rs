use dotenvy::dotenv;
use fiddupay::config::Config;
use fiddupay::payment::sol_monitor::SolanaMonitor;
use rust_decimal::Decimal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();
    let config = Config::from_env().map_err(|e| e.to_string())?;
    
    // The provided hash is on Mainnet
    let is_sandbox = false;
    // Use the official USDT mint confirmed by the user
    let usdt_mint = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string();
    let monitor = SolanaMonitor::new(&config, is_sandbox, Some(usdt_mint.clone()));
    
    let hash = "4jCt18y8JC2UrxkQA6HYPDzKcj7deqNYKBupACSJ2Da8yJNc1vZusT8kYUKogDjmZrHPSMz5NNYAg8zbuwFgYY67";
    
    println!("🔍 VERIFICATION: Fetching transaction {} for USDT mint {}", hash, usdt_mint);
    
    let tx = monitor.get_transaction_details(hash).await?;
    
    println!("✅ Transaction parsed!");
    println!("   Hash: {}", tx.hash);
    println!("   From: {}", tx.from_address);
    println!("   To (Recipient): {}", tx.to_address);
    println!("   Amount: {} tokens", tx.amount);
    println!("   Detected Mint: {:?}", tx.token_mint);
    
    if tx.amount > Decimal::ZERO {
        println!("\n✨ SUCCESS: Detected a valid transfer!");
        if let Some(mint) = &tx.token_mint {
            if mint == &usdt_mint {
                println!("   CONFIRMED: Mint matches the expected USDT address.");
            } else {
                println!("   WARNING: Mint mismatch! Expected {}, got {}", usdt_mint, mint);
            }
        }
    } else {
        println!("\n❌ FAILED: No token transfer detected for the specified mint.");
    }

    Ok(())
}
