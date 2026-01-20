// Fee Calculator
// Handles fee calculation and validation logic

use rust_decimal::Decimal;
use crate::error::ServiceError;

/// Fee Calculator for payment processing
/// 
/// Provides fee calculation and validation logic to ensure fees are within
/// acceptable bounds and calculated correctly based on fiat values.
pub struct FeeCalculator;

impl FeeCalculator {
    /// Minimum allowed fee percentage (0.1%)
    const MIN_FEE_PERCENTAGE: &'static str = "0.10";
    
    /// Maximum allowed fee percentage (5.0%)
    const MAX_FEE_PERCENTAGE: &'static str = "5.00";

    /// Validate that a fee percentage is within acceptable bounds
    /// 
    /// Ensures the fee percentage is between 0.1% and 5.0% as per requirements.
    /// 
    /// # Arguments
    /// * `fee_percentage` - Fee percentage to validate (e.g., 1.50 for 1.5%)
    /// 
    /// # Returns
    /// * `Ok(())` if fee percentage is valid
    /// * `Err(ServiceError::InvalidFeePercentage)` if out of bounds
    /// 
    /// # Requirements
    /// * 6.4: Support fee percentages between 0.1% and 5%
    pub fn validate_fee_percentage(fee_percentage: Decimal) -> Result<(), ServiceError> {
        let min_fee = Decimal::from_str_exact(Self::MIN_FEE_PERCENTAGE)
            .map_err(|e| ServiceError::Internal(format!("Failed to parse min fee: {}", e)))?;
        let max_fee = Decimal::from_str_exact(Self::MAX_FEE_PERCENTAGE)
            .map_err(|e| ServiceError::Internal(format!("Failed to parse max fee: {}", e)))?;
        
        if fee_percentage < min_fee {
            return Err(ServiceError::InvalidFeePercentage(
                format!("Fee percentage {} is below minimum of {}%", fee_percentage, Self::MIN_FEE_PERCENTAGE)
            ));
        }
        
        if fee_percentage > max_fee {
            return Err(ServiceError::InvalidFeePercentage(
                format!("Fee percentage {} exceeds maximum of {}%", fee_percentage, Self::MAX_FEE_PERCENTAGE)
            ));
        }
        
        Ok(())
    }

    /// Calculate fee amount in USD based on payment amount and fee percentage
    /// 
    /// Calculates the fee based on the fiat value at payment creation time.
    /// 
    /// # Arguments
    /// * `amount_usd` - Base payment amount in USD
    /// * `fee_percentage` - Fee percentage (e.g., 1.50 for 1.5%)
    /// 
    /// # Returns
    /// * Fee amount in USD
    /// 
    /// # Requirements
    /// * 6.1: Calculate fees and add to total amount
    /// * 6.6: Calculate fees based on fiat value at creation time
    pub fn calculate_fee_usd(amount_usd: Decimal, fee_percentage: Decimal) -> Decimal {
        amount_usd * fee_percentage / Decimal::new(100, 0)
    }

    /// Calculate fee amount in cryptocurrency based on USD fee and crypto price
    /// 
    /// Converts the USD fee amount to cryptocurrency amount using the current price.
    /// For stablecoins (USDT), the amount is 1:1 with USD.
    /// 
    /// # Arguments
    /// * `fee_amount_usd` - Fee amount in USD
    /// * `crypto_price` - Current price of cryptocurrency in USD (use 1.0 for stablecoins)
    /// 
    /// # Returns
    /// * Fee amount in cryptocurrency
    /// 
    /// # Requirements
    /// * 6.1: Calculate fees in both USD and crypto
    pub fn calculate_fee_crypto(fee_amount_usd: Decimal, crypto_price: Decimal) -> Decimal {
        fee_amount_usd / crypto_price
    }

