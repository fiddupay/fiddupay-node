use rust_decimal::Decimal;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tx_hash = "4jCt18y8JC2UrxkQA6HYPDzKcj7deqNYKBupACSJ2Da8yJNc1vZusT8kYUKogDjmZrHPSMz5NNYAg8zbuwFgYY67";
    
    // Config needed just for RPC URL
    let client = reqwest::Client::new();
    let rpc_url = "https://api.mainnet-beta.solana.com"; 
    
    println!("Fetching transaction: {}", tx_hash);
    
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
        println!("Transaction not found on this RPC node.");
        return Ok(());
    }
    
    let meta = &result["meta"];
    
    println!("--- Pre Token Balances ---");
    if let Some(pre) = meta["preTokenBalances"].as_array() {
        for (i, v) in pre.iter().enumerate() {
            println!("[{}] Index: {} | Mint: {} | Owner: {} | Amount: {}", 
                i, 
                v["accountIndex"], 
                v["mint"].as_str().unwrap_or("none"), 
                v["owner"].as_str().unwrap_or("none"),
                v["uiTokenAmount"]["uiAmount"]
            );
        }
    } else {
        println!("None");
    }
    
    println!("\n--- Post Token Balances ---");
    if let Some(post) = meta["postTokenBalances"].as_array() {
        for (i, v) in post.iter().enumerate() {
            println!("[{}] Index: {} | Mint: {} | Owner: {} | Amount: {}", 
                i, 
                v["accountIndex"], 
                v["mint"].as_str().unwrap_or("none"), 
                v["owner"].as_str().unwrap_or("none"),
                v["uiTokenAmount"]["amount"] // using raw amount like our rust code
            );
        }
    } else {
        println!("None");
    }
    
    // Replicate our Rust parsing logic
    println!("\n--- Replicating parsing logic ---");
    let mut best_owner = String::new();
    let mut best_amount = Decimal::ZERO;
    
    if let Some(post) = meta["postTokenBalances"].as_array() {
        for post_tb in post {
            let account_index = post_tb["accountIndex"].as_u64().unwrap_or(0);
            let post_raw = post_tb["uiTokenAmount"]["amount"]
                .as_str()
                .and_then(|s| s.parse::<u128>().ok())
                .unwrap_or(0);
            let decimals = post_tb["uiTokenAmount"]["decimals"].as_u64().unwrap_or(6) as u32;
            
            // Find pre balance
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
                let token_amount = Decimal::from(increase) / Decimal::from(10u64.pow(decimals));
                
                if token_amount > best_amount {
                    best_amount = token_amount;
                    if let Some(owner) = post_tb["owner"].as_str() {
                        best_owner = owner.to_string();
                    }
                }
            }
        }
    }
    
    println!("Recipient Owner: {}", best_owner);
    println!("Amount Received: {}", best_amount);

    Ok(())
}
