// Smart test script to identify the ACTUAL token in a BSC transaction
// Usage: cargo run --bin test_bsc_hash

use fiddupay::config::Config;
use fiddupay::payment::blockchain_monitor::get_blockchain_monitor;
use fiddupay::payment::models::CryptoType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;
    
    let tx_hash = "0xda818a10f64cb9ff785174d5b8725d44e29795d675efcb889479b2d7ca3bb4ab";
    println!("--- Smart Blockchain Analysis for Hash: {} ---", tx_hash);
    
    // We only need one monitor now because it's universal!
    let monitor = get_blockchain_monitor(&CryptoType::Bnb, config.clone(), false);
    
    match monitor.get_transaction_details(tx_hash, None).await {
        Ok(tx) => {
            println!("\n✅ Transaction Analyzed!");
            println!("----------------------------------");
            println!("Hash: {}", tx.hash);
            println!("From: {}", tx.from_address);
            println!("To (Recipient): {}", tx.to_address);
            
            if let Some(mint) = tx.token_mint {
                println!("Detected Asset Type: TOKEN TRANSFER");
                println!("Contract Address: {}", mint);
                
                // Identify the token by its contract
                if mint.to_lowercase() == "0x55d398326f99059ff775485246999027b3197955" {
                    println!("Token Identified: ✨ USDT (BEP20) ✨");
                } else if mint.to_lowercase() == "0xe9e7cea3dedca5984780bafc599bd69add087d56" {
                    println!("Token Identified: ✨ BUSD (BEP20) ✨");
                } else {
                    println!("Token Identified: Unknown Token Contract");
                }
                
                println!("Amount: {} units", tx.amount);
            } else {
                println!("Detected Asset Type: NATIVE (BNB)");
                println!("Amount: {} BNB", tx.amount);
            }
            
            println!("Success: {}", tx.success);
            println!("Confirmations: {}", tx.confirmations);
            println!("----------------------------------");
        },
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
    
    Ok(())
}
