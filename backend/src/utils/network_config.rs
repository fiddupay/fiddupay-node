// Network Configuration Utility
// Handles sandbox vs production network selection

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub solana_rpc_url: String,
    pub ethereum_rpc_url: String,
    pub bsc_rpc_url: String,
    pub arbitrum_rpc_url: String,
    pub polygon_rpc_url: String,
    pub ethereum_chain_id: u64,
    pub bsc_chain_id: u64,
    pub polygon_chain_id: u64,
    pub arbitrum_chain_id: u64,
}

impl NetworkConfig {
    /// Get network configuration based on sandbox mode
    pub fn for_sandbox_mode(config: &Config, is_sandbox: bool) -> Self {
        if is_sandbox {
            // Use testnet/devnet for sandbox
            Self {
                solana_rpc_url: config.solana_devnet_rpc_url.clone(),
                ethereum_rpc_url: config.ethereum_sepolia_rpc_url.clone(),
                bsc_rpc_url: config.bsc_testnet_rpc_url.clone(),
                arbitrum_rpc_url: config.arbitrum_sepolia_rpc_url.clone(),
                polygon_rpc_url: config.polygon_mumbai_rpc_url.clone(),
                ethereum_chain_id: config.ethereum_sepolia_chain_id,
                bsc_chain_id: config.bsc_testnet_chain_id,
                polygon_chain_id: config.polygon_mumbai_chain_id,
                arbitrum_chain_id: config.arbitrum_sepolia_chain_id,
            }
        } else {
            // Use mainnet for production
            Self {
                solana_rpc_url: config.solana_rpc_url.clone(),
                ethereum_rpc_url: config.ethereum_rpc_url.clone(),
                bsc_rpc_url: config.bsc_rpc_url.clone(),
                arbitrum_rpc_url: config.arbitrum_rpc_url.clone(),
                polygon_rpc_url: config.polygon_rpc_url.clone(),
                ethereum_chain_id: config.ethereum_chain_id,
                bsc_chain_id: config.bsc_chain_id,
                polygon_chain_id: config.polygon_chain_id,
                arbitrum_chain_id: config.arbitrum_chain_id,
            }
        }
    }

    /// Get network name for display
    pub fn network_name(&self, is_sandbox: bool) -> &'static str {
        if is_sandbox {
            "Testnet/Devnet"
        } else {
            "Mainnet"
        }
    }
}
