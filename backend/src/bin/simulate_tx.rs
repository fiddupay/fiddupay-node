// Simulation Tool for Transactions
// Usage: cargo run --bin simulate_tx <network> <tx_hash>

use dotenvy::dotenv;
use fiddupay::config::Config;
use rust_decimal::Decimal;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        println!("Usage: cargo run --bin simulate_tx <network> <tx_hash>");
        println!("Example: cargo run --bin simulate_tx btc 1234abcd...");
        return Ok(());
    }

    let network = args[1].to_uppercase();
    let tx_hash = &args[2];

    println!("=== Simulating Transaction Lookup ===");
    println!("Network: {}", network);
    println!("TxHash:  {}", tx_hash);
    println!("--------------------------------------");

    match network.as_str() {
        "BTC" => simulate_btc(tx_hash).await?,
        "SOL" => simulate_solana(tx_hash).await?,
        "ETH" | "BNB" | "MATIC" | "ARB" => simulate_evm(&network, tx_hash).await?,
        _ => println!("Unsupported network: {}. Use BTC, SOL, ETH, BNB, MATIC, ARB.", network),
    }

    Ok(())
}

async fn simulate_btc(tx_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env().map_err(|e| format!("Config loading failed: {}", e))?;
    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL")?).await?;

    let api_url = if config.bitcoin_rpc_url.contains("testnet") {
        "https://blockstream.info/testnet/api"
    } else {
        "https://blockstream.info/api"
    };
    
    let url = format!("{}/tx/{}", api_url, tx_hash);
    println!("Fetching from: {}", url);
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;
    
    if !response.status().is_success() {
        println!("Error fetching BTC Tx: {}", response.status());
        return Ok(());
    }

    let tx_data: Value = response.json().await?;
    
    println!("\n[BTC Transaction Details]");
    println!("Fee: {} sats", tx_data["fee"].as_u64().unwrap_or(0));
    println!("Size: {} bytes", tx_data["size"].as_u64().unwrap_or(0));
    
    println!("\nInputs:");
    if let Some(inputs) = tx_data["vin"].as_array() {
        for (i, input) in inputs.iter().enumerate() {
            let prevout = &input["prevout"];
            let addr = prevout["scriptpubkey_address"].as_str().unwrap_or("Unknown");
            let value = prevout["value"].as_u64().unwrap_or(0);
            println!("  #{}: {} -> {} sats", i, addr, value);
        }
    }

    println!("\nOutputs:");
    let mut outputs_list = Vec::new();
    if let Some(outputs) = tx_data["vout"].as_array() {
        for (i, output) in outputs.iter().enumerate() {
            let addr = output["scriptpubkey_address"].as_str().unwrap_or("Unknown");
            let value = output["value"].as_u64().unwrap_or(0);
            println!("  #{}: {} -> {} sats", i, addr, value);
            outputs_list.push(addr.to_string());
        }
    }

    println!("\n[System Interpretation]");
    for addr in outputs_list {
        // Query AddressOnlyPayment
        let match_address_only = sqlx::query(
            "SELECT payment_id, status, requested_amount FROM address_only_payments WHERE gateway_deposit_address = $1"
        )
        .bind(&addr)
        .fetch_optional(&pool).await?;

        if let Some(row) = match_address_only {
            use sqlx::Row;
            println!("✅ MATCH: Address {} belongs to AddressOnlyPayment!", addr);
            println!("   Payment ID: {}", row.get::<String, _>("payment_id"));
            println!("   Status:     {}", row.get::<String, _>("status"));
            println!("   Amount:     {} BTC required", row.get::<Decimal, _>("requested_amount"));
            continue;
        }

        // Query Merchant Wallets Config
        let match_merchant_wallet = sqlx::query(
            "SELECT merchant_id FROM merchant_wallets WHERE address = $1"
        )
        .bind(&addr)
        .fetch_optional(&pool).await?;

        if let Some(row) = match_merchant_wallet {
            use sqlx::Row;
            println!("✅ MATCH: Address {} belongs to Merchant {}", addr, row.get::<i32, _>("merchant_id"));
            continue;
        }
    }
    
    Ok(())
}

async fn simulate_solana(tx_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Solana Simulation lookup via RpcClient needing config loading...");
    // To support full simulate, we'd load config and use solana-client.
    // Simplifying with explanation for now:
    println!("(Use `solana-client` with Config to dump instructions)");
    Ok(())
}

async fn simulate_evm(network: &str, tx_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("EVM Simulation lookup via Web3 for {}...", network);
    Ok(())
}
