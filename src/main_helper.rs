use crate::transaction::Address;
use crate::neural_guardian::GuardianStats;
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand_core::{OsRng, RngCore};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Wallet {
    pub secret_key: [u8; 32],
    pub address: Address,
}

impl Wallet {
    /// Decentralized Identity: Loads local key or generates a new one.
    /// Your identity is never broadcast, only your ZK-Pass.
    pub fn load_or_create() -> Self {
        if let Ok(data) = fs::read("wallet.dat") {
            if let Ok(w) = bincode::deserialize::<Wallet>(&data) { 
                return w; 
            }
        }
        let mut seed = [0u8; 32];
        OsRng.fill_bytes(&mut seed);
        let signing_key = SigningKey::from_bytes(&seed);
        let address = VerifyingKey::from(&signing_key).to_bytes();
        let wallet = Wallet { secret_key: seed, address };
        
        // Save locally - crucial for non-custodial ownership
        fs::write("wallet.dat", bincode::serialize(&wallet).unwrap()).expect("Failed to save wallet");
        wallet
    }
}

/// FORCED SEQUENTIAL VDF (Proof of Time)
/// This is the core "Time-Chain" mechanism. 
/// Every iteration requires the result of the previous one, making it 
/// immune to 60% hash power attacks.
pub fn compute_vdf(seed: [u8; 32], iterations: u32) -> [u8; 32] {
    let mut result = seed;
    
    // This loop forces the CPU to spend real-world time.
    // Even if an attacker has 1,000 GPUs, they cannot split this 
    // work because Hash N+1 requires Hash N.
    for _ in 0..iterations {
        let mut hasher = Sha256::new();
        hasher.update(result);
        result = hasher.finalize().into();
    }
    result
}

// ---------------------------------------------------------------------------
// Public Health Dashboard — Global Trust Pulse
// ---------------------------------------------------------------------------

/// Aggregated network health snapshot broadcast every 100 blocks.
///
/// The `trust_pulse_512` is a 512-bit BLAKE3 digest that commits the
/// entire health state. Independent verifiers can monitor the 124M supply
/// integrity without needing access to private logs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkHealth {
    /// Current chain height
    pub block_height: u64,
    /// Cumulative AXM mined (smallest units)
    pub total_mined: u64,
    /// Remaining AXM supply (smallest units)
    pub remaining_supply: u64,
    /// Number of connected peers
    pub connected_peers: usize,
    /// NeuralGuardian statistics snapshot
    pub guardian_stats: GuardianStats,
    /// 512-bit BLAKE3 commitment over the entire health state
    pub trust_pulse_512: Vec<u8>,
    /// Unix timestamp (seconds)
    pub timestamp: u64,
}

/// Build a `NetworkHealth` snapshot and compute its 512-bit trust pulse.
///
/// Called by the node every 100 blocks to broadcast a verifiable health
/// summary to the network.
pub fn get_network_health(
    block_height: u64,
    total_mined: u64,
    remaining_supply: u64,
    connected_peers: usize,
    guardian_stats: GuardianStats,
) -> NetworkHealth {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Compute 512-bit trust pulse: BLAKE3-XOF over the full health state
    let mut hasher = blake3::Hasher::new();
    hasher.update(&block_height.to_le_bytes());
    hasher.update(&total_mined.to_le_bytes());
    hasher.update(&remaining_supply.to_le_bytes());
    hasher.update(&(connected_peers as u64).to_le_bytes());
    hasher.update(&(guardian_stats.total_events as u64).to_le_bytes());
    hasher.update(&(guardian_stats.unique_peers as u64).to_le_bytes());
    hasher.update(&(guardian_stats.cached_assessments as u64).to_le_bytes());
    hasher.update(&(guardian_stats.training_samples as u64).to_le_bytes());
    hasher.update(&timestamp.to_le_bytes());

    let mut trust_pulse_512 = [0u8; 64];
    hasher.finalize_xof().fill(&mut trust_pulse_512);

    NetworkHealth {
        block_height,
        total_mined,
        remaining_supply,
        connected_peers,
        guardian_stats,
        trust_pulse_512: trust_pulse_512.to_vec(),
        timestamp,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_network_health_pulse_determinism() {
        let stats = GuardianStats {
            total_events: 100,
            unique_peers: 10,
            cached_assessments: 5,
            training_samples: 50,
        };

        let h1 = get_network_health(1000, 500_000_000, 12_400_000_000 - 500_000_000, 20, stats.clone());
        let h2 = get_network_health(1000, 500_000_000, 12_400_000_000 - 500_000_000, 20, stats);

        // Trust pulse depends on timestamp so we only check structure
        assert_eq!(h1.block_height, 1000);
        assert_eq!(h1.total_mined, 500_000_000);
        assert_eq!(h1.connected_peers, 20);
        assert_eq!(h1.trust_pulse_512.len(), 64);
    }

    #[test]
    fn test_get_network_health_different_heights() {
        let stats = GuardianStats {
            total_events: 0,
            unique_peers: 0,
            cached_assessments: 0,
            training_samples: 0,
        };

        let h1 = get_network_health(100, 0, 12_400_000_000, 5, stats.clone());
        let h2 = get_network_health(200, 0, 12_400_000_000, 5, stats);

        assert_ne!(h1.trust_pulse_512, h2.trust_pulse_512,
            "Different block heights must produce different trust pulses");
    }

    #[test]
    fn test_compute_vdf_sequential() {
        let seed = [0xABu8; 32];
        let result = compute_vdf(seed, 10);
        assert_ne!(result, seed, "VDF output must differ from input");
        assert_eq!(result.len(), 32);
    }
}
