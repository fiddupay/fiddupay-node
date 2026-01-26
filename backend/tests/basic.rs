#[cfg(test)]
mod basic_tests {
    use fiddupay::payment::models::CryptoType;

    #[test]
    fn test_crypto_type_basic() {
        let crypto = CryptoType::Sol;
        assert_eq!(format!("{}", crypto), "SOL");
        assert!(crypto.is_native_currency());
    }

    #[test]
    fn test_crypto_type_network() {
        let crypto = CryptoType::UsdtEth;
        assert_eq!(crypto.network(), "ETHEREUM");
        assert_eq!(crypto.get_native_currency(), CryptoType::Eth);
    }
}
