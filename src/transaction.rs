use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

/// Type alias for 32-byte public addresses
pub type Address = [u8; 32];

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub from: Address,
    pub to: Address,
    pub amount: u64,
    pub fee: u64,
    pub nonce: u64,
    pub zk_proof: Vec<u8>,
}

#[allow(dead_code)]
impl Transaction {
    /// Generates a unique transaction identifier (TXID).
    pub fn hash(&self) -> [u8; 32] {
        let serialized = bincode::serialize(self).expect("CRITICAL: Transaction serialization failed");
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        let result = hasher.finalize();
        
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    /// Helper to create a new transaction.
    pub fn new(
        from: Address, 
        to: Address, 
        amount: u64, 
        fee: u64, 
        nonce: u64, 
        zk_proof: Vec<u8>
    ) -> Self {
        Self {
            from,
            to,
            amount,
            fee,
            nonce,
            zk_proof,
        }
    }
}
