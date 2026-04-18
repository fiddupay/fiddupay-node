use rust_decimal::Decimal;

/// Formats a crypto amount for human-readable display.
///
/// This function:
/// 1. Strips trailing zeros (e.g., 0.0201500000 -> 0.02015)
/// 2. Limits precision to a maximum of 9 decimal places.
/// 3. Returns a clean string suitable for notifications and UI.
pub fn format_crypto_amount(amount: Decimal) -> String {
    let mut amount = amount.normalize();

    // Check if we need to round to 9 places (max standard for most chains)
    if amount.scale() > 9 {
        amount = amount.trunc_with_scale(9);
    }

    amount.to_string()
}
