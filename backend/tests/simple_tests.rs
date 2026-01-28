// Simple Phase 1 Tests - Core Infrastructure
// Basic tests to validate system components

#[cfg(test)]
mod phase1_tests {
    use fiddupay::utils::keygen::KeyGenerator;

    #[test]
    fn test_key_generator_evm() {
        println!(" Testing EVM wallet generation...");
        
        let result = KeyGenerator::generate_evm_wallet();
        assert!(result.is_ok(), "EVM wallet generation should succeed");
        
        let wallet = result.unwrap();
        assert_eq!(wallet.private_key.len(), 64, "Private key should be 64 hex chars");
        assert!(wallet.address.starts_with("0x"), "Address should start with 0x");
        assert_eq!(wallet.address.len(), 42, "Address should be 42 chars");
        
        println!(" EVM wallet generation test passed");
    }

    #[test]
    fn test_key_generator_solana() {
        println!(" Testing Solana wallet generation...");
        
        let result = KeyGenerator::generate_solana_wallet();
        assert!(result.is_ok(), "Solana wallet generation should succeed");
        
        let wallet = result.unwrap();
        assert!(!wallet.private_key.is_empty(), "Private key should not be empty");
        assert!(!wallet.address.is_empty(), "Address should not be empty");
        
        println!(" Solana wallet generation test passed");
    }

    #[test]
    fn test_crypto_type_display() {
        use fiddupay::models::payment::CryptoType;
        
        println!(" Testing CryptoType display...");
        
        assert_eq!(format!("{}", CryptoType::Sol), "SOL");
        assert_eq!(format!("{}", CryptoType::Eth), "ETH");
        assert_eq!(format!("{}", CryptoType::UsdtEth), "USDT-ERC20");
        
        println!(" CryptoType display test passed");
    }
}
