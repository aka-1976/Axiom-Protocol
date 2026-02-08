// Production modules
pub mod error;
pub mod config;
pub mod mempool;

// Core modules
pub mod zk;
pub mod consensus; // VDF consensus implementation
pub mod ai; // AI Oracle network

// NEW: Guardian and AI Security (v2.1.0+)
pub mod guardian; // Immutable safety manifest
pub mod ai_core; // Multi-layer security engine
pub mod guardian_enhancement; // AI-Guardian bridge
// Re-export modules and wallet so they can be used by bin crates
pub mod transaction;
pub mod main_helper;
pub mod block;
pub mod genesis;
pub mod chain;
pub mod state;
pub mod economics;
pub mod wallet;
pub mod vdf;
pub mod ai_engine;
pub mod bridge;
pub mod time;
pub mod storage;
pub mod network;
pub mod network_legacy; // Legacy network implementation with TimechainBehaviour
pub mod network_config; // NEW: Network configuration and peer discovery
pub mod guardian_sentinel; // NEW: Sovereign Guardian sentinel with eternal monitoring
pub mod neural_guardian; // NEW: AI-powered security with federated learning
pub mod openclaw_integration; // NEW: OpenClaw automation integration

// 2026 Best Practices Modules
pub mod privacy; // View keys & selective disclosure
pub mod sustainability; // Energy benchmarking & reporting
pub mod mobile; // Mobile mining with 1 AXM rewards

pub use wallet::Wallet;
pub use block::Block;

// Re-export 124M economics constants
pub use economics::{
    TOTAL_SUPPLY,
    INITIAL_REWARD,
    HALVING_INTERVAL,
    BLOCK_TIME_SECONDS,
    ERA_DURATION_YEARS,
    PROTOCOL_NAME,
    TICKER,
    CREATOR,
    get_mining_reward,
    calculate_total_supply,
    remaining_supply,
    supply_percentage,
    current_era,
    blocks_until_halving,
    format_supply_stats,
    validate_economics,
    NetworkPhase,
};

// Re-export LWMA difficulty functions
pub use consensus::{
    calculate_lwma_difficulty,
    TARGET_BLOCK_TIME,
    LWMA_WINDOW,
    estimate_hashrate,
    format_hashrate,
};

// Re-export production types
pub use error::{AxiomError, Result};
pub use config::AxiomConfig;

// Note: vdf and main_helper are already public via `pub mod` declarations above
// No need to re-export them - this caused E0255 duplicate definition errors
