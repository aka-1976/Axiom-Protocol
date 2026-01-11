use crate::transaction::Address;
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
