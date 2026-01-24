//! # Axiom SDK
//!
//! Official Rust SDK for Axiom Protocol - Privacy-First Blockchain
//!
//! ## Features
//! - ✅ Wallet management (create, import, export)
//! - ✅ Transaction creation with ZK-SNARK privacy
//! - ✅ Cross-chain bridge integration
//! - ✅ View keys for compliance
//! - ✅ RPC client for node communication
//! - ✅ Type-safe API
//!
//! ## Quick Start
//!
//! ```no_run
//! use axiom_sdk::{AxiomClient, Wallet, NetworkConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to Axiom network
//!     let config = NetworkConfig::mainnet();
//!     let client = AxiomClient::new(&config.rpc_url).await?;
//!     
//!     // Create wallet
//!     let wallet = Wallet::new();
//!     println!("Address: {}", wallet.address_hex());
//!     
//!     // Check balance
//!     let balance = client.get_balance(&wallet.address()).await?;
//!     println!("Balance: {} AXM", balance.as_axm());
//!     
//!     // Send transaction
//!     let tx = wallet.create_transaction(
//!         "axm1recipient...",
//!         1_000_000_000, // 1 AXM (in satoshis)
//!         100_000_000,   // 0.1 AXM fee
//!     )?;
//!     
//!     let hash = client.broadcast_transaction(tx).await?;
//!     println!("TX: {}", hash);
//!     
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod wallet;
pub mod transaction;
pub mod types;
pub mod error;

pub use client::AxiomClient;
pub use wallet::Wallet;
pub use transaction::Transaction;
pub use types::{Address, Balance, TxHash};
pub use error::{AxiomError, Result};

/// SDK version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Axiom network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub chain_id: u64,
    pub rpc_url: String,
    pub explorer_url: String,
}

impl NetworkConfig {
    /// Mainnet configuration
    pub fn mainnet() -> Self {
        Self {
            chain_id: 84000,
            rpc_url: "https://rpc.axiom.network".to_string(),
            explorer_url: "https://explorer.axiom.network".to_string(),
        }
    }
    
    /// Mainnet US-East endpoint
    pub fn mainnet_us() -> Self {
        Self {
            chain_id: 84000,
            rpc_url: "https://rpc-us.axiom.network".to_string(),
            explorer_url: "https://explorer.axiom.network".to_string(),
        }
    }
    
    /// Custom network
    pub fn custom(chain_id: u64, rpc_url: String, explorer_url: String) -> Self {
        Self {
            chain_id,
            rpc_url,
            explorer_url,
        }
    }
}

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{
        AxiomClient,
        Wallet,
        Transaction,
        Address,
        Balance,
        NetworkConfig,
        AxiomError,
        Result,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
    
    #[test]
    fn test_network_config() {
        let mainnet = NetworkConfig::mainnet();
        assert_eq!(mainnet.chain_id, 84000);
        
        let mainnet_us = NetworkConfig::mainnet_us();
        assert_eq!(mainnet_us.chain_id, 84000);
    }
}
