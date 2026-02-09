//! # Axiom SDK
//!
//! Official Rust SDK for Axiom Protocol - Privacy-First Blockchain
//!
//! ## Features
//! - ✅ Wallet management (create, import, export)
//! - ✅ Transaction creation with ZK-STARK privacy
//! - ✅ Cross-chain bridge integration
//! - ✅ View keys for compliance
//! - ✅ RPC client for node communication
//! - ✅ Type-safe API
//! - ✅ `#[axiom_contract]` macro for Provable-by-Default smart contracts
//! - ✅ `prove_transaction()` for local Proof-of-Execution via RISC-V
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
pub mod zk_pulse;

pub use client::AxiomClient;
pub use wallet::Wallet;
pub use transaction::Transaction;
pub use types::{Address, Balance, TxHash};
pub use error::{AxiomError, Result};
pub use zk_pulse::{ZkPulse, ProveTransactionInput, ProveTransactionOutput};

/// SDK version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default RISC-V build target for Axiom smart contracts.
/// All apps built on Axiom are "Provable by Default" using this target.
pub const RISCV_TARGET: &str = "riscv32im-unknown-none-elf";

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

// ---------------------------------------------------------------------------
// #[axiom_contract] — Attribute macro stub
// ---------------------------------------------------------------------------

/// Attribute macro that marks a function as an Axiom smart-contract handler.
///
/// Functions decorated with `#[axiom_contract]` are compiled to the
/// `riscv32im-unknown-none-elf` target, ensuring they execute inside the
/// RISC Zero zkVM for Proof-of-Execution.
///
/// # Example
///
/// ```ignore
/// #[axiom_contract]
/// fn transfer(from: Address, to: Address, amount: u64) -> Result<()> {
///     // This logic runs inside the zkVM.
///     // A STARK proof of correct execution is generated automatically.
///     Ok(())
/// }
/// ```
///
/// In the current release this is a no-op attribute; full procedural macro
/// expansion (compiling to RISC-V ELF and wrapping with a host prover) is
/// planned for the next SDK version.
#[macro_export]
macro_rules! axiom_contract {
    (
        $(#[$meta:meta])*
        $vis:vis fn $name:ident ( $($arg:ident : $argty:ty),* $(,)? ) $( -> $ret:ty )? $body:block
    ) => {
        $(#[$meta])*
        $vis fn $name ( $($arg : $argty),* ) $( -> $ret )? $body
    };
}

/// Convenience function: generate a local Proof-of-Execution for a
/// transaction without broadcasting it.
///
/// This compiles the user-defined logic into RISC-V, runs it inside the
/// zkVM, and returns a serialised STARK receipt. The receipt proves
/// correct execution without revealing private transaction data.
///
/// # Arguments
/// * `input` — The transaction input to prove.
///
/// # Returns
/// A `ProveTransactionOutput` containing the 512-bit BLAKE3 digest and
/// the serialised proof bytes.
pub fn prove_transaction(
    input: &ProveTransactionInput,
) -> Result<ProveTransactionOutput> {
    zk_pulse::ZkPulse::prove_transaction(input)
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
        ZkPulse,
        ProveTransactionInput,
        ProveTransactionOutput,
        prove_transaction,
        RISCV_TARGET,
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

    #[test]
    fn test_riscv_target() {
        assert_eq!(RISCV_TARGET, "riscv32im-unknown-none-elf");
    }

    #[test]
    fn test_axiom_contract_macro() {
        // Verify the macro expands correctly.
        axiom_contract! {
            fn add(a: u64, b: u64) -> u64 { a + b }
        }
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_prove_transaction_valid() {
        let input = ProveTransactionInput {
            initial_balance: 10_000,
            amount: 1_000,
            fee: 50,
            nonce: 1,
        };
        let output = prove_transaction(&input).unwrap();
        assert_eq!(output.digest_512.len(), 128); // 64 bytes = 128 hex chars
        assert!(!output.proof_bytes.is_empty());
    }

    #[test]
    fn test_prove_transaction_insufficient_balance() {
        let input = ProveTransactionInput {
            initial_balance: 50,
            amount: 100,
            fee: 10,
            nonce: 1,
        };
        assert!(prove_transaction(&input).is_err());
    }
}
