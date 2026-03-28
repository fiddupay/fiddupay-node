use fiddupay::payment::blockchain_monitor::btc_monitor::BtcMonitor;
use fiddupay::payment::blockchain_monitor::BlockchainMonitor;
use fiddupay::payment::models::CryptoType;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: cargo run --bin btc_test_util -- <txid> <target_address> [sandbox=false]");
        return Ok(());
    }

    let txid = &args[1];
    let target_address = &args[2];
    let sandbox = args.get(3).map(|s| s == "true").unwrap_or(false);

    println!("--- BTC Transaction Verification Test ---");
    println!("TXID: {}", txid);
    println!("Target Address: {}", target_address);
    println!("Network: {}", if sandbox { "Testnet" } else { "Mainnet" });
    println!("-----------------------------------------");

    let monitor = BtcMonitor::new(sandbox);

    println!("Fetching transaction details...");
    match monitor.get_transaction_details(txid, Some(target_address)).await {
        Ok(details) => {
            println!("SUCCESS: Transaction found!");
            println!("  Hash:           {}", details.hash);
            println!("  Sender:         {}", details.from_address);
            println!("  Recipient:      {}", details.to_address);
            println!("  Amount:         {} BTC", details.amount);
            println!("  Confirmations:  {}", details.confirmations);
            println!("  Success:        {}", details.success);
            println!("  Timestamp:      {:?}", details.timestamp);
            
            if details.to_address == *target_address {
                println!("\n✅ VERIFICATION PASSED: Address matches.");
            } else {
                println!("\n❌ VERIFICATION FAILED: Address mismatch (found {})", details.to_address);
            }
        }
        Err(e) => {
            println!("ERROR: Failed to fetch transaction: {}", e);
        }
    }

    Ok(())
}