    /// Calculate total payment amount including fees
    /// 
    /// Adds the fee to the base amount to get the total amount the customer must pay.
    /// 
    /// # Arguments
    /// * `base_amount_usd` - Base payment amount in USD
    /// * `fee_amount_usd` - Fee amount in USD
    /// 
    /// # Returns
    /// * Total amount in USD (base + fee)
    /// 
    /// # Requirements
    /// * 2.6: Include platform fee in total amount
    /// * 6.1: Add fee to payment total
    pub fn calculate_total_with_fee(base_amount_usd: Decimal, fee_amount_usd: Decimal) -> Decimal {
        base_amount_usd + fee_amount_usd
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_validate_fee_percentage_valid() {
        // Test valid fee percentages
        let valid_fees = vec![
            Decimal::new(10, 2),   // 0.10% (minimum)
            Decimal::new(150, 2),  // 1.50% (typical)
            Decimal::new(250, 2),  // 2.50%
            Decimal::new(500, 2),  // 5.00% (maximum)
        ];
        
        for fee in valid_fees {
            let result = FeeCalculator::validate_fee_percentage(fee);
            assert!(result.is_ok(), "Fee {} should be valid", fee);
        }
    }

    #[test]
    fn test_validate_fee_percentage_below_minimum() {
        // Test fee below minimum (0.1%)
        let too_low = Decimal::new(9, 2); // 0.09%
        let result = FeeCalculator::validate_fee_percentage(too_low);
        assert!(result.is_err());
        
        match result {
            Err(ServiceError::InvalidFeePercentage(msg)) => {
                assert!(msg.contains("below minimum"));
            }
            _ => panic!("Expected InvalidFeePercentage error"),
        }
    }

    #[test]
    fn test_validate_fee_percentage_above_maximum() {
        // Test fee above maximum (5%)
        let too_high = Decimal::new(501, 2); // 5.01%
        let result = FeeCalculator::validate_fee_percentage(too_high);
        assert!(result.is_err());
        
        match result {
            Err(ServiceError::InvalidFeePercentage(msg)) => {
                assert!(msg.contains("exceeds maximum"));
            }
            _ => panic!("Expected InvalidFeePercentage error"),
        }
    }

    #[test]
    fn test_validate_fee_percentage_zero() {
        // Test zero fee (below minimum)
        let zero_fee = Decimal::ZERO;
        let result = FeeCalculator::validate_fee_percentage(zero_fee);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_fee_percentage_negative() {
        // Test negative fee (invalid)
        let negative_fee = Decimal::new(-100, 2);
        let result = FeeCalculator::validate_fee_percentage(negative_fee);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_fee_usd_typical() {
        // Test typical fee calculation: $100 payment with 1.5% fee
        let amount_usd = Decimal::new(10000, 2); // $100.00
        let fee_percentage = Decimal::new(150, 2); // 1.50%
        
        let fee_amount = FeeCalculator::calculate_fee_usd(amount_usd, fee_percentage);
        
        assert_eq!(fee_amount, Decimal::new(150, 2)); // $1.50
    }

    #[test]
    fn test_calculate_fee_usd_minimum() {
        // Test minimum fee: $100 payment with 0.1% fee
        let amount_usd = Decimal::new(10000, 2); // $100.00
        let fee_percentage = Decimal::new(10, 2); // 0.10%
        
        let fee_amount = FeeCalculator::calculate_fee_usd(amount_usd, fee_percentage);
        
        assert_eq!(fee_amount, Decimal::new(10, 2)); // $0.10
    }

    #[test]
    fn test_calculate_fee_usd_maximum() {
        // Test maximum fee: $100 payment with 5% fee
        let amount_usd = Decimal::new(10000, 2); // $100.00
        let fee_percentage = Decimal::new(500, 2); // 5.00%
        
        let fee_amount = FeeCalculator::calculate_fee_usd(amount_usd, fee_percentage);
        
        assert_eq!(fee_amount, Decimal::new(500, 2)); // $5.00
    }

    #[test]
    fn test_calculate_fee_usd_large_amount() {
        // Test with large payment amount: $10,000 with 2.5% fee
        let amount_usd = Decimal::new(1000000, 2); // $10,000.00
        let fee_percentage = Decimal::new(250, 2); // 2.50%
        
        let fee_amount = FeeCalculator::calculate_fee_usd(amount_usd, fee_percentage);
        
        assert_eq!(fee_amount, Decimal::new(25000, 2)); // $250.00
    }

    #[test]
    fn test_calculate_fee_crypto_stablecoin() {
        // Test fee calculation for stablecoin (1:1 with USD)
        let fee_amount_usd = Decimal::new(150, 2); // $1.50
        let crypto_price = Decimal::new(1, 0); // $1.00 (USDT)
        
        let fee_crypto = FeeCalculator::calculate_fee_crypto(fee_amount_usd, crypto_price);
        
        assert_eq!(fee_crypto, Decimal::new(150, 2)); // 1.50 USDT
    }

    #[test]
    fn test_calculate_fee_crypto_non_stablecoin() {
        // Test fee calculation for non-stablecoin (SOL at $50)
        let fee_amount_usd = Decimal::new(150, 2); // $1.50
        let crypto_price = Decimal::new(5000, 2); // $50.00 per SOL
        
        let fee_crypto = FeeCalculator::calculate_fee_crypto(fee_amount_usd, crypto_price);
        
        assert_eq!(fee_crypto, Decimal::new(3, 2)); // 0.03 SOL
    }

    #[test]
    fn test_calculate_fee_crypto_high_price() {
        // Test with high crypto price (e.g., BTC at $50,000)
        let fee_amount_usd = Decimal::new(150, 2); // $1.50
        let crypto_price = Decimal::new(5000000, 2); // $50,000.00
        
        let fee_crypto = FeeCalculator::calculate_fee_crypto(fee_amount_usd, crypto_price);
        
        // $1.50 / $50,000 = 0.00003
        assert_eq!(fee_crypto, Decimal::new(3, 5)); // 0.00003
    }

    #[test]
    fn test_calculate_total_with_fee() {
        // Test total calculation: $100 base + $1.50 fee = $101.50
        let base_amount = Decimal::new(10000, 2); // $100.00
        let fee_amount = Decimal::new(150, 2); // $1.50
        
        let total = FeeCalculator::calculate_total_with_fee(base_amount, fee_amount);
        
        assert_eq!(total, Decimal::new(10150, 2)); // $101.50
    }

    #[test]
    fn test_calculate_total_with_fee_zero_fee() {
        // Test total with zero fee (edge case)
        let base_amount = Decimal::new(10000, 2); // $100.00
        let fee_amount = Decimal::ZERO;
        
        let total = FeeCalculator::calculate_total_with_fee(base_amount, fee_amount);
        
        assert_eq!(total, base_amount); // $100.00
    }

    #[test]
    fn test_fee_calculation_precision() {
        // Test that fee calculation maintains precision
        let amount_usd = Decimal::new(9999, 2); // $99.99
        let fee_percentage = Decimal::new(175, 2); // 1.75%
        
        let fee_amount = FeeCalculator::calculate_fee_usd(amount_usd, fee_percentage);
        
        // $99.99 * 1.75% = $1.749825
        // Should maintain precision
        assert!(fee_amount > Decimal::new(174, 2)); // > $1.74
        assert!(fee_amount < Decimal::new(176, 2)); // < $1.76
    }

    #[test]
    fn test_end_to_end_fee_calculation() {
        // Test complete fee calculation flow
        let base_amount_usd = Decimal::new(10000, 2); // $100.00
        let fee_percentage = Decimal::new(150, 2); // 1.50%
        
        // Validate fee percentage
        assert!(FeeCalculator::validate_fee_percentage(fee_percentage).is_ok());
        
        // Calculate fee in USD
        let fee_amount_usd = FeeCalculator::calculate_fee_usd(base_amount_usd, fee_percentage);
        assert_eq!(fee_amount_usd, Decimal::new(150, 2)); // $1.50
        
        // Calculate total
        let total_usd = FeeCalculator::calculate_total_with_fee(base_amount_usd, fee_amount_usd);
        assert_eq!(total_usd, Decimal::new(10150, 2)); // $101.50
        
        // Calculate fee in crypto (SOL at $50)
        let crypto_price = Decimal::new(5000, 2);
        let fee_crypto = FeeCalculator::calculate_fee_crypto(fee_amount_usd, crypto_price);
        assert_eq!(fee_crypto, Decimal::new(3, 2)); // 0.03 SOL
    }

    #[test]
    fn test_boundary_values() {
        // Test at exact boundaries
        let min_fee = Decimal::new(10, 2); // 0.10%
        let max_fee = Decimal::new(500, 2); // 5.00%
        
        assert!(FeeCalculator::validate_fee_percentage(min_fee).is_ok());
        assert!(FeeCalculator::validate_fee_percentage(max_fee).is_ok());
        
        // Just below minimum
        let below_min = Decimal::new(9, 2); // 0.09%
        assert!(FeeCalculator::validate_fee_percentage(below_min).is_err());
        
        // Just above maximum
        let above_max = Decimal::new(501, 2); // 5.01%
        assert!(FeeCalculator::validate_fee_percentage(above_max).is_err());
    }
}
