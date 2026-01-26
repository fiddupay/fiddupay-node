// Blockchain Transaction Sender
// Handles actual transaction broadcasting for address-only forwarding

use crate::error::ServiceError;
use crate::payment::models::CryptoType;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use web3::{
    transports::Http,
    types::{Address, TransactionParameters, U256},
    signing::Key,
    Web3,
};

pub struct BlockchainTransactionSender {
    config: crate::config::Config,
}

impl BlockchainTransactionSender {
    pub fn new(config: crate::config::Config) -> Self {
        Self { config }
    }

    /// Send native currency transaction
    pub async fn send_native_transaction(
        &self,
        crypto_type: CryptoType,
        private_key: &str,
        to_address: &str,
        amount: Decimal,
        gas_price: Option<U256>,
    ) -> Result<String, ServiceError> {
        match crypto_type {
            CryptoType::Sol => self.send_solana_transaction_placeholder(private_key, to_address, amount).await,
            _ => self.send_evm_transaction(crypto_type, private_key, to_address, amount, gas_price).await,
        }
    }

    /// Send Solana transaction (placeholder - requires solana-sdk)
    async fn send_solana_transaction_placeholder(
        &self,
        _private_key: &str,
        _to_address: &str,
        amount: Decimal,
    ) -> Result<String, ServiceError> {
        // Placeholder implementation - would need solana-sdk
        let lamports = amount.to_u64().unwrap_or(0);
        tracing::info!("Placeholder Solana transaction: {} lamports", lamports);
        Ok(format!("sol_tx_{}", uuid::Uuid::new_v4()))
    }

    /// Send EVM transaction (ETH, BNB, MATIC, ARB)
    async fn send_evm_transaction(
        &self,
        crypto_type: CryptoType,
        private_key: &str,
        to_address: &str,
        amount: Decimal,
        gas_price: Option<U256>,
    ) -> Result<String, ServiceError> {
        let rpc_url = match crypto_type {
            CryptoType::Eth => &self.config.ethereum_rpc_url,
            CryptoType::Bnb => &self.config.bsc_rpc_url,
            CryptoType::Matic => &self.config.polygon_rpc_url,
            CryptoType::Arb => &self.config.arbitrum_rpc_url,
            _ => return Err(ServiceError::ValidationError("Unsupported EVM network".to_string())),
        };

        // Create web3 transport
        let transport = Http::new(rpc_url)
            .map_err(|e| ServiceError::Internal(format!("Failed to create transport: {}", e)))?;
        let web3 = Web3::new(transport);

        // Parse private key
        let private_key = private_key.strip_prefix("0x").unwrap_or(private_key);
        let key_bytes = hex::decode(private_key)
            .map_err(|_| ServiceError::ValidationError("Invalid private key hex".to_string()))?;
        
        // Use web3's SecretKey type directly
        let secret_key_bytes: [u8; 32] = key_bytes.try_into()
            .map_err(|_| ServiceError::ValidationError("Invalid key length".to_string()))?;
        let secret_key = web3::signing::SecretKey::from_slice(&secret_key_bytes)
            .map_err(|_| ServiceError::ValidationError("Invalid private key".to_string()))?;

        // Get sender address from secret key
        let from_address = (&secret_key).address();

        // Parse destination address
        let to_address: Address = to_address.parse()
            .map_err(|_| ServiceError::ValidationError("Invalid destination address".to_string()))?;

        // Convert amount to wei
        let wei_amount = (amount * Decimal::new(1_000_000_000_000_000_000i64, 0))
            .to_u128()
            .ok_or_else(|| ServiceError::ValidationError("Invalid amount".to_string()))?;

        // Get nonce
        let nonce = web3.eth()
            .transaction_count(from_address, None)
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to get nonce: {}", e)))?;

        // Get gas price if not provided
        let gas_price = match gas_price {
            Some(price) => price,
            None => web3.eth()
                .gas_price()
                .await
                .map_err(|e| ServiceError::Internal(format!("Failed to get gas price: {}", e)))?,
        };

        // Create transaction parameters
        let tx_params = TransactionParameters {
            nonce: Some(nonce),
            to: Some(to_address),
            value: U256::from(wei_amount),
            gas_price: Some(gas_price),
            gas: U256::from(21000), // Standard gas limit for ETH transfer
            data: web3::types::Bytes::default(),
            ..Default::default()
        };

        let signed_tx = web3.accounts()
            .sign_transaction(tx_params, &secret_key)
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to sign transaction: {}", e)))?;

        let tx_hash = web3.eth()
            .send_raw_transaction(signed_tx.raw_transaction)
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to send transaction: {}", e)))?;

        Ok(format!("0x{:x}", tx_hash))
    }

    /// Estimate gas for transaction
    pub async fn estimate_gas(
        &self,
        crypto_type: CryptoType,
        _from: &str,
        _to: &str,
        _amount: Decimal,
    ) -> Result<U256, ServiceError> {
        match crypto_type {
            CryptoType::Sol => Ok(U256::from(5000)), // Base fee in lamports
            _ => Ok(U256::from(21000)), // Standard gas limit for EVM
        }
    }
}
