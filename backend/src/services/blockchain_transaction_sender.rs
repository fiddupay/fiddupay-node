// Blockchain Transaction Sender
// Handles actual transaction broadcasting for address-only forwarding

use crate::error::ServiceError;
use crate::payment::models::CryptoType;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use web3::{
    signing::Key,
    transports::Http,
    types::{Address, TransactionParameters, U256},
    Web3,
};

pub struct BlockchainTransactionSender {
    config: crate::config::Config,
}

impl BlockchainTransactionSender {
    pub fn new(config: crate::config::Config) -> Self {
        Self { config }
    }

    fn get_evm_rpc_urls(&self, crypto_type: &CryptoType, sandbox_mode: bool) -> Vec<(String, u64)> {
        match (crypto_type, sandbox_mode) {
            (CryptoType::Eth | CryptoType::UsdtEth, false) => vec![
                (
                    self.config.ethereum_rpc_url.clone(),
                    self.config.ethereum_chain_id,
                ),
                (
                    self.config.ethereum_rpc_url_backup.clone(),
                    self.config.ethereum_chain_id,
                ),
            ],
            (CryptoType::Eth | CryptoType::UsdtEth, true) => vec![
                (
                    self.config.ethereum_sepolia_rpc_url.clone(),
                    self.config.ethereum_sepolia_chain_id,
                ),
                (
                    self.config.ethereum_sepolia_rpc_url_backup.clone(),
                    self.config.ethereum_sepolia_chain_id,
                ),
            ],
            (CryptoType::Bnb | CryptoType::UsdtBep20 | CryptoType::BusdBep20, false) => vec![
                (self.config.bsc_rpc_url.clone(), self.config.bsc_chain_id),
                (
                    self.config.bsc_rpc_url_backup.clone(),
                    self.config.bsc_chain_id,
                ),
            ],
            (CryptoType::Bnb | CryptoType::UsdtBep20 | CryptoType::BusdBep20, true) => vec![
                (
                    self.config.bsc_testnet_rpc_url.clone(),
                    self.config.bsc_testnet_chain_id,
                ),
                (
                    self.config.bsc_testnet_rpc_url_backup.clone(),
                    self.config.bsc_testnet_chain_id,
                ),
            ],
            (CryptoType::Matic | CryptoType::UsdtPolygon, false) => vec![
                (
                    self.config.polygon_rpc_url.clone(),
                    self.config.polygon_chain_id,
                ),
                (
                    self.config.polygon_rpc_url_backup.clone(),
                    self.config.polygon_chain_id,
                ),
            ],
            (CryptoType::Matic | CryptoType::UsdtPolygon, true) => vec![
                (
                    self.config.polygon_amoy_rpc_url.clone(),
                    self.config.polygon_amoy_chain_id,
                ),
                (
                    self.config.polygon_amoy_rpc_url_backup.clone(),
                    self.config.polygon_amoy_chain_id,
                ),
            ],
            (CryptoType::Arb | CryptoType::UsdtArbitrum, false) => vec![
                (
                    self.config.arbitrum_rpc_url.clone(),
                    self.config.arbitrum_chain_id,
                ),
                (
                    self.config.arbitrum_rpc_url_backup.clone(),
                    self.config.arbitrum_chain_id,
                ),
            ],
            (CryptoType::Arb | CryptoType::UsdtArbitrum, true) => vec![
                (
                    self.config.arbitrum_sepolia_rpc_url.clone(),
                    self.config.arbitrum_sepolia_chain_id,
                ),
                (
                    self.config.arbitrum_sepolia_rpc_url_backup.clone(),
                    self.config.arbitrum_sepolia_chain_id,
                ),
            ],
            _ => vec![],
        }
    }

    fn get_solana_rpc_urls(&self, sandbox_mode: bool) -> Vec<String> {
        if sandbox_mode {
            vec![
                self.config.solana_devnet_rpc_url.clone(),
                self.config.solana_devnet_rpc_url_backup.clone(),
                self.config.solana_devnet_rpc_url_backup_2.clone(),
            ]
        } else {
            vec![
                self.config.solana_rpc_url.clone(),
                self.config.solana_rpc_url_backup.clone(),
                self.config.solana_rpc_url_backup_2.clone(),
            ]
        }
    }

