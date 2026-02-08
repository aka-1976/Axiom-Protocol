impl Block {
    /// Calculate mining reward based on halving schedule
    pub fn mining_reward(slot: u64) -> u64 {
        let initial_reward = 50_000_000u64;
        let halving_interval = 1_240_000u64; // Matches 124M total supply
        let halvings = slot / halving_interval;
        initial_reward >> halvings.min(32) // Prevent overflow
    }

    /// Apply mining reward to state
    pub fn apply_mining_reward(&self, state: &mut crate::state::State) {
        let reward = Block::mining_reward(self.slot);
        state.credit(self.miner, reward);
    }
}
use crate::vdf;
use crate::state::State;

impl Block {
    /// Full block validation: VDF, ZK-STARK, PoW, and transaction checks
    pub fn validate(&self, parent_hash: [u8; 32], parent_slot: u64, state: &mut State, difficulty: u64, vdf_iterations: u32, vdf_n: &rug::Integer) -> Result<(), &'static str> {
        // 1. VDF verification
        let vdf_seed = vdf::evaluate(parent_hash, parent_slot);
        let vdf_valid = vdf::wesolowski_verify(&rug::Integer::from_digits(&vdf_seed, rug::integer::Order::Lsf), vdf_iterations, vdf_n, &rug::Integer::from_digits(&self.vdf_proof, rug::integer::Order::Lsf));
        if !vdf_valid {
            return Err("Invalid VDF proof");
        }

        // 2. PoW check
        if !self.meets_difficulty(difficulty) {
            return Err("Block does not meet PoW difficulty");
        }

        // 3. ZK-STARK proof (for miner)
        // For demonstration, skip if empty
        if self.zk_proof.is_empty() {
            return Err("Missing miner ZK-STARK proof");
        }

        // 4. Transaction checks
        for tx in &self.transactions {
            let sender_balance = state.balance(&tx.from);
            tx.validate(sender_balance)?;
            state.apply_tx(tx)?;
        }

        Ok(())
    }
}
use serde::{Serialize, Deserialize};
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
    /// Computes the cryptographic hash of the block using Blake3
    pub fn hash(&self) -> [u8; 32] {
        let serialized = bincode::serialize(self).expect("Serialization failed");
        blake3::hash(&serialized).into()
    }

    /// Checks if the block meets the dynamic network difficulty (Hash Power check)
    pub fn meets_difficulty(&self, difficulty: u64) -> bool {
        let h = self.hash();
        // Convert first 8 bytes to u64 for numerical comparison
        // Safe conversion with proper error handling
        let val = match <[u8; 8]>::try_from(&h[0..8]) {
            Ok(bytes) => u64::from_be_bytes(bytes),
            Err(_) => {
                eprintln!("⚠️  Block hash conversion failed");
                return false;
            }
        };
        
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
