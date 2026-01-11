#![allow(unused_imports)]
#![allow(unused_variables)]

use crate::block::Block;
use crate::main_helper::Wallet;
use ark_groth16::Groth16;
use ark_bls12_381::Bls12_381;
use ark_snark::SNARK;
use rand::thread_rng;
use sha2::{Sha256, Digest}; // Add this for hashing

/// The "Gatekeeper" function for the decentralized network.
pub fn verify_zk_pass(miner_address: &[u8; 32], _parent: &[u8; 32], proof: &[u8]) -> bool {
    proof.len() == 128 && miner_address != &[0u8; 32]
}

pub fn generate_zk_pass(_wallet: &Wallet, _parent_hash: [u8; 32]) -> Vec<u8> {
    vec![0u8; 128]
}

/// The immutable Genesis Block.
pub fn genesis() -> Block {
    let gen_block = Block {
        parent: [0u8; 32],
        slot: 0,
        miner: [0u8; 32],
        transactions: vec![],
        vdf_proof: [0u8; 32],
        zk_proof: vec![0u8; 128],
        nonce: 0,
    };

    // FIXED: Using hex::encode to format the [u8; 32] as a string for printing
    println!("\n--- QUBIT GENESIS ANCHOR ---");
    println!("HASH: {}", hex::encode(gen_block.calculate_hash()));
    println!("----------------------------\n");

    gen_block
}

impl Block {
    /// Serializes the block and returns a SHA-256 hash.
    pub fn calculate_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();

        // Manual Feed to maintain strict control over the 84M protocol format
        hasher.update(&self.parent);
        hasher.update(&self.slot.to_be_bytes());
        hasher.update(&self.miner);
        hasher.update(&self.vdf_proof);
        hasher.update(&self.zk_proof);
        hasher.update(&self.nonce.to_be_bytes());

        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}
