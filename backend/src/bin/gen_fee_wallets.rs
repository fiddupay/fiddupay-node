use fiddupay::utils::keygen::KeyGenerator;

fn main() {
    println!("====================================================");
    println!("   FIDDUPAY PLATFORM FEE WALLET GENERATOR");
    println!("====================================================");
    println!("");

    // Generate Solana Wallet
    let solana_wallet =
        KeyGenerator::generate_solana_wallet().expect("Failed to generate Solana wallet");

    // Generate EVM Wallet (used for ETH, BSC, Polygon, Arbitrum)
    let evm_wallet = KeyGenerator::generate_evm_wallet().expect("Failed to generate EVM wallet");

    println!("--- GENERATED WALLETS ---");
    println!("SOLANA:  {}", solana_wallet.address);
    println!(
        "EVM:     {} (Used for ETH, BSC, Polygon, Arbitrum)",
        evm_wallet.address
    );
    println!("");

    println!("--- .env CONFIGURATION ---");
    println!("Add the following to your backend .env file:");
    println!("");
    println!("PLATFORM_FEE_WALLET_SOL={}", solana_wallet.address);
    println!("PLATFORM_FEE_WALLET_ETH={}", evm_wallet.address);
    println!("PLATFORM_FEE_WALLET_BSC={}", evm_wallet.address);
    println!("PLATFORM_FEE_WALLET_POLYGON={}", evm_wallet.address);
    println!("PLATFORM_FEE_WALLET_ARBITRUM={}", evm_wallet.address);
    println!("");

    println!("--- PRIVATE KEYS (BACKUP THESE SAFELY!) ---");
    println!("SOLANA PRIVATE KEY:  {}", solana_wallet.private_key);
    println!("EVM PRIVATE KEY:     {}", evm_wallet.private_key);
    println!("");
    println!("WARNING: Keep these private keys secure. If lost, you cannot access collected fees.");
    println!("====================================================");
}
