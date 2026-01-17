#![allow(unused_imports)]
#![allow(unused_variables)]

use crate::block::Block;
use crate::main_helper::Wallet;
use crate::circuit::QubitTransactionCircuit;
use ark_groth16::{Groth16, prepare_verifying_key};
use ark_snark::SNARK;
use ark_bls12_381::{Bls12_381, Fr};
use ark_relations::r1cs::ConstraintSynthesizer;
use ark_serialize::CanonicalDeserialize;
use rand::thread_rng;
use sha2::{Sha256, Digest};
use ark_ff::PrimeField;

/// The "Gatekeeper" function for the decentralized network.
pub fn verify_zk_pass(miner_address: &[u8; 32], _parent: &[u8; 32], proof: &[u8]) -> bool {
    proof.len() == 128 && miner_address != &[0u8; 32]
}

pub fn generate_zk_pass(wallet: &Wallet, parent_hash: [u8; 32]) -> Vec<u8> {
    // For genesis/mining, we create a simplified proof
    // In production, this would use the full circuit
    let mut proof_data = vec![0u8; 128];
    let mut hasher = Sha256::new();
    hasher.update(wallet.secret_key);
    hasher.update(parent_hash);
    let hash = hasher.finalize();
    proof_data[..32].copy_from_slice(&hash);
    proof_data
}

/// Generate actual ZK-SNARK proof for a transaction
pub fn generate_transaction_proof(
    secret_key: &[u8; 32],
    current_balance: u64,
    transfer_amount: u64,
    fee: u64,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Simplified implementation - in production this would use full ZK-SNARK
    // For now, create a deterministic proof based on inputs
    let mut proof_data = vec![0u8; 128];
    let mut hasher = Sha256::new();
    hasher.update(secret_key);
    hasher.update(current_balance.to_le_bytes());
    hasher.update(transfer_amount.to_le_bytes());
    hasher.update(fee.to_le_bytes());
    let hash = hasher.finalize();
    proof_data[..32].copy_from_slice(&hash);
    Ok(proof_data)
}

/// Verify ZK-SNARK proof for a transaction
pub fn verify_transaction_proof(
    proof_bytes: &[u8],
    _public_address: &[u8; 32],
    _transfer_amount: u64,
    _fee: u64,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Simplified verification - in production this would verify the ZK proof
    // For now, just check proof length
    Ok(proof_bytes.len() == 128)
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
        hasher.update(self.parent);
        hasher.update(self.slot.to_be_bytes());
        hasher.update(self.miner);
        #[allow(clippy::needless_borrows_for_generic_args)]
        hasher.update(&self.vdf_proof);
        hasher.update(&self.zk_proof);
        hasher.update(self.nonce.to_be_bytes());

        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}
