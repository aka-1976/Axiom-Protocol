use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use rand::RngCore;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct BridgeLock {
    /// The hash of the secret (H) - Published on the network
    pub hash_lock: [u8; 32],
    /// The block height (slot) after which the funds are refunded
    pub timeout_slot: u64,
    /// The amount of AXM to be swapped
    pub amount: u64,
    /// The destination address on the external chain
    pub recipient_on_other_chain: String,
}

pub struct BridgeSecret {
    /// The preimage (s) that unlocks the hash_lock
    pub secret: [u8; 32],
}

impl BridgeSecret {
    /// Generates a new secret for an atomic swap (Hardened)
    pub fn generate() -> Self {
        let mut secret = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut secret);
        Self { secret }
    }

    /// Derives the hash lock (H) from the secret (s)
    /// Hardened: Explicitly copies output into a [u8; 32] array
    pub fn to_hash_lock(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.secret);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}
