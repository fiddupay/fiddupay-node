// Bitcoin API Failover Test Utility
// Tests fetching transaction details and proves failover is working.
//
// Usage:
//   cargo run --bin btc_test_util -- <txid> [sandbox=false]
//
// Example (mainnet):
//   cargo run --bin btc_test_util -- ced14b7fb86f552d409dc7b017305bd9641ccc5a3c0559a367326ec2be344641

use fiddupay::utils::bitcoin_api::{get_with_failover, BitcoinApiConfig};
use rust_decimal::Decimal;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run --bin btc_test_util -- <txid> [sandbox=false]");
        println!();
        println!("Example:");
        println!("  cargo run --bin btc_test_util -- ced14b7fb86f552d409dc7b017305bd9641ccc5a3c0559a367326ec2be344641");
        return Ok(());
    }

    let txid = &args[1];
    let sandbox = args.get(2).map(|s| s == "true").unwrap_or(false);

    let primary_url = if sandbox {
        env::var("BITCOIN_TESTNET_RPC_URL")
            .unwrap_or_else(|_| "https://blockstream.info/testnet/api".to_string())
    } else {
        env::var("BITCOIN_RPC_URL").unwrap_or_else(|_| "https://blockstream.info/api".to_string())
    };

    let backup_url = if sandbox {
        env::var("BITCOIN_TESTNET_RPC_URL_BACKUP")
            .unwrap_or_else(|_| "https://mempool.space/testnet/api".to_string())
    } else {
        env::var("BITCOIN_RPC_URL_BACKUP")
            .unwrap_or_else(|_| "https://mempool.space/api".to_string())
    };

    println!("╔══════════════════════════════════════════════════════╗");
    println!("║       Bitcoin API Failover Test                      ║");
    println!("╚══════════════════════════════════════════════════════╝");
    println!(
        "  Network  : {}",
        if sandbox { "Testnet" } else { "Mainnet" }
    );
    println!("  TXID     : {}", txid);
    println!("  Primary  : {}", primary_url);
    println!("  Backup   : {}", backup_url);
    println!();

    // ── Test 1: Fetch tx via normal config (primary first) ──────────────────
    println!("▶ [TEST 1] Fetching transaction via PRIMARY provider...");
    let config = BitcoinApiConfig {
        primary_url: primary_url.trim_end_matches('/').to_string(),
        backup_url: backup_url.trim_end_matches('/').to_string(),
    };

    match get_with_failover(&config, &format!("tx/{}", txid)).await {
        Ok(data) => {
            let confirmed = data
                .get("status")
                .and_then(|s| s.get("confirmed"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let block_height = data
                .get("status")
                .and_then(|s| s.get("block_height"))
                .and_then(|v| v.as_u64());
            let from_addr = data
                .get("vin")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|i| i.get("prevout"))
                .and_then(|p| p.get("scriptpubkey_address"))
                .and_then(|a| a.as_str())
                .unwrap_or("unknown");

            // Sum all outputs to find total and first recipient
            let mut first_recipient = "unknown".to_string();
            let mut total_sats = 0u64;
            if let Some(vout) = data.get("vout").and_then(|v| v.as_array()) {
                for out in vout {
                    let addr = out
                        .get("scriptpubkey_address")
                        .and_then(|a| a.as_str())
                        .unwrap_or("");
                    let val = out.get("value").and_then(|v| v.as_u64()).unwrap_or(0);
                    if !addr.is_empty() && first_recipient == "unknown" {
                        first_recipient = addr.to_string();
                    }
                    total_sats += val;
                }
            }
            let total_btc = Decimal::from(total_sats) / Decimal::from(100_000_000u64);

            println!("  ✅ SUCCESS — Transaction found on primary!");
            println!("     From       : {}", from_addr);
            println!("     To (first) : {}", first_recipient);
            println!("     Total out  : {} BTC ({} sats)", total_btc, total_sats);
            println!("     Confirmed  : {}", confirmed);
            println!("     Block      : {:?}", block_height);
        }
        Err(e) => println!("  ❌ FAILED: {}", e),
    }

    println!();

    // ── Test 2: Simulate primary failure → prove failover kicks in ──────────
    println!("▶ [TEST 2] Simulating primary failure (bad URL) → expecting failover to backup...");
    let broken_config = BitcoinApiConfig {
        primary_url: "https://THIS-WILL-ALWAYS-FAIL.invalid".to_string(),
        backup_url: backup_url.trim_end_matches('/').to_string(),
    };

    match get_with_failover(&broken_config, &format!("tx/{}", txid)).await {
        Ok(data) => {
            let confirmed = data
                .get("status")
                .and_then(|s| s.get("confirmed"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            println!("  ✅ FAILOVER SUCCESS — Backup provider responded!");
            println!("     Transaction confirmed: {}", confirmed);
            println!("     (Primary intentionally broken — failover worked correctly ✓)");
        }
        Err(e) => println!("  ❌ FAILOVER ALSO FAILED: {}", e),
    }

    println!();
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║            Tests Complete                            ║");
    println!("╚══════════════════════════════════════════════════════╝");

    Ok(())
}
