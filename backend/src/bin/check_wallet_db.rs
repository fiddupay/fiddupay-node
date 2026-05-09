// Diagnostic script to check wallet registration for a specific address
// Usage: cargo run --bin check_wallet_db

use fiddupay::config::Config;
use sqlx::PgPool;
use sqlx::Row;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;
    let pool = PgPool::connect(&config.database_url).await?;

    let address = "0x06b64b859de6d1bc947557ae89fbdaadae56e099";
    println!("--- Checking DB for address: {} ---", address);

    let rows = sqlx::query(
        "SELECT customer_id, merchant_id, crypto_type, address FROM merchant_customer_wallets WHERE LOWER(address) = LOWER($1)"
    )
    .bind(address)
    .fetch_all(&pool)
    .await?;

    if rows.is_empty() {
        println!("❌ No wallet found for this address in merchant_customer_wallets.");
    } else {
        println!("✅ Found {} wallet(s):", rows.len());
        for row in rows {
            let c_id: i64 = row.get("customer_id");
            let m_id: i64 = row.get("merchant_id");
            let c_type: String = row.get("crypto_type");
            let addr: String = row.get("address");
            println!(
                "  - Customer: {}, Merchant: {}, Type: {}, Address: {}",
                c_id, m_id, c_type, addr
            );
        }
    }

    Ok(())
}
