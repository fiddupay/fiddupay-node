
const { Client } = require('pg');

async function checkMerchantWallet() {
    const client = new Client({
        connectionString: process.env.DATABASE_URL || 'postgres://postgres:postgres@localhost:5432/fiddupay'
    });

    try {
        await client.connect();
        
        console.log("--- Checking Merchant 12 Solana Wallets ---");
        const res = await client.query(
            "SELECT crypto_type, address, is_active, sandbox_mode FROM merchant_wallets WHERE merchant_id = 12"
        );
        
        console.table(res.rows);

        console.log("\n--- Checking Merchant 12 Balances ---");
        const balRes = await client.query(
            "SELECT crypto_type, available_balance, sandbox_mode FROM merchant_balances WHERE merchant_id = 12"
        );
        console.table(balRes.rows);

    } catch (err) {
        console.error(err);
    } finally {
        await client.end();
    }
}

checkMerchantWallet();
