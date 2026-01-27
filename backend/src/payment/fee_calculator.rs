use rust_decimal::Decimal;
use crate::error::ServiceError;

/// Fee calculator for payment processing
/// 
/// Handles fee calculations and validation to ensure fees are calculated correctly.
pub struct FeeCalculator;

impl FeeCalculator {
    /// Validate that a fee percentage is positive
    /// 
    /// Ensures the fee percentage is greater than 0.
    /// 
    /// # Arguments
    /// * `fee_percentage` - Fee percentage to validate (e.g., 1.50 for 1.5%)
    /// 
    /// # Returns
    /// * `Ok(())` if fee percentage is valid
    /// * `Err(ServiceError::InvalidFeePercentage)` if invalid
    pub fn validate_fee_percentage(fee_percentage: Decimal) -> Result<(), ServiceError> {
        if fee_percentage <= Decimal::ZERO {
            return Err(ServiceError::InvalidFeePercentage(
                "Fee percentage must be greater than 0".to_string()
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
    /// * Fee amount in USD as Decimal
    /// 
    /// # Example
    /// ```
    /// use rust_decimal::Decimal;
    /// use fiddupay::payment::fee_calculator::FeeCalculator;
    /// 
    /// let amount = Decimal::new(10000, 2); // $100.00
    /// let fee_pct = Decimal::new(150, 2);  // 1.50%
    /// let fee = FeeCalculator::calculate_fee_usd(amount, fee_pct);
    /// assert_eq!(fee, Decimal::new(150, 2)); // $1.50
    /// ```
    pub fn calculate_fee_usd(amount_usd: Decimal, fee_percentage: Decimal) -> Decimal {
        amount_usd * fee_percentage / Decimal::new(100, 0)
    }

    /// Calculate the net amount after deducting fees
    /// 
    /// # Arguments
    /// * `gross_amount_usd` - Total payment amount in USD
    /// * `fee_percentage` - Fee percentage to deduct
    /// 
    /// # Returns
    /// * Net amount after fee deduction
    pub fn calculate_net_amount(gross_amount_usd: Decimal, fee_percentage: Decimal) -> Decimal {
        let fee = Self::calculate_fee_usd(gross_amount_usd, fee_percentage);
        gross_amount_usd - fee
    }

    /// Calculate total amount including fee
    /// 
    /// # Arguments
    /// * `base_amount_usd` - Base payment amount in USD
    /// * `fee_amount_usd` - Fee amount in USD
    /// 
    /// # Returns
    /// * Total amount including fee
    pub fn calculate_total_with_fee(base_amount_usd: Decimal, fee_amount_usd: Decimal) -> Decimal {
        base_amount_usd + fee_amount_usd
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_validate_fee_percentage() {
        // Valid fee percentages
        assert!(FeeCalculator::validate_fee_percentage(dec!(0.01)).is_ok());
        assert!(FeeCalculator::validate_fee_percentage(dec!(1.50)).is_ok());
        assert!(FeeCalculator::validate_fee_percentage(dec!(10.00)).is_ok());
        
        // Invalid fee percentages
        assert!(FeeCalculator::validate_fee_percentage(dec!(0.00)).is_err());
        assert!(FeeCalculator::validate_fee_percentage(dec!(-1.00)).is_err());
    }

    #[test]
    fn test_calculate_fee_usd() {
        let amount = dec!(100.00);
        let fee_pct = dec!(1.50);
        let expected_fee = dec!(1.50);
        
        assert_eq!(FeeCalculator::calculate_fee_usd(amount, fee_pct), expected_fee);
    }

    #[test]
    fn test_calculate_net_amount() {
        let gross = dec!(100.00);
        let fee_pct = dec!(2.50);
        let expected_net = dec!(97.50);
        
        assert_eq!(FeeCalculator::calculate_net_amount(gross, fee_pct), expected_net);
    }
}