    /// Send the transaction (routes to native or token transfer based on crypto_type)
    pub async fn send_transaction(
        &self,
        crypto_type: CryptoType,
        private_key: &str,
        to_address: &str,
        amount: Decimal,
        gas_price: Option<U256>,
        sandbox_mode: bool,
    ) -> Result<String, ServiceError> {
        let is_solana = crypto_type.network() == "SOLANA";
        let is_solana_native = crypto_type == CryptoType::Sol;
        let is_spl = is_solana && !is_solana_native;
        let is_bitcoin = crypto_type.network() == "BITCOIN";

        let is_evm_token = !is_solana && !is_bitcoin && !crypto_type.is_native_currency();

        if is_solana_native {
            self.send_solana_transaction(private_key, to_address, amount, sandbox_mode)
                .await
        } else if is_bitcoin {
            self.send_bitcoin_transaction(private_key, to_address, amount, sandbox_mode)
                .await
        } else if is_spl {
            let mint = crypto_type.token_address().ok_or_else(|| {
                ServiceError::ValidationError("Missing SPL mint address".to_string())
            })?;
            self.send_solana_token_transaction(&mint, private_key, to_address, amount, sandbox_mode)
                .await
        } else if is_evm_token {
            let token_address = crypto_type.token_address().ok_or_else(|| {
                ServiceError::ValidationError("Missing token contract address".to_string())
            })?;
            self.send_evm_token_transaction(
                crypto_type,
                &token_address,
                private_key,
                to_address,
                amount,
                gas_price,
                sandbox_mode,
            )
            .await
        } else {
            // It's a native EVM currency
            self.send_evm_transaction(
                crypto_type,
                private_key,
                to_address,
                amount,
                gas_price,
                sandbox_mode,
            )
            .await
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
            pubkey::Pubkey,
            signature::{Keypair, Signer},
        };
        use std::str::FromStr;

        // Parse sender private key (expected as base58 string)
        let sender_keypair = Keypair::from_base58_string(private_key);

        // Parse destination address
        let to_pubkey = Pubkey::from_str(to_address).map_err(|_| {
            ServiceError::ValidationError("Invalid Solana destination address".to_string())
        })?;

        let lamports = (amount * Decimal::new(1_000_000_000, 0))
            .to_u64()
            .ok_or_else(|| ServiceError::ValidationError("Invalid SOL amount".to_string()))?;

        if lamports == 0 {
            return Err(ServiceError::ValidationError(
                "Amount must be greater than 0".to_string(),
            ));
        }

        let rpc_urls = self.get_solana_rpc_urls(sandbox_mode);
        let mut last_err = None;

        for url in rpc_urls {
            tracing::info!("[SOLANA] Attempting transaction via RPC: {}", url);
            let rpc_client = RpcClient::new(url.clone());

            // Get latest blockhash
            let recent_blockhash = match rpc_client.get_latest_blockhash().await {
                Ok(bh) => bh,
                Err(e) => {
                    tracing::warn!("[SOLANA] RPC {} failed (blockhash): {}", url, e);
                    last_err = Some(e);
                    continue;
                }
            };

            use solana_sdk::transaction::Transaction;
            use solana_system_interface::instruction as system_instruction;

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

            // Simulate
            if let Err(e) = rpc_client.simulate_transaction(&tx).await {
                tracing::warn!("[SOLANA] Simulation failed on {}: {}", url, e);
                last_err = Some(e);
                continue;
            }

            // Send and confirm
            match rpc_client.send_and_confirm_transaction(&tx).await {
                Ok(sig) => return Ok(sig.to_string()),
                Err(e) => {
                    tracing::warn!("[SOLANA] Send failed on {}: {}", url, e);
                    last_err = Some(e);
                    continue;
                }
            }
        }

        Err(ServiceError::Internal(format!(
            "All Solana RPCs failed. Last error: {:?}",
            last_err
        )))
    }

