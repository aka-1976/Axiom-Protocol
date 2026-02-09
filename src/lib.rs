// Production modules
pub mod error;
pub mod config;
pub mod mempool;

/// Compute a 512-bit (64-byte) BLAKE3 hash using extended output (XOF) mode.
///
/// This is the AXIOM standard for all protocol-level hashing, providing
/// 256-bit collision resistance and post-quantum alignment.
pub fn axiom_hash_512(data: &[u8]) -> [u8; 64] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(data);
    let mut output = [0u8; 64];
    hasher.finalize_xof().fill(&mut output);
    output
}

/// Hardcoded bootstrap multiaddresses for the AXIOM peer-to-peer mesh.
///
/// Seed diversity prevents a single-point-of-failure: if one bootstrap
/// node goes down the remaining seeds keep the network discoverable.
/// Nodes also use mDNS (local) and Kademlia DHT (global) for resilient
/// peer discovery beyond these seeds.
pub const BOOTSTRAP_NODES: &[&str] = &[
    "/ip4/34.10.172.20/tcp/6000",
    "/ip4/34.160.111.145/tcp/7000",
    "/ip4/51.15.23.200/tcp/7000",
    "/ip4/3.8.120.113/tcp/7000",
];

/// Serde helper for fixed-size 64-byte arrays (512-bit hashes).
///
/// `serde` only derives for arrays up to 32 elements, so we serialise
/// the 64 bytes as a `Vec<u8>` on the wire and validate the length on
/// deserialize.
mod serde_bytes_64 {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(bytes)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: Vec<u8> = Vec::deserialize(deserializer)?;
        v.try_into()
            .map_err(|v: Vec<u8>| serde::de::Error::custom(format!("expected 64 bytes, got {}", v.len())))
    }
}

/// Real-time network pulse for instant block/supply synchronization.
///
/// Broadcast via Gossipsub topic `axiom/realtime/pulse/v1` the instant a
/// new block is mined. Receivers verify freshness via the BLAKE3-512
/// `block_hash` before updating their local state.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AxiomPulse {
    /// Current block height
    pub height: u64,
    /// Cumulative AXM mined (in smallest units)
    pub total_mined: u64,
    /// Remaining supply: TOTAL_SUPPLY - total_mined
    pub remaining: u64,
    /// 512-bit BLAKE3 hash of the latest block
    #[serde(with = "serde_bytes_64")]
    pub block_hash: [u8; 64],
    /// 512-bit Deterministic AI Oracle seal
    #[serde(with = "serde_bytes_64")]
    pub oracle_seal: [u8; 64],
    /// 512-bit BLAKE3 hash of the previous pulse (tamper-evident chain)
    #[serde(with = "serde_bytes_64")]
    pub prev_pulse_hash: [u8; 64],
    /// Unix timestamp (seconds) for freshness check
    pub timestamp: i64,
}

// Core modules
pub mod zk;
pub mod stark; // RISC Zero zkVM STARK proving (124M supply integrity)
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

// Re-export genesis anchor
pub use genesis::GENESIS_ANCHOR_512;

/// SHA-256 fingerprint of the production NeuralGuardian `weights.bin`.
///
/// Every node **must** verify its local model file against this hash at
/// startup via [`neural_guardian::NeuralGuardian::load_model`].  If the
/// hashes diverge, the node panics — preventing tampered AI weights from
/// silently corrupting trust decisions on the 124M network.
///
/// To update this constant, compute:
/// ```text
/// sha256sum weights.bin
/// ```
/// and paste the hex digest below.
pub const GENESIS_WEIGHTS_HASH: &str =
    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

/// 512-bit BLAKE3 hash of the Genesis Pulse — the absolute origin of the
/// tamper-evident pulse chain.
///
/// At startup the node looks for `config/genesis_pulse.json`. If found,
/// its raw bytes are hashed with [`axiom_hash_512`] and compared to this
/// constant. A match means the pulse chain can be anchored all the way
/// back to block 0. If the file is absent, the node falls back to an
/// all-zeros `prev_pulse_hash` (unanchored start).
///
/// **Note:** This constant is initialised to all-zeros until the official
/// `genesis_pulse.json` is published as part of the mainnet release.
/// Once the file is generated, update this constant with the output of:
/// ```text
/// python3 -c "import blake3; print(blake3.blake3(open('config/genesis_pulse.json','rb').read()).hexdigest(length=64))"
/// ```
pub const GENESIS_PULSE_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000\
     0000000000000000000000000000000000000000000000000000000000000000";

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
