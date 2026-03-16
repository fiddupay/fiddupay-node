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
            self.send_solana_transaction(private_key, to_address, amount, sandbox_mode).await
        } else if is_bitcoin {
            self.send_bitcoin_transaction(private_key, to_address, amount, sandbox_mode).await
        } else if is_spl {
            let mint = crypto_type.token_address().ok_or_else(|| ServiceError::ValidationError("Missing SPL mint address".to_string()))?;
            self.send_solana_token_transaction(&mint, private_key, to_address, amount, sandbox_mode).await
        } else if is_evm_token {
            let token_address = crypto_type.token_address().ok_or_else(|| ServiceError::ValidationError("Missing token contract address".to_string()))?;
            self.send_evm_token_transaction(crypto_type, &token_address, private_key, to_address, amount, gas_price, sandbox_mode).await
        } else {
            // It's a native EVM currency
            self.send_evm_transaction(crypto_type, private_key, to_address, amount, gas_price, sandbox_mode).await
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

        // Run simulation before sending (Requirement: Catch errors early)
        match rpc_client.simulate_transaction(&tx).await {
            Ok(sim) => {
                if let Some(err) = sim.value.err {
                    tracing::error!("[SOLANA-SIM] Simulation failed for SOL transfer: {:?}", err);
                    return Err(ServiceError::Internal(format!("Transaction simulation failed: {}", err)));
                }
                tracing::info!("[SOLANA-SIM] Simulation successful for SOL transfer");
            },
            Err(e) => {
                tracing::error!("[SOLANA-SIM] Simulation RPC error: {}", e);
                return Err(ServiceError::Internal(format!("Failed to simulate transaction: {}", e)));
            }
        }

        // Send and confirm the transaction on-chain
        let signature = rpc_client.send_and_confirm_transaction(&tx)
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to send Solana transaction: {}", e)))?;

        Ok(signature.to_string())
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
            signature::{Keypair, Signer},
            pubkey::Pubkey,
            system_instruction,
            transaction::Transaction,
            instruction::Instruction,
        };
        use spl_token::instruction::transfer_checked;
        use std::str::FromStr;
        use spl_associated_token_account::{get_associated_token_address, instruction::create_associated_token_account_idempotent};

        let sender_keypair = Keypair::from_base58_string(private_key);
        
        let to_pubkey = Pubkey::from_str(to_address)
            .map_err(|_| ServiceError::ValidationError("Invalid destination address".to_string()))?;
            
        let mint_pubkey = Pubkey::from_str(mint_address)
            .map_err(|_| ServiceError::ValidationError("Invalid mint address".to_string()))?;

        // Most SPL tokens have 6 decimals (like USDT), but we need to check mint info dynamically
        // or hardcode based on known assets. We will use 6 for stablecoins and 9 for WSOL.
        let decimals: u8 = if mint_address == "So11111111111111111111111111111111111111112" { 9 } else { 6 };
        let multiplier = 10u64.pow(decimals as u32);
        
        let token_amount = (amount * Decimal::new(multiplier as i64, 0))
            .to_u64()
            .ok_or_else(|| ServiceError::ValidationError("Invalid token amount".to_string()))?;

        if token_amount == 0 {
            return Err(ServiceError::ValidationError("Amount must be greater than 0".to_string()));
        }

        let rpc_url = if sandbox_mode {
            self.config.solana_devnet_rpc_url.clone()
        } else {
            self.config.solana_rpc_url.clone()
        };
        let rpc_client = RpcClient::new(rpc_url);
        

        // Calculate ATAs
        let sender_pubkey = sender_keypair.pubkey();
        let source_ata = get_associated_token_address(&sender_pubkey, &mint_pubkey);
        let destination_ata = get_associated_token_address(&to_pubkey, &mint_pubkey);

        tracing::info!("[SOLANA-TRANSFER] Sender: {}, to: {}, mint: {}", sender_pubkey, to_pubkey, mint_pubkey);
        tracing::info!("[SOLANA-TRANSFER] Source ATA: {}, Destination ATA: {}", source_ata, destination_ata);

        // 1. Check if source ATA exists and has balance
        let source_account = rpc_client.get_account(&source_ata).await;
        if source_account.is_err() {
            tracing::error!("[SOLANA-TRANSFER] Source ATA {} does not exist. Withdrawal impossible without on-chain tokens.", source_ata);
            return Err(ServiceError::ValidationError(format!("Source token account not initialized. Ensure you have {} on-chain.", if mint_address == "So11111111111111111111111111111111111111112" { "WSOL" } else { "USDT" })));
        }

        let mut instructions: Vec<Instruction> = Vec::new();

        // 2. Check if destination ATA exists (Requirement: Idempotent creation)
        let dest_account = rpc_client.get_account(&destination_ata).await;
        if dest_account.is_err() {
            // Must create the destination ATA funded by the sender
            // We use the IDEMPOTENT variant to prevent failures if the account is created concurrently
            tracing::info!("[SOLANA-FEE] Destination ATA {} does not exist. Adding 'CreateATAIdempotent' instruction (Rent ≈ 0.002 SOL)", destination_ata);
            instructions.push(
                create_associated_token_account_idempotent(
                    &sender_pubkey,
                    &to_pubkey,
                    &mint_pubkey,
                    &spl_token::id(),
                )
            );
        }

        // Add the transfer_checked instruction
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
            ).map_err(|e| ServiceError::Internal(format!("Failed to build transfer instruction: {}", e)))?
        );

        let recent_blockhash = rpc_client.get_latest_blockhash()
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to get Solana blockhash: {}", e)))?;

        let tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&sender_keypair.pubkey()),
            &[&sender_keypair],
            recent_blockhash,
        );

        // Run simulation before sending (Requirement: Catch errors early)
        match rpc_client.simulate_transaction(&tx).await {
            Ok(sim) => {
                if let Some(err) = sim.value.err {
                    tracing::error!("[SOLANA-SIM] Simulation failed for token transfer: {:?}", err);
                    return Err(ServiceError::Internal(format!("Transaction simulation failed: {}. Ensure you have enough SOL for fees and enough tokens for withdrawal.", err)));
                }
                tracing::info!("[SOLANA-SIM] Simulation successful for token transfer");
            },
            Err(e) => {
                tracing::error!("[SOLANA-SIM] Simulation RPC error: {}", e);
                return Err(ServiceError::Internal(format!("Failed to simulate transaction: {}", e)));
            }
        }

        let signature = rpc_client.send_and_confirm_transaction(&tx)
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to send Solana token transaction: {}", e)))?;

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
        let (rpc_url, chain_id) = match (crypto_type.clone(), sandbox_mode) {
            (CryptoType::UsdtEth, false) => (&self.config.ethereum_rpc_url, self.config.ethereum_chain_id),
            (CryptoType::UsdtEth, true) => (&self.config.ethereum_sepolia_rpc_url, self.config.ethereum_sepolia_chain_id),
            (CryptoType::UsdtBep20, false) => (&self.config.bsc_rpc_url, self.config.bsc_chain_id),
            (CryptoType::UsdtBep20, true) => (&self.config.bsc_testnet_rpc_url, self.config.bsc_testnet_chain_id),
            (CryptoType::UsdtPolygon, false) => (&self.config.polygon_rpc_url, self.config.polygon_chain_id),
            (CryptoType::UsdtPolygon, true) => (&self.config.polygon_mumbai_rpc_url, self.config.polygon_mumbai_chain_id),
            (CryptoType::UsdtArbitrum, false) => (&self.config.arbitrum_rpc_url, self.config.arbitrum_chain_id),
            (CryptoType::UsdtArbitrum, true) => (&self.config.arbitrum_sepolia_rpc_url, self.config.arbitrum_sepolia_chain_id),
            _ => return Err(ServiceError::ValidationError("Unsupported EVM token network".to_string())),
        };

        let transport = Http::new(rpc_url)
            .map_err(|e| ServiceError::Internal(format!("Failed to create transport: {}", e)))?;
        let web3 = Web3::new(transport);

        let private_key = private_key.strip_prefix("0x").unwrap_or(private_key);
        let key_bytes = hex::decode(private_key)
            .map_err(|_| ServiceError::ValidationError("Invalid private key hex".to_string()))?;
        
        let secret_key_bytes: [u8; 32] = key_bytes.try_into()
            .map_err(|_| ServiceError::ValidationError("Invalid key length".to_string()))?;
        let secret_key = web3::signing::SecretKey::from_slice(&secret_key_bytes)
            .map_err(|_| ServiceError::ValidationError("Invalid private key".to_string()))?;

        let from_address = (&secret_key).address();

        let to_address_parsed: Address = to_address.parse()
            .map_err(|_| ServiceError::ValidationError("Invalid destination address".to_string()))?;

        let token_contract_address: Address = token_address_str.parse()
            .map_err(|_| ServiceError::ValidationError("Invalid token contract address".to_string()))?;

        // Most tokens like USDT/USDC have 6 decimals on EVM (except BSC where USDT BEP20 is 18)
        let decimals = if crypto_type == CryptoType::UsdtBep20 { 18 } else { 6 };
        let token_amount = (amount * Decimal::new(10i64.pow(decimals as u32), 0))
            .to_u128()
            .ok_or_else(|| ServiceError::ValidationError("Invalid token amount".to_string()))?;

        let nonce = web3.eth()
            .transaction_count(from_address, None)
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to get nonce: {}", e)))?;

        let gas_price = match gas_price {
            Some(price) => price,
            None => web3.eth()
                .gas_price()
                .await
                .map_err(|e| ServiceError::Internal(format!("Failed to get gas price: {}", e)))?,
        };

        // Construct ERC20 transfer(address to, uint256 amount) data
        // Method ID: 0xa9059cbb
        let mut data = vec![0xa9, 0x05, 0x9c, 0xbb];
        
        // Append padded 'to' address
        let mut padded_to = vec![0u8; 32];
        padded_to[12..32].copy_from_slice(to_address_parsed.as_bytes());
        data.extend(padded_to);

        // Append padded 'amount'
        let mut padded_amount = vec![0u8; 32];
        U256::from(token_amount).to_big_endian(&mut padded_amount);
        data.extend(padded_amount);

        let tx_params = TransactionParameters {
            nonce: Some(nonce),
            to: Some(token_contract_address), // Sent to the contract!
            value: U256::zero(),
            gas_price: Some(gas_price),
            gas: U256::from(65000), // Tokens need more gas (~65k)
            chain_id: Some(chain_id),
            data: web3::types::Bytes(data),
            ..Default::default()
        };

        let signed_tx = web3.accounts()
            .sign_transaction(tx_params, &secret_key)
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to sign token transaction: {}", e)))?;

        let tx_hash = web3.eth()
            .send_raw_transaction(signed_tx.raw_transaction)
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to send token transaction: {}", e)))?;

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

    /// Send Bitcoin transaction (Builds, signs, and broadcasts via Blockstream API)
    async fn send_bitcoin_transaction(
        &self,
        private_key: &str,
        to_address: &str,
        amount: Decimal,
        sandbox_mode: bool,
    ) -> Result<String, ServiceError> {
        use bitcoin::{Network, Address, PrivateKey, Transaction, TxIn, TxOut, OutPoint, ScriptBuf, Sequence, Witness};
        use bitcoin::sighash::{SighashCache, EcdsaSighashType};
        use bitcoin::blockdata::transaction::Version;
        use bitcoin::locktime::absolute::LockTime;
        use std::str::FromStr;

        let network = if sandbox_mode { Network::Testnet } else { Network::Bitcoin };
        let api_url = if sandbox_mode { "https://blockstream.info/testnet/api" } else { "https://blockstream.info/api" };

        let pk = PrivateKey::from_wif(private_key)
            .map_err(|e| ServiceError::Internal(format!("Invalid BTC Private Key: {}", e)))?;
        
        let secp = bitcoin::key::Secp256k1::new();
        let pubkey = pk.public_key(&secp);
        
        let compressed_public_key = bitcoin::CompressedPublicKey::try_from(pubkey)
            .map_err(|_| ServiceError::Internal("Failed to create compressed public key".to_string()))?;
        let from_address = Address::p2wpkh(&compressed_public_key, network);

        // 1. Fetch UTXOs
        let utxo_url = format!("{}/address/{}/utxo", api_url, from_address);
        let client = reqwest::Client::new();
        let utxos_resp = client.get(&utxo_url).send().await
            .map_err(|e| ServiceError::Internal(format!("Failed to fetch UTXOs: {}", e)))?;
        let utxos: Vec<serde_json::Value> = utxos_resp.json().await
            .map_err(|e| ServiceError::Internal(format!("Failed to parse UTXOs: {}", e)))?;

        // 2. Select UTXOs
        let target_sats = (amount * Decimal::new(100_000_000, 0)).to_u64().unwrap_or(0);
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
            return Err(ServiceError::ValidationError(format!("Insufficient BTC balance. Need {} sats, have {} sats", target_sats + fee_sats, total_input_sats)));
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
            let txid = bitcoin::Txid::from_str(txid_str).map_err(|_| ServiceError::Internal("Invalid Txid".to_string()))?;
            tx.input.push(TxIn {
                previous_output: OutPoint { txid, vout: *vout },
                script_sig: ScriptBuf::new(),
                sequence: Sequence::MAX,
                witness: Witness::new(),
            });
        }

        // Outputs
        use bitcoin::Amount;
        let dest_addr = Address::from_str(to_address).map_err(|_| ServiceError::ValidationError("Invalid destination addr".to_string()))?
            .require_network(network)
            .map_err(|_| ServiceError::ValidationError("Address network mismatch".to_string()))?;
            
        tx.output.push(TxOut {
            value: Amount::from_sat(target_sats),
            script_pubkey: dest_addr.script_pubkey(),
        });

        // Change output
        let change_sats = total_input_sats - target_sats - fee_sats;
        if change_sats > 546 { // Dust limit
            tx.output.push(TxOut {
                value: Amount::from_sat(change_sats),
                script_pubkey: from_address.script_pubkey(),
            });
        }

        // 4. Sign inputs (P2WPKH SegWit signatures)
        let mut signatures = Vec::new();
        let sighash_all = EcdsaSighashType::All;

        {
            let cache = SighashCache::new(&tx);
            let pubkey_hash = compressed_public_key.wpubkey_hash();
            let script_code = ScriptBuf::new_p2wpkh(&pubkey_hash);

            for (idx, (_, _, value)) in selected_utxos.iter().enumerate() {
                let sighash = cache.segwit_signature_hash(idx, &script_code, Amount::from_sat(*value), sighash_all)
                    .map_err(|e| ServiceError::Internal(format!("Sighash error: {}", e)))?;
                
                let sig = secp.sign_ecdsa(&bitcoin::secp256k1::Message::from_slice(&sighash[..]).unwrap(), &pk.inner);
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

        // 5. Broadcast
        use bitcoin::consensus::encode::serialize_hex;
        let tx_hex = serialize_hex(&tx);
        
        let broadcast_url = format!("{}/tx", api_url);
        let broadcast_resp = client.post(&broadcast_url)
            .body(tx_hex)
            .send()
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to broadcast: {}", e)))?;

        if !broadcast_resp.status().is_success() {
            let err_text = broadcast_resp.text().await.unwrap_or_default();
            return Err(ServiceError::Internal(format!("Broadcast failed: {}", err_text)));
        }

        Ok(tx.compute_txid().to_string())
    }
}