    /// Send Solana SPL Token transaction
    async fn send_solana_token_transaction(
        &self,
        mint_address: &str,
        private_key: &str,
        to_address: &str,
        amount: Decimal,
        sandbox_mode: bool,
    ) -> Result<String, ServiceError> {
        use solana_client::nonblocking::rpc_client::RpcClient;
        use solana_sdk::{
            instruction::Instruction,
            pubkey::Pubkey,
            signature::{Keypair, Signer},
            transaction::Transaction,
        };
        use solana_system_interface::instruction as system_instruction;
        use spl_associated_token_account::{
            get_associated_token_address, instruction::create_associated_token_account_idempotent,
        };
        use spl_token::instruction::transfer_checked;
        use std::str::FromStr;

        let sender_keypair = Keypair::from_base58_string(private_key);

        let to_pubkey = Pubkey::from_str(to_address).map_err(|_| {
            ServiceError::ValidationError("Invalid destination address".to_string())
        })?;

        let mint_pubkey = Pubkey::from_str(mint_address)
            .map_err(|_| ServiceError::ValidationError("Invalid mint address".to_string()))?;

        // Most SPL tokens have 6 decimals (like USDT), but we need to check mint info dynamically
        // or hardcode based on known assets. We will use 6 for stablecoins and 9 for WSOL.
        let decimals: u8 = if mint_address == "So11111111111111111111111111111111111111112" {
            9
        } else {
            6
        };
        let multiplier = 10u64.pow(decimals as u32);

        let token_amount = (amount * Decimal::new(multiplier as i64, 0))
            .to_u64()
            .ok_or_else(|| ServiceError::ValidationError("Invalid token amount".to_string()))?;

        if token_amount == 0 {
            return Err(ServiceError::ValidationError(
                "Amount must be greater than 0".to_string(),
            ));
        }

        let rpc_urls = self.get_solana_rpc_urls(sandbox_mode);
        let mut last_err = None;

        for url in rpc_urls {
            tracing::info!("[SOLANA-SPL] Attempting transaction via RPC: {}", url);
            let rpc_client = RpcClient::new(url.clone());

            let sender_pubkey = sender_keypair.pubkey();
            let source_ata = get_associated_token_address(&sender_pubkey, &mint_pubkey);
            let destination_ata = get_associated_token_address(&to_pubkey, &mint_pubkey);

            let mut instructions: Vec<Instruction> = Vec::new();

            // Check accounts
            let dest_account = rpc_client.get_account(&destination_ata).await;
            if dest_account.is_err() {
                instructions.push(create_associated_token_account_idempotent(
                    &sender_pubkey,
                    &to_pubkey,
                    &mint_pubkey,
                    &spl_token::id(),
                ));
            }

            instructions.push(
                transfer_checked(
                    &spl_token::id(),
                    &source_ata,
                    &mint_pubkey,
                    &destination_ata,
                    &sender_keypair.pubkey(),
                    &[&sender_keypair.pubkey()],
                    token_amount,
                    decimals,
                )
                .map_err(|e| ServiceError::Internal(format!("Failed to build transfer: {}", e)))?,
            );

            let recent_blockhash = match rpc_client.get_latest_blockhash().await {
                Ok(bh) => bh,
                Err(e) => {
                    last_err = Some(e);
                    continue;
                }
            };

            let tx = Transaction::new_signed_with_payer(
                &instructions,
                Some(&sender_keypair.pubkey()),
                &[&sender_keypair],
                recent_blockhash,
            );

            if let Err(e) = rpc_client.simulate_transaction(&tx).await {
                last_err = Some(e);
                continue;
            }

            match rpc_client.send_and_confirm_transaction(&tx).await {
                Ok(sig) => return Ok(sig.to_string()),
                Err(e) => {
                    last_err = Some(e);
                    continue;
                }
            }
        }

        Err(ServiceError::Internal(format!(
            "All Solana RPCs failed. Last error: {:?}",
            last_err
        )))
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
        let rpc_configs = self.get_evm_rpc_urls(&crypto_type, sandbox_mode);
        let mut last_err = None;

        for (url, chain_id) in rpc_configs {
            tracing::info!("[EVM] Attempting transaction via RPC: {}", url);
            let transport = match Http::new(&url) {
                Ok(t) => t,
                Err(e) => {
                    last_err = Some(e.to_string());
                    continue;
                }
            };
            let web3 = Web3::new(transport);

            // Key setup
            let private_key_clean = private_key.strip_prefix("0x").unwrap_or(private_key);
            let key_bytes = match hex::decode(private_key_clean) {
                Ok(b) => b,
                Err(_) => return Err(ServiceError::ValidationError("Invalid private key".into())),
            };
            let secret_key = match web3::signing::SecretKey::from_slice(&key_bytes) {
                Ok(sk) => sk,
                Err(_) => return Err(ServiceError::ValidationError("Invalid private key".into())),
            };
            let from_address = (&secret_key).address();
            let to_address_parsed: Address = to_address
                .parse()
                .map_err(|_| ServiceError::ValidationError("Invalid dest addr".into()))?;

            // Convert Decimal to native amount using crypto_type.decimals()
            let wei_amount = (amount * Decimal::from(10u64.pow(crypto_type.decimals())))
                .to_u128()
                .unwrap_or(0);

            // Nonce & Gas
            let nonce = match web3.eth().transaction_count(from_address, None).await {
                Ok(n) => n,
                Err(e) => {
                    last_err = Some(e.to_string());
                    continue;
                }
            };
            let gas_price_val = match gas_price {
                Some(p) => p,
                None => match web3.eth().gas_price().await {
                    Ok(p) => p,
                    Err(e) => {
                        last_err = Some(e.to_string());
                        continue;
                    }
                },
            };

            let tx_params = TransactionParameters {
                nonce: Some(nonce),
                to: Some(to_address_parsed),
                value: U256::from(wei_amount),
                gas_price: Some(gas_price_val),
                gas: U256::from(21000),
                chain_id: Some(chain_id),
                ..Default::default()
            };

            let signed_tx = match web3
                .accounts()
                .sign_transaction(tx_params, &secret_key)
                .await
            {
                Ok(s) => s,
                Err(e) => {
                    last_err = Some(e.to_string());
                    continue;
                }
            };

            match web3
                .eth()
                .send_raw_transaction(signed_tx.raw_transaction)
                .await
            {
                Ok(hash) => return Ok(format!("0x{:x}", hash)),
                Err(e) => {
                    last_err = Some(e.to_string());
                    continue;
                }
            }
        }

        Err(ServiceError::Internal(format!(
            "All EVM RPCs failed. Last error: {:?}",
            last_err
        )))
    }

