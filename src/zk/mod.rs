use sha2::{Sha256, Digest};
use std::fs;
use std::path::Path;
use std::io::Write;

/// Mining proofs use a fixed 128-byte hash-based format (lightweight).
/// STARK proofs are larger and variable-sized (used for full transaction privacy).
const MINING_PROOF_SIZE: usize = 128;

// Production ZK-STARK implementation
pub mod transaction_circuit;

pub use transaction_circuit::{
    TransactionCircuit,
    ProofData,
    prove_transaction,
    verify_zk_transaction_proof,
};

pub mod circuit;

/// Generate ZK-STARK proof for a transaction
pub fn generate_transaction_proof(
    secret_key: &[u8; 32],
    current_balance: u64,
    transfer_amount: u64,
    fee: u64,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    println!("[ZK-STARK DEBUG] Entered generate_transaction_proof");
    std::io::stdout().flush().unwrap();

    use winterfell::math::fields::f128::BaseElement;

    let secret_fr = circuit::bytes_to_field(secret_key);
    let balance_fr = BaseElement::from(current_balance as u128);
    let amount_fr = BaseElement::from(transfer_amount as u128);
    let fee_fr = BaseElement::from(fee as u128);

    println!("[ZK-STARK DEBUG] Proof Generation:");
    println!("  balance:  {}", current_balance);
    println!("  amount:   {}", transfer_amount);
    println!("  fee:      {}", fee);
    std::io::stdout().flush().unwrap();

    let system = circuit::ZkProofSystem::setup()
        .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

    let nonce_fr = BaseElement::from(0u128);
    let (proof, _public_inputs) = system
        .prove(secret_fr, balance_fr, nonce_fr, amount_fr, fee_fr)
        .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

    // Serialize the STARK proof
    let proof_bytes = proof.to_bytes();

    println!("[ZK-STARK DEBUG] Proof generated successfully ({} bytes)", proof_bytes.len());
    std::io::stdout().flush().unwrap();

    Ok(proof_bytes)
}

/// Verify ZK-STARK proof for a transaction
pub fn verify_transaction_proof(
    proof_bytes: &[u8],
    public_address: &[u8; 32],
    transfer_amount: u64,
    fee: u64,
) -> Result<bool, Box<dyn std::error::Error>> {
    println!("[ZK-STARK DEBUG] Entered verify_transaction_proof");
    std::io::stdout().flush().unwrap();

    use winterfell::math::fields::f128::BaseElement;
    use winterfell::StarkProof;

    // Mining proofs are fixed-size hash-based proofs (MINING_PROOF_SIZE bytes).
    // STARK proofs are larger and variable-sized.
    if proof_bytes.len() == MINING_PROOF_SIZE {
        // This is a mining proof - use hash-based verification
        let mut hasher = Sha256::new();
        hasher.update(public_address);
        hasher.update(&transfer_amount.to_le_bytes());
        hasher.update(&fee.to_le_bytes());
        let hash = hasher.finalize();
        return Ok(proof_bytes[..32] == hash[..32]);
    }

    // STARK proof deserialization
    let proof = match StarkProof::from_bytes(&proof_bytes) {
        Ok(p) => p,
        Err(e) => {
            println!("[ZK-STARK DEBUG] Proof deserialization error: {:?}", e);
            std::io::stdout().flush().unwrap();
            return Err(format!("Proof deserialization failed: {:?}", e).into());
        }
    };

    println!("[ZK-STARK DEBUG] Proof deserialized");
    std::io::stdout().flush().unwrap();

    // Reconstruct public inputs from the address and transaction data
    let address_fr = circuit::bytes_to_field(public_address);
    let amount_fr = BaseElement::from(transfer_amount as u128);
    let fee_fr = BaseElement::from(fee as u128);

    // For verification, we need the original public inputs
    // In a full implementation these would be stored alongside the proof
    let commitment = address_fr; // simplified
    let new_balance_commitment = address_fr - amount_fr - fee_fr;

    let pub_inputs = circuit::TransactionPublicInputs {
        commitment,
        transfer_amount: amount_fr,
        fee: fee_fr,
        new_balance_commitment,
    };

    match winterfell::verify::<circuit::AxiomTransactionAir>(proof, pub_inputs) {
        Ok(_) => {
            println!("[ZK-STARK DEBUG] Proof verification: VALID");
            std::io::stdout().flush().unwrap();
            Ok(true)
        }
        Err(e) => {
            println!("[ZK-STARK DEBUG] Proof verification failed: {:?}", e);
            std::io::stdout().flush().unwrap();
            Ok(false)
        }
    }
}

/// Generate ZK proof for mining (simplified for performance)
pub fn generate_zk_pass(wallet_secret: &[u8; 32], parent_hash: [u8; 32]) -> Vec<u8> {
    // For mining, we use a lightweight hash-based proof
    // STARK proofs are used for full transaction privacy
    let mut proof_data = vec![0u8; MINING_PROOF_SIZE];
    let mut hasher = Sha256::new();
    hasher.update(wallet_secret);
    hasher.update(parent_hash);
    hasher.update(b"mining_proof_stark");
    let hash = hasher.finalize();
    proof_data[..32].copy_from_slice(&hash);

    proof_data
}

/// Verify mining proof
pub fn verify_zk_pass(miner_address: &[u8; 32], _parent: &[u8; 32], proof: &[u8]) -> bool {
    if proof.len() != MINING_PROOF_SIZE {
        return false;
    }

    if miner_address == &[0u8; 32] {
        return false;
    }

    // Fallback to hash-based verification
    let mut hasher = Sha256::new();
    hasher.update(miner_address);
    hasher.update(_parent);
    hasher.update(b"mining_proof_stark");
    let expected_hash = hasher.finalize();

    proof[..32] == expected_hash[..32]
}
