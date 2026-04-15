use fiddupay::{config::Config, utils::encryption::Encryption};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Load config
    let config = Config::from_env()?;
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let target_address = "28dcadjmtjHeu7LgtzA6BHyANFGgfcT5PFHfjU25XTr5";
    let history_id: i64 = 3;

    println!("🚀 Starting SOL withdrawal for history ID: {}", history_id);

    // 1. Connect to DB
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await?;

    // 2. Fetch the record
    let record =
        sqlx::query("SELECT encrypted_private_key FROM merchant_wallet_history WHERE id = $1")
            .bind(history_id)
            .fetch_optional(&pool)
            .await?;

    let record = match record {
        Some(r) => r,
        None => {
            println!(
                "❌ Record with ID {} not found in merchant_wallet_history",
                history_id
            );
            return Ok(());
        }
    };

    use sqlx::Row;
    let encrypted_key: String = record.get("encrypted_private_key");

    // Default to Mainnet for history withdrawal unless specified
    let force_devnet = env::var("SOLANA_NETWORK").unwrap_or_default() == "devnet";
    let sandbox_mode = force_devnet;

    // 3. Decrypt key
    let encryption = Encryption::new().map_err(|e| format!("Encryption init failed: {}", e))?;
    let private_key = encryption
        .decrypt(&encrypted_key)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    // 4. Initialize Solana Client
    let rpc_url = if sandbox_mode {
        println!("🛠️ Using Devnet RPC");
        config.solana_devnet_rpc_url.clone()
    } else {
        println!("🌐 Using Mainnet RPC");
        config.solana_rpc_url.clone()
    };
    let rpc_client = RpcClient::new(rpc_url.clone());

    // 5. Parse keys
    let sender_keypair = Keypair::from_base58_string(&private_key);
    let sender_pubkey = sender_keypair.pubkey();
    let to_pubkey = Pubkey::from_str(target_address)?;

    println!("💳 Source Wallet: {}", sender_pubkey);
    println!("🎯 Target Address: {}", to_pubkey);

    // 6. Check Balance
    let balance = rpc_client.get_balance(&sender_pubkey).await?;
    println!(
        "💰 Current Balance: {} lamports ({} SOL)",
        balance,
        balance as f64 / 1_000_000_000.0
    );

    if balance == 0 {
        println!("⚠️ Balance is 0. Nothing to withdraw.");
        return Ok(());
    }

    // 7. Get Fee
    let recent_blockhash = rpc_client.get_latest_blockhash().await?;

    // Create a dummy message to estimate fee
    let message = solana_sdk::message::Message::new(
        &[system_instruction::transfer(
            &sender_pubkey,
            &to_pubkey,
            balance,
        )],
        Some(&sender_pubkey),
    );
    let fee = rpc_client
        .get_fee_for_message(&message)
        .await
        .unwrap_or(5000);
    println!("💸 Estimated Fee: {} lamports", fee);

    if balance <= fee {
        println!("❌ Balance is too low to cover fees.");
        return Ok(());
    }

    let transfer_amount = balance - fee;
    println!(
        "✨ Transferring: {} lamports ({} SOL)",
        transfer_amount,
        transfer_amount as f64 / 1_000_000_000.0
    );

    // 8. Create and Send Transaction
    let instruction = system_instruction::transfer(&sender_pubkey, &to_pubkey, transfer_amount);
    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&sender_pubkey),
        &[&sender_keypair],
        recent_blockhash,
    );

    // 9. Simulation
    print!("🔍 Simulating transaction... ");
    let sim = rpc_client.simulate_transaction(&tx).await?;
    if let Some(err) = sim.value.err {
        println!("❌ FAILED: {:?}", err);
        return Ok(());
    }
    println!("✅ SUCCESS");

    // 10. Execution
    if env::var("DRY_RUN").unwrap_or_default() == "true" {
        println!("🛑 DRY_RUN enabled. Skipping broadcast.");
        return Ok(());
    }

    println!("⚡ Broadcasting transaction...");
    let signature = rpc_client.send_and_confirm_transaction(&tx).await?;
    println!("🎉 Withdrawal successful!");
    println!("🔗 Signature: {}", signature);
    println!(
        "🔍 View on Explorer: https://explorer.solana.com/tx/{}?cluster={}",
        signature,
        if sandbox_mode {
            "devnet"
        } else {
            "mainnet-beta"
        }
    );

    Ok(())
}