    /// Send EVM Token (ERC20/BEP20) transaction
    async fn send_evm_token_transaction(
        &self,
        crypto_type: CryptoType,
        token_address_str: &str,
        private_key: &str,
        to_address: &str,
        amount: Decimal,
        gas_price: Option<U256>,
        sandbox_mode: bool,
    ) -> Result<String, ServiceError> {
        let rpc_configs = self.get_evm_rpc_urls(&crypto_type, sandbox_mode);
        let mut last_err = None;

        for (url, chain_id) in rpc_configs {
            tracing::info!("[EVM-TOKEN] Attempting transaction via RPC: {}", url);
            let transport = match Http::new(&url) {
                Ok(t) => t,
                Err(e) => {
                    last_err = Some(e.to_string());
                    continue;
                }
            };
            let web3 = Web3::new(transport);

            // Setup
            let private_key_clean = private_key.strip_prefix("0x").unwrap_or(private_key);
            let key_bytes = match hex::decode(private_key_clean) {
                Ok(b) => b,
                Err(_) => return Err(ServiceError::ValidationError("Invalid secret".into())),
            };
            let secret_key = match web3::signing::SecretKey::from_slice(&key_bytes) {
                Ok(sk) => sk,
                Err(_) => return Err(ServiceError::ValidationError("Invalid secret".into())),
            };
            let from_address = (&secret_key).address();
            let to_address_parsed: Address = to_address
                .parse()
                .map_err(|_| ServiceError::ValidationError("Invalid to addr".into()))?;
            let token_contract_address: Address = token_address_str
                .parse()
                .map_err(|_| ServiceError::ValidationError("Invalid token addr".into()))?;

            // Convert Decimal to token amount using crypto_type.decimals()
            let token_amount = (amount * Decimal::from(10u64.pow(crypto_type.decimals())))
                .to_u128()
                .unwrap_or(0);

            let nonce = match web3.eth().transaction_count(from_address, None).await {
                Ok(n) => n,
                Err(e) => {
                    last_err = Some(e.to_string());
                    continue;
                }
            };
            let gas_price_val = match gas_price {
                Some(p) => p,
                None => match web3.eth().gas_price().await {
                    Ok(p) => p,
                    Err(e) => {
                        last_err = Some(e.to_string());
                        continue;
                    }
                },
            };

            // Data
            let mut data = vec![0xa9, 0x05, 0x9c, 0xbb];
            let mut padded_to = vec![0u8; 32];
            padded_to[12..32].copy_from_slice(to_address_parsed.as_bytes());
            data.extend(padded_to);
            let mut padded_amount = vec![0u8; 32];
            U256::from(token_amount).to_big_endian(&mut padded_amount);
            data.extend(padded_amount);

            let tx_params = TransactionParameters {
                nonce: Some(nonce),
                to: Some(token_contract_address),
                value: U256::zero(),
                gas_price: Some(gas_price_val),
                gas: U256::from(65000),
                chain_id: Some(chain_id),
                data: web3::types::Bytes(data),
                ..Default::default()
            };

            let signed_tx = match web3
                .accounts()
                .sign_transaction(tx_params, &secret_key)
                .await
            {
                Ok(s) => s,
                Err(e) => {
                    last_err = Some(e.to_string());
                    continue;
                }
            };

            match web3
                .eth()
                .send_raw_transaction(signed_tx.raw_transaction)
                .await
            {
                Ok(hash) => return Ok(format!("0x{:x}", hash)),
                Err(e) => {
                    last_err = Some(e.to_string());
                    continue;
                }
            }
        }

        Err(ServiceError::Internal(format!(
            "All EVM Token RPCs failed. Last error: {:?}",
            last_err
        )))
    }

