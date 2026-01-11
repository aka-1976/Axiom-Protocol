use sha2::{Sha256, Digest};

/// EVALUATE: Creates the seed for the VDF chain.
/// This links the current block to the parent and the specific time-slot.
pub fn evaluate(parent_hash: [u8; 32], slot: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(parent_hash);
    hasher.update(slot.to_le_bytes());
    hasher.finalize().into()
}

/// VERIFY: Recomputes the sequential chain to ensure the time-lock was respected.
/// This is the "Self-Healing" heart: any node can verify that time has passed
/// without trusting the miner.
#[allow(dead_code)]
pub fn verify_vdf(seed: [u8; 32], iterations: u32, proof: [u8; 32]) -> bool {
    // The main_helper contains the actual sequential hashing loop
    let expected = crate::main_helper::compute_vdf(seed, iterations);
    expected == proof
}
