use serde_json::Value;
use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tx_hash =
        "5r648Qu72nVFqC4huKQ5eicKfVH2QYm98TM6jhtFYoEtiKW8qw3gW2p3vhNbU9NBqTnGuMQJWYUDAfvEnTEPAZfA";
    let customer_sol_address = "GmMQxPVEY8SSncaWsLXytZAMN2aTR1JifkKYnS5WyNWQ";
    let usdt_mint = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
    let wsol_mint = "So11111111111111111111111111111111111111112";

    println!("============================================================");
    println!("Testing Verification Logic for TX:");
    println!("Hash: {}", tx_hash);
    println!("Customer SOL Address: {}", customer_sol_address);
    println!("============================================================\n");

    // Calculate ATAs
    let owner_pk = Pubkey::from_str(customer_sol_address)?;
    let usdt_mint_pk = Pubkey::from_str(usdt_mint)?;
    let wsol_mint_pk = Pubkey::from_str(wsol_mint)?;

    let usdt_ata = get_associated_token_address(&owner_pk, &usdt_mint_pk).to_string();
    let wsol_ata = get_associated_token_address(&owner_pk, &wsol_mint_pk).to_string();

    println!("[1] Computed Customer Addresses:");
    println!("  - Primary Address: {}", customer_sol_address);
    println!("  - USDT ATA:        {}", usdt_ata);
    println!("  - WSOL ATA:        {}", wsol_ata);

    // Fetch on-chain TX details via Public Solana RPC
    let client = reqwest::Client::new();
    let rpc_url =
        "https://basic.rpc.solanavibestation.com/?api_key=870b1e8740456e66961b45d9031044f4";

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getTransaction",
        "params": [
            tx_hash,
            {
                "encoding": "json",
                "maxSupportedTransactionVersion": 0,
                "commitment": "confirmed"
            }
        ]
    });

    let response = client.post(rpc_url).json(&request).send().await?;
    let text = response.text().await?;
    let v: Value = serde_json::from_str(&text)?;
    let result = &v["result"];

    if result.is_null() {
        println!("\n❌ Transaction not found on RPC node.");
        return Ok(());
    }

    let meta = &result["meta"];
    let mut detected_to_address = String::new();
    let mut detected_mint = String::new();
    let mut detected_amount = 0.0;

    if let Some(post) = meta["postTokenBalances"].as_array() {
        for post_tb in post {
            let account_index = post_tb["accountIndex"].as_u64().unwrap_or(0);
            let post_raw = post_tb["uiTokenAmount"]["amount"]
                .as_str()
                .and_then(|s| s.parse::<u128>().ok())
                .unwrap_or(0);
            let decimals = post_tb["uiTokenAmount"]["decimals"].as_u64().unwrap_or(6) as u32;
            let mint = post_tb["mint"].as_str().unwrap_or("");
            let owner = post_tb["owner"].as_str().unwrap_or("");

            let mut pre_raw = 0;
            if let Some(pre) = meta["preTokenBalances"].as_array() {
                for pre_tb in pre {
                    if pre_tb["accountIndex"].as_u64().unwrap_or(0) == account_index {
                        pre_raw = pre_tb["uiTokenAmount"]["amount"]
                            .as_str()
                            .and_then(|s| s.parse::<u128>().ok())
                            .unwrap_or(0);
                        break;
                    }
                }
            }

            if post_raw > pre_raw {
                let increase = post_raw - pre_raw;
                detected_amount = (increase as f64) / 10f64.powi(decimals as i32);
                detected_mint = mint.to_string();

                // Get the account address from transaction account keys
                if let Some(account_keys) =
                    result["transaction"]["message"]["accountKeys"].as_array()
                {
                    if let Some(key_val) = account_keys.get(account_index as usize) {
                        if let Some(key_str) = key_val.as_str() {
                            detected_to_address = key_str.to_string();
                        } else if let Some(pubkey_str) = key_val["pubkey"].as_str() {
                            detected_to_address = pubkey_str.to_string();
                        }
                    }
                }

                // If owner is present in token balance, we can also check that
                if detected_to_address.is_empty() && !owner.is_empty() {
                    detected_to_address = owner.to_string();
                }
                break;
            }
        }
    }

    println!("\n[2] Parsed Blockchain Details:");
    println!("  - Detected Recipient Address: {}", detected_to_address);
    println!("  - Token Mint:                 {}", detected_mint);
    println!("  - Token Amount:               {}", detected_amount);

    // Simulate New Matcher Logic
    let matches_primary = detected_to_address.trim() == customer_sol_address.trim();
    let matches_usdt_ata = detected_to_address.trim() == usdt_ata.trim();
    let matches_wsol_ata = detected_to_address.trim() == wsol_ata.trim();
    let addresses_match = matches_primary || matches_usdt_ata || matches_wsol_ata;

    println!("\n[3] Verifier Matching Results:");
    println!("  - Matches Primary Base Address? {}", matches_primary);
    println!("  - Matches USDT ATA Address?     {}", matches_usdt_ata);
    println!("  - Matches WSOL ATA Address?     {}", matches_wsol_ata);

    if addresses_match {
        println!("\n✅ VERIFICATION SUCCESSFUL!");
        println!("  - Recipient match confirmed.");
        if detected_mint == usdt_mint {
            println!(
                "  - Asset Identified: USDT_SPL (Amount: {})",
                detected_amount
            );
        } else if detected_mint == wsol_mint {
            println!("  - Asset Identified: WSOL (Amount: {})", detected_amount);
        } else {
            println!("  - Asset Identified: Native SOL / Other");
        }
    } else {
        println!("\n❌ VERIFICATION FAILED: Recipient address mismatch!");
    }

    Ok(())
}