    /// Estimate gas for transaction
    pub async fn estimate_gas(
        &self,
        crypto_type: CryptoType,
        _from: &str,
        _to: &str,
        _amount: Decimal,
    ) -> Result<U256, ServiceError> {
        match crypto_type.network() {
            "SOLANA" => Ok(U256::from(1)), // Solana uses flat fees, treat limit as 1 signature
            "BITCOIN" => Ok(U256::from(1)), // Bitcoin fees handled differently, but treat as 1 unit for generic scaling
            _ => {
                // EVM Networks
                if crypto_type.is_native_currency() {
                    Ok(U256::from(21000)) // Standard ETH transfer
                } else {
                    Ok(U256::from(65000)) // Standard ERC20/Token transfer
                }
            }
        }
    }

    /// Query the actual on-chain native balance of an address
    pub async fn get_native_balance(
        &self,
        crypto_type: CryptoType,
        address: &str,
        sandbox_mode: bool,
    ) -> Result<U256, ServiceError> {
        let is_solana = crypto_type.network() == "SOLANA";

        if is_solana {
            use solana_client::nonblocking::rpc_client::RpcClient;
            use solana_sdk::pubkey::Pubkey;
            use std::str::FromStr;

            let rpc_url = if sandbox_mode {
                self.config.solana_devnet_rpc_url.clone()
            } else {
                self.config.solana_rpc_url.clone()
            };
            let rpc_client = RpcClient::new(rpc_url);

            let to_pubkey = Pubkey::from_str(address)
                .map_err(|_| ServiceError::ValidationError("Invalid Solana address".to_string()))?;

            let balance = rpc_client.get_balance(&to_pubkey).await.map_err(|e| {
                ServiceError::Internal(format!("Failed to get Solana balance: {}", e))
            })?;

            Ok(U256::from(balance))
        } else {
            let (rpc_url, _) = match (crypto_type.clone(), sandbox_mode) {
                (CryptoType::Eth | CryptoType::UsdtEth, false) => {
                    (&self.config.ethereum_rpc_url, self.config.ethereum_chain_id)
                }
                (CryptoType::Eth | CryptoType::UsdtEth, true) => (
                    &self.config.ethereum_sepolia_rpc_url,
                    self.config.ethereum_sepolia_chain_id,
                ),
                (CryptoType::Bnb | CryptoType::UsdtBep20 | CryptoType::BusdBep20, false) => {
                    (&self.config.bsc_rpc_url, self.config.bsc_chain_id)
                }
                (CryptoType::Bnb | CryptoType::UsdtBep20 | CryptoType::BusdBep20, true) => (
                    &self.config.bsc_testnet_rpc_url,
                    self.config.bsc_testnet_chain_id,
                ),
                (CryptoType::Matic | CryptoType::UsdtPolygon, false) => {
                    (&self.config.polygon_rpc_url, self.config.polygon_chain_id)
                }
                (CryptoType::Matic | CryptoType::UsdtPolygon, true) => (
                    &self.config.polygon_amoy_rpc_url,
                    self.config.polygon_amoy_chain_id,
                ),
                (CryptoType::Arb | CryptoType::UsdtArbitrum, false) => {
                    (&self.config.arbitrum_rpc_url, self.config.arbitrum_chain_id)
                }
                (CryptoType::Arb | CryptoType::UsdtArbitrum, true) => (
                    &self.config.arbitrum_sepolia_rpc_url,
                    self.config.arbitrum_sepolia_chain_id,
                ),
                _ => {
                    return Err(ServiceError::ValidationError(
                        "Unsupported network for balance query".to_string(),
                    ))
                }
            };

            let transport = Http::new(rpc_url)
                .map_err(|e| ServiceError::Internal(format!("Failed to connect to EVM: {}", e)))?;
            let web3 = Web3::new(transport);

            let addr: Address = address
                .parse()
                .map_err(|_| ServiceError::ValidationError("Invalid EVM address".to_string()))?;
            let balance = web3
                .eth()
                .balance(addr, None)
                .await
                .map_err(|e| ServiceError::Internal(format!("Failed EVM balance: {}", e)))?;
            Ok(balance)
        }
    }

