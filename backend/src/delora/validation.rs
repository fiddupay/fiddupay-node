// Delora Calldata Validation
// Security checks on quote responses before exposing to frontend or executing

use crate::delora::constants::known_router_contracts;
use crate::delora::error::DeloraError;
use crate::delora::models::Calldata;

/// Validates that a quote's calldata is safe.
/// Called before returning quote data to the frontend.
pub fn validate_quote_calldata(
    calldata: &Calldata,
    origin_chain_id: u64,
) -> Result<(), DeloraError> {
    // 1. Non-empty fields
    if calldata.to.is_empty() {
        return Err(DeloraError::InvalidCalldata("Empty 'to' address".into()));
    }
    if calldata.data.is_empty() {
        return Err(DeloraError::InvalidCalldata("Empty 'data' field".into()));
    }

    // 2. Valid hex format (EVM: 0x..)
    let is_evm = calldata.to.starts_with("0x");

    if is_evm {
        if calldata.to.len() != 42 {
            return Err(DeloraError::InvalidCalldata(format!(
                "Invalid EVM address length: got {} chars, expected 42",
                calldata.to.len()
            )));
        }
        if !calldata.data.starts_with("0x") {
            return Err(DeloraError::InvalidCalldata(
                "EVM calldata must start with 0x".into(),
            ));
        }
        // Verify hex characters
        if !calldata.to[2..].chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(DeloraError::InvalidCalldata(
                "Invalid hex in 'to' address".into(),
            ));
        }
    }

    // 3. Router contract verification
    // Check the calldata.to is a known Delora router for this chain
    let routers = known_router_contracts();
    if let Some(expected_router) = routers.get(&origin_chain_id) {
        let normalized_to = calldata.to.to_lowercase();
        let normalized_expected = expected_router.to_lowercase();
        if normalized_to != normalized_expected {
            return Err(DeloraError::InvalidRouterContract {
                got: calldata.to.clone(),
                chain_id: origin_chain_id,
            });
        }
    }
    // If chain_id not in our known list, skip router check (new chains added by Delora)

    // 4. Value must be valid hex or "0x0"
    if is_evm && calldata.value != "0x0" && !calldata.value.starts_with("0x") {
        return Err(DeloraError::InvalidCalldata("Invalid value format".into()));
    }

    Ok(())
}

/// Verify that the merchant's destination address matches what's encoded in the quote.
/// For Delora, the merchant address is passed as receiverAddress in the quote request,
/// which Delora encodes into the router contract calldata. We verify indirectly by:
/// 1. Validating the router contract (above)
/// 2. Ensuring we passed the correct receiverAddress in the request (done in DeloraService)
pub fn verify_destination_consistency(
    calldata: &Calldata,
    origin_chain_id: u64,
) -> Result<(), DeloraError> {
    // The calldata 'to' must be a known router contract
    validate_quote_calldata(calldata, origin_chain_id)
}

/// Validate transaction hash format (EVM: 0x + 64 hex, Solana: base58)
pub fn validate_tx_hash(hash: &str) -> bool {
    if hash.is_empty() {
        return false;
    }

    // EVM format: 0x + 64 hex chars
    if let Some(stripped) = hash.strip_prefix("0x") {
        return stripped.len() == 64 && stripped.chars().all(|c| c.is_ascii_hexdigit());
    }

    // Solana format: base58, 87-88 chars typically
    if hash.len() >= 87 && hash.len() <= 88 {
        return hash
            .chars()
            .all(|c| c.is_alphanumeric() && c != '0' && c != 'O' && c != 'I' && c != 'l');
    }

    false
}
