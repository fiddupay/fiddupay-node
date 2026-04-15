// Ethereum (EVM) Test Utility
// Tests fetching transaction details using the EVM monitor.
//
// Usage:
//   cargo run --bin eth_test_util -- <crypto_type> <txid> [sandbox=false]
//
// Examples:
//   cargo run --bin eth_test_util -- USDT_ETH 0xfe0923a72f894a4982b4f464b0fd59c4b34ce74f33a2b78fff26d688792f6fdf
//   cargo run --bin eth_test_util -- ETH 0x399e9971ddff89ee537744e5cde976e94ce2f2034b9fb54e1b2ba06d94c701fc

use fiddupay::config::Config;
use fiddupay::payment::blockchain_monitor::get_blockchain_monitor;
use fiddupay::payment::models::CryptoType;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Disable noisy tracing logs in tests unless user explicitly asks
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    tracing_subscriber::fmt::init();

    let config = Config::from_env().expect("Failed to load config");

    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: cargo run --bin eth_test_util -- <crypto_type> <txid> [sandbox=false]");
        println!();
        println!("Examples:");
        println!("  cargo run --bin eth_test_util -- USDT_ETH 0xfe0923a72f894a4982b4f464b0fd59c4b34ce74f33a2b78fff26d688792f6fdf");
        println!("  cargo run --bin eth_test_util -- ETH 0x399e9971ddff89ee537744e5cde976e94ce2f2034b9fb54e1b2ba06d94c701fc");
        return Ok(());
    }

    let crypto_str = &args[1];
    let txid = &args[2];
    let sandbox = args.get(3).map(|s| s == "true").unwrap_or(false);

    let crypto_type = CryptoType::from_string(crypto_str).expect("Invalid CryptoType");

    println!("╔══════════════════════════════════════════════════════╗");
    println!("║       EVM Blockchain Monitor Test                    ║");
    println!("╚══════════════════════════════════════════════════════╝");
    println!("  Network  : {}", crypto_type.network());
    println!("  Asset    : {}", crypto_type);
    println!(
        "  Mode     : {}",
        if sandbox { "Sandbox" } else { "Mainnet" }
    );
    println!("  TXID     : {}", txid);
    println!("  Token    : {:?}", crypto_type.token_address());
    println!();

    println!("▶  Fetching transaction details...");

    // We pass None for target_address because EvmMonitor relies on its internal parsed token matches
    let monitor = get_blockchain_monitor(&crypto_type, config, sandbox);

    match monitor.get_transaction_details(txid, None).await {
        Ok(tx) => {
            println!("  ✅ SUCCESS — Transaction found!");
            println!("     From       : {}", tx.from_address);
            println!("     To         : {}", tx.to_address);
            println!("     Amount     : {} {}", tx.amount, crypto_type.as_str());
            println!("     Success    : {}", tx.success);
            println!(
                "     Confirmed  : {}",
                if tx.confirmations >= 1 {
                    "true"
                } else {
                    "false"
                }
            );
            println!("     Confs      : {}", tx.confirmations);
            println!("     Block      : {:?}", tx.block_number);
        }
        Err(e) => {
            println!("  ❌ FAILED: {}", e);
            println!("\nMake sure your ETHERSCAN_API_KEY is set in your .env file!");
        }
    }

    println!();
    Ok(())
}