    /// Get current gas price for an EVM network
    pub async fn get_current_gas_price(
        &self,
        crypto_type: CryptoType,
        sandbox_mode: bool,
    ) -> Result<U256, ServiceError> {
        let (rpc_url, _) = match (crypto_type.clone(), sandbox_mode) {
            (CryptoType::Eth, false) => {
                (&self.config.ethereum_rpc_url, self.config.ethereum_chain_id)
            }
            (CryptoType::Eth, true) => (
                &self.config.ethereum_sepolia_rpc_url,
                self.config.ethereum_sepolia_chain_id,
            ),
            (CryptoType::Bnb, false) => (&self.config.bsc_rpc_url, self.config.bsc_chain_id),
            (CryptoType::Bnb, true) => (
                &self.config.bsc_testnet_rpc_url,
                self.config.bsc_testnet_chain_id,
            ),
            (CryptoType::Matic, false) => {
                (&self.config.polygon_rpc_url, self.config.polygon_chain_id)
            }
            (CryptoType::Matic, true) => (
                &self.config.polygon_amoy_rpc_url,
                self.config.polygon_amoy_chain_id,
            ),
            (CryptoType::Arb, false) => {
                (&self.config.arbitrum_rpc_url, self.config.arbitrum_chain_id)
            }
            (CryptoType::Arb, true) => (
                &self.config.arbitrum_sepolia_rpc_url,
                self.config.arbitrum_sepolia_chain_id,
            ),
            (CryptoType::Sol | CryptoType::UsdtSpl | CryptoType::WSol, _) => {
                // For Solana, return the base fee (5000 lamports) as the "price"
                // This ensures generic (price * limit) logic works: 5000 * 1 = 5000 lamports
                return Ok(U256::from(5000));
            }
            _ => {
                return Err(ServiceError::ValidationError(format!(
                    "Gas price query not implemented for network: {}",
                    crypto_type.network()
                )))
            }
        };

        let transport = Http::new(rpc_url)
            .map_err(|e| ServiceError::Internal(format!("Failed to create transport: {}", e)))?;
        let web3 = Web3::new(transport);

        web3.eth()
            .gas_price()
            .await
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

        let fee_calculator = rpc_client
            .get_fee_for_message(&solana_sdk::message::Message::default())
            .await
            .unwrap_or(5000); // 5000 is default base fee

        Ok(fee_calculator)
    }

