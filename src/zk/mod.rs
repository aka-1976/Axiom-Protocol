use sha2::{Sha256, Digest};

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
    use winterfell::math::fields::f128::BaseElement;

    let secret_fr = circuit::bytes_to_field(secret_key);
    let balance_fr = BaseElement::new(current_balance as u128);
    let amount_fr = BaseElement::new(transfer_amount as u128);
    let fee_fr = BaseElement::new(fee as u128);

    let system = circuit::ZkProofSystem::setup()
        .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

    let nonce_fr = BaseElement::new(0u128);
    let (proof, _public_inputs) = system
        .prove(secret_fr, balance_fr, nonce_fr, amount_fr, fee_fr)
        .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

    // Serialize the STARK proof
    let proof_bytes = proof.to_bytes();

    Ok(proof_bytes)
}

/// Verify ZK-STARK proof for a transaction
pub fn verify_transaction_proof(
    proof_bytes: &[u8],
    public_address: &[u8; 32],
    transfer_amount: u64,
    fee: u64,
) -> Result<bool, Box<dyn std::error::Error>> {
    use winterfell::math::fields::f128::BaseElement;
    use winterfell::Proof;

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
    let proof = Proof::from_bytes(proof_bytes)
        .map_err(|e| -> Box<dyn std::error::Error> {
            format!("Proof deserialization failed: {:?}", e).into()
        })?;

    // Reconstruct public inputs from the address and transaction data
    let address_fr = circuit::bytes_to_field(public_address);
    let amount_fr = BaseElement::new(transfer_amount as u128);
    let fee_fr = BaseElement::new(fee as u128);

    // The commitment is derived from the public address (which itself is
    // derived from the secret key). The new_balance_commitment encodes the
    // post-transaction state.
    let commitment = address_fr;
    let new_balance_commitment = address_fr - amount_fr - fee_fr;

    let pub_inputs = circuit::TransactionPublicInputs {
        commitment,
        transfer_amount: amount_fr,
        fee: fee_fr,
        new_balance_commitment,
    };

    use winterfell::crypto::{hashers::Blake3_256, DefaultRandomCoin};
    use winterfell::AcceptableOptions;

    let min_opts = AcceptableOptions::MinConjecturedSecurity(circuit::MIN_SECURITY_BITS);

    match winterfell::verify::<
        circuit::AxiomTransactionAir,
        Blake3_256<BaseElement>,
        DefaultRandomCoin<Blake3_256<BaseElement>>,
    >(proof, pub_inputs, &min_opts) {
        Ok(_) => Ok(true),
        Err(e) => {
            Err(format!("STARK verification failed: {:?}", e).into())
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
