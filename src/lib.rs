pub mod zk;
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
pub mod neural_guardian; // NEW: AI-powered security with federated learning
pub use main_helper::Wallet;
pub use block::Block;