    /// Send Bitcoin transaction (Builds, signs, and broadcasts via Blockstream API)
    async fn send_bitcoin_transaction(
        &self,
        private_key: &str,
        to_address: &str,
        amount: Decimal,
        sandbox_mode: bool,
    ) -> Result<String, ServiceError> {
        use bitcoin::blockdata::transaction::Version;
        use bitcoin::locktime::absolute::LockTime;
        use bitcoin::sighash::{EcdsaSighashType, SighashCache};
        use bitcoin::{
            Address, Network, OutPoint, PrivateKey, ScriptBuf, Sequence, Transaction, TxIn, TxOut,
            Witness,
        };
        use std::str::FromStr;

        let network = if sandbox_mode {
            Network::Testnet
        } else {
            Network::Bitcoin
        };
        let api_config =
            crate::utils::bitcoin_api::BitcoinApiConfig::from_config(&self.config, sandbox_mode);

        let pk = PrivateKey::from_wif(private_key)
            .map_err(|e| ServiceError::Internal(format!("Invalid BTC Private Key: {}", e)))?;

        let secp = bitcoin::key::Secp256k1::new();
        let pubkey = pk.public_key(&secp);

        let compressed_public_key =
            bitcoin::CompressedPublicKey::try_from(pubkey).map_err(|_| {
                ServiceError::Internal("Failed to create compressed public key".to_string())
            })?;
        let from_address = Address::p2wpkh(&compressed_public_key, network);

        // 1. Fetch UTXOs
        let utxos: Vec<serde_json::Value> = crate::utils::bitcoin_api::get_with_failover(
            &api_config,
            &format!("address/{}/utxo", from_address),
        )
        .await
        .map_err(|e| ServiceError::Internal(format!("Failed to fetch UTXOs: {}", e)))
        .and_then(|v| {
            v.as_array()
                .cloned()
                .ok_or_else(|| ServiceError::Internal("Invalid UTXO response format".to_string()))
        })?;

        // 2. Select UTXOs
        let target_sats = (amount * Decimal::new(100_000_000, 0))
            .to_u64()
            .unwrap_or(0);
        let mut selected_utxos = Vec::new();
        let mut total_input_sats = 0u64;
        let fee_sats = 1500u64; // Flat estimate or vsize weight multiplied by rate

        for utxo in utxos {
            let value = utxo["value"].as_u64().unwrap_or(0);
            let txid = utxo["txid"].as_str().unwrap_or("");
            let vout = utxo["vout"].as_u64().unwrap_or(0) as u32;

            if value > 0 && !txid.is_empty() {
                selected_utxos.push((txid.to_string(), vout, value));
                total_input_sats += value;
                if total_input_sats >= target_sats + fee_sats {
                    break;
                }
            }
        }

        if total_input_sats < target_sats + fee_sats {
            return Err(ServiceError::ValidationError(format!(
                "Insufficient BTC balance. Need {} sats, have {} sats",
                target_sats + fee_sats,
                total_input_sats
            )));
        }

        // 3. Build Transaction
        let mut tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: Vec::new(),
            output: Vec::new(),
        };

        // Inputs
        for (txid_str, vout, _) in &selected_utxos {
            let txid = bitcoin::Txid::from_str(txid_str)
                .map_err(|_| ServiceError::Internal("Invalid Txid".to_string()))?;
            tx.input.push(TxIn {
                previous_output: OutPoint { txid, vout: *vout },
                script_sig: ScriptBuf::new(),
                sequence: Sequence::MAX,
                witness: Witness::new(),
            });
        }

        // Outputs
        use bitcoin::Amount;
        let dest_addr = Address::from_str(to_address)
            .map_err(|_| ServiceError::ValidationError("Invalid destination addr".to_string()))?
            .require_network(network)
            .map_err(|_| ServiceError::ValidationError("Address network mismatch".to_string()))?;

        tx.output.push(TxOut {
            value: Amount::from_sat(target_sats),
            script_pubkey: dest_addr.script_pubkey(),
        });

        // Change output
        let change_sats = total_input_sats - target_sats - fee_sats;
        if change_sats > 546 {
            // Dust limit
            tx.output.push(TxOut {
                value: Amount::from_sat(change_sats),
                script_pubkey: from_address.script_pubkey(),
            });
        }

        // 4. Sign inputs (P2WPKH SegWit signatures)
        let mut signatures = Vec::new();
        let sighash_all = EcdsaSighashType::All;

        {
            let mut cache = SighashCache::new(&tx);
            let pubkey_hash = compressed_public_key.wpubkey_hash();
            let script_code = ScriptBuf::new_p2wpkh(&pubkey_hash);

            for (idx, (_, _, value)) in selected_utxos.iter().enumerate() {
                let sighash = cache
                    .p2wpkh_signature_hash(idx, &script_code, Amount::from_sat(*value), sighash_all)
                    .map_err(|e| ServiceError::Internal(format!("Sighash error: {}", e)))?;

                let sig = secp.sign_ecdsa(
                    &bitcoin::secp256k1::Message::from_slice(&sighash[..]).unwrap(),
                    &pk.inner,
                );
                signatures.push(sig);
            }
        }

        // Apply witnesses
        for (idx, sig) in signatures.into_iter().enumerate() {
            let mut witness = Witness::new();
            witness.push(sig.serialize_der().to_vec());
            witness.push(pubkey.to_bytes());
            tx.input[idx].witness = witness;
        }

        // 5. Broadcast via failover
        use bitcoin::consensus::encode::serialize_hex;
        let tx_hex = serialize_hex(&tx);

        let txid = crate::utils::bitcoin_api::post_tx_with_failover(&api_config, &tx_hex)
            .await
            .map_err(|e| ServiceError::Internal(e))?;

        Ok(txid)
    }
}
