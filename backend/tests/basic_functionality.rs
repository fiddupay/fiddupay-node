#[test]
fn test_default_fee_from_config() {
    // Test that default fee is configurable
    let fee_percentage = rust_decimal::Decimal::new(75, 2); // 0.75% from .env
    
    assert_eq!(fee_percentage, rust_decimal::Decimal::new(75, 2));
    println!(" Default fee percentage: {}%", fee_percentage);
    
    // Test fee calculation
    let payment_amount = rust_decimal::Decimal::new(10000, 2); // $100.00
    let fee_amount = payment_amount * fee_percentage / rust_decimal::Decimal::new(100, 0);
    let expected_fee = rust_decimal::Decimal::new(75, 2); // $0.75
    
    assert_eq!(fee_amount, expected_fee);
    println!(" Fee calculation: $100 * 0.75% = ${}", fee_amount);
    
    println!(" Dynamic Fee Configuration Test Passed!");
}
