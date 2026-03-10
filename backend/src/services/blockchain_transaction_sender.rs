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
        sandbox_mode: bool,
    ) -> Result<String, ServiceError> {
        match crypto_type {
            CryptoType::Sol => self.send_solana_transaction(private_key, to_address, amount, sandbox_mode).await,
            _ => self.send_evm_transaction(crypto_type, private_key, to_address, amount, gas_price, sandbox_mode).await,
        }
    }

    async fn send_solana_transaction(
        &self,
        private_key: &str,
        to_address: &str,
        amount: Decimal,
        sandbox_mode: bool,
    ) -> Result<String, ServiceError> {
        use solana_client::nonblocking::rpc_client::RpcClient;
        use solana_sdk::{
            signature::{Keypair, Signer},
            pubkey::Pubkey,
        };
        use std::str::FromStr;

        // Parse sender private key (expected as base58 string)
        let sender_keypair = Keypair::from_base58_string(private_key);

        // Parse destination address
        let to_pubkey = Pubkey::from_str(to_address)
            .map_err(|_| ServiceError::ValidationError("Invalid Solana destination address".to_string()))?;

        // Convert SOL amount to lamports (1 SOL = 1,000,000,000 lamports)
        let lamports = (amount * Decimal::new(1_000_000_000, 0))
            .to_u64()
            .ok_or_else(|| ServiceError::ValidationError("Invalid SOL amount".to_string()))?;

        if lamports == 0 {
            return Err(ServiceError::ValidationError("Amount must be greater than 0".to_string()));
        }

        // Initialize non-blocking RPC client based on sandbox mode
        let rpc_url = if sandbox_mode {
            self.config.solana_devnet_rpc_url.clone()
        } else {
            self.config.solana_rpc_url.clone()
        };
        let rpc_client = RpcClient::new(rpc_url);

        // Get latest blockhash for transaction signing
        let recent_blockhash = rpc_client.get_latest_blockhash()
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to get Solana blockhash: {}", e)))?;

        use solana_sdk::system_instruction;
        use solana_sdk::transaction::Transaction;

        // Create the native SOL transfer transaction
        let instructions = vec![system_instruction::transfer(
            &sender_keypair.pubkey(),
            &to_pubkey,
            lamports,
        )];
        let tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&sender_keypair.pubkey()),
            &[&sender_keypair],
            recent_blockhash,
        );

        // Send and confirm the transaction on-chain
        let signature = rpc_client.send_and_confirm_transaction(&tx)
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to send Solana transaction: {}", e)))?;

        Ok(signature.to_string())
    }

    /// Send EVM transaction (ETH, BNB, MATIC, ARB)
    async fn send_evm_transaction(
        &self,
        crypto_type: CryptoType,
        private_key: &str,
        to_address: &str,
        amount: Decimal,
        gas_price: Option<U256>,
        sandbox_mode: bool,
    ) -> Result<String, ServiceError> {
        let (rpc_url, chain_id) = match (crypto_type, sandbox_mode) {
            (CryptoType::Eth, false) => (&self.config.ethereum_rpc_url, self.config.ethereum_chain_id),
            (CryptoType::Eth, true) => (&self.config.ethereum_sepolia_rpc_url, self.config.ethereum_sepolia_chain_id),
            (CryptoType::Bnb, false) => (&self.config.bsc_rpc_url, self.config.bsc_chain_id),
            (CryptoType::Bnb, true) => (&self.config.bsc_testnet_rpc_url, self.config.bsc_testnet_chain_id),
            (CryptoType::Matic, false) => (&self.config.polygon_rpc_url, self.config.polygon_chain_id),
            (CryptoType::Matic, true) => (&self.config.polygon_mumbai_rpc_url, self.config.polygon_mumbai_chain_id),
            (CryptoType::Arb, false) => (&self.config.arbitrum_rpc_url, self.config.arbitrum_chain_id),
            (CryptoType::Arb, true) => (&self.config.arbitrum_sepolia_rpc_url, self.config.arbitrum_sepolia_chain_id),
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
            chain_id: Some(chain_id),
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

    /// Get current gas price for an EVM network
    pub async fn get_current_gas_price(
        &self,
        crypto_type: CryptoType,
        sandbox_mode: bool,
    ) -> Result<U256, ServiceError> {
        let (rpc_url, _) = match (crypto_type.clone(), sandbox_mode) {
            (CryptoType::Eth, false) => (&self.config.ethereum_rpc_url, self.config.ethereum_chain_id),
            (CryptoType::Eth, true) => (&self.config.ethereum_sepolia_rpc_url, self.config.ethereum_sepolia_chain_id),
            (CryptoType::Bnb, false) => (&self.config.bsc_rpc_url, self.config.bsc_chain_id),
            (CryptoType::Bnb, true) => (&self.config.bsc_testnet_rpc_url, self.config.bsc_testnet_chain_id),
            (CryptoType::Matic, false) => (&self.config.polygon_rpc_url, self.config.polygon_chain_id),
            (CryptoType::Matic, true) => (&self.config.polygon_mumbai_rpc_url, self.config.polygon_mumbai_chain_id),
            (CryptoType::Arb, false) => (&self.config.arbitrum_rpc_url, self.config.arbitrum_chain_id),
            (CryptoType::Arb, true) => (&self.config.arbitrum_sepolia_rpc_url, self.config.arbitrum_sepolia_chain_id),
            _ => return Err(ServiceError::ValidationError("Unsupported or non-EVM network for gas query".to_string())),
        };

        let transport = Http::new(rpc_url)
            .map_err(|e| ServiceError::Internal(format!("Failed to create transport: {}", e)))?;
        let web3 = Web3::new(transport);

        web3.eth().gas_price().await
            .map_err(|e| ServiceError::Internal(format!("Failed to get gas price: {}", e)))
    }

    /// Get Solana recent blockhash fee calculator (simplified base fee)
    pub async fn get_solana_fee(&self, sandbox_mode: bool) -> Result<u64, ServiceError> {
        use solana_client::nonblocking::rpc_client::RpcClient;

        let rpc_url = if sandbox_mode {
            self.config.solana_devnet_rpc_url.clone()
        } else {
            self.config.solana_rpc_url.clone()
        };
        let rpc_client = RpcClient::new(rpc_url);

        let fee_calculator = rpc_client.get_fee_for_message(&solana_sdk::message::Message::default())
            .await
            .unwrap_or(5000); // 5000 is default base fee

        Ok(fee_calculator)
    }
}
