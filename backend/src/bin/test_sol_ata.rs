use dotenvy::dotenv;
use fiddupay::config::Config;
use fiddupay::payment::sol_monitor::SolanaMonitor;
use rust_decimal::Decimal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();
    let config = Config::from_env();
    
    // The provided hash is on Mainnet
    let rpc_url = Some(config.solana_rpc_url.clone());
    // Use the official USDT mint for testing
    let monitor = SolanaMonitor::new(&config, rpc_url, Some("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string()));
    
    let hash = "4jCt18y8JC2UrxkQA6HYPDzKcj7deqNYKBupACSJ2Da8yJNc1vZusT8kYUKogDjmZrHPSMz5NNYAg8zbuwFgYY67";
    
    println!("🔍 Fetching details for Solana USDT transaction: {}", hash);
    
    let tx = monitor.get_transaction_details(hash).await?;
    
    println!("✅ Transaction detected!");
    println!("   Hash: {}", tx.hash);
    println!("   From: {}", tx.from_address);
    println!("   To (Owner parsed): {}", tx.to_address);
    println!("   Amount: {} USDT", tx.amount);
    println!("   Success: {}", tx.success);
    println!("   Confirmations: {}", tx.confirmations);
    
    if tx.amount > Decimal::ZERO {
        println!("\n✨ SUCCESS: SPL Token amount correctly parsed!");
        println!("   The 'to_address' should be the OWNER of the ATA: {}", tx.to_address);
    } else {
        println!("\n❌ FAILED: Amount is zero. Check mint address and parsing logic.");
    }

    Ok(())
}
