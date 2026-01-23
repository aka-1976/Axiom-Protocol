// Bridge module - Cross-chain bridge functionality

pub mod cross_chain;
pub mod atomic_swap;

pub use cross_chain::{
    AxiomBridge, BridgeContract, BridgeOracle, BridgeStatus, BridgeTransaction, ChainId,
};

pub use atomic_swap::{BridgeLock, BridgeSecret};
