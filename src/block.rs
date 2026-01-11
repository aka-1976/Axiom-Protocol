use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use crate::transaction::{Transaction, Address};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub parent: [u8; 32],
    pub slot: u64,
    pub miner: Address,
    pub transactions: Vec<Transaction>,
    pub vdf_proof: [u8; 32],
    pub zk_proof: Vec<u8>,
    pub nonce: u64, // The PoW layer for Hash Power
}

impl Block {
    /// Computes the cryptographic hash of the block
    pub fn hash(&self) -> [u8; 32] {
        let serialized = bincode::serialize(self).expect("Serialization failed");
        let mut hasher = Sha256::new();
        hasher.update(serialized);
        hasher.finalize().into()
    }

    /// Checks if the block meets the dynamic network difficulty (Hash Power check)
    pub fn meets_difficulty(&self, difficulty: u64) -> bool {
        let h = self.hash();
        // Convert first 8 bytes to u64 for numerical comparison
        let val = u64::from_be_bytes(h[0..8].try_into().unwrap());
        
        // Difficulty formula: higher difficulty results in a smaller target range
        val < (u64::MAX / difficulty.max(1))
    }

    pub fn new(
        parent: [u8; 32],
        slot: u64,
        miner: Address,
        transactions: Vec<Transaction>,
        vdf_proof: [u8; 32],
        zk_proof: Vec<u8>,
        nonce: u64,
    ) -> Self {
        Self {
            parent,
            slot,
            miner,
            transactions,
            vdf_proof,
            zk_proof,
            nonce,
        }
    }
}
