#[cfg(test)]
mod minimal_tests {
    use fiddupay::payment::models::CryptoType;

    #[test]
    fn test_crypto_type_display() {
        let crypto = CryptoType::Sol;
        assert_eq!(format!("{}", crypto), "SOL");
        println!("✅ CryptoType display test passed");
    }

    #[test]
    fn test_crypto_type_network() {
        let crypto = CryptoType::UsdtEth;
        assert_eq!(crypto.network(), "ETHEREUM");
        assert_eq!(crypto.get_native_currency(), CryptoType::Eth);
        println!("✅ CryptoType network test passed");
    }

    #[test]
    fn test_crypto_type_from_str() {
        let crypto: CryptoType = "SOL".parse().unwrap();
        assert_eq!(crypto, CryptoType::Sol);
        println!("✅ CryptoType FromStr test passed");
    }
}
