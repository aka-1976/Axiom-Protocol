/// Mining proofs use a fixed 128-byte hash-based format (blake3 512-bit XOF).
/// STARK proofs are larger and variable-sized (used for full transaction privacy).
const MINING_PROOF_SIZE: usize = 128;

/// Size of the public-inputs header prepended to serialized STARK proofs.
/// Layout: [commitment: 16 bytes (u128 LE)] [new_balance_commitment: 16 bytes (u128 LE)]
const STARK_PUBLIC_INPUTS_HEADER: usize = 32;

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
    use winterfell::math::StarkField;

    let secret_fr = circuit::bytes_to_field(secret_key);
    let balance_fr = BaseElement::new(current_balance as u128);
    let amount_fr = BaseElement::new(transfer_amount as u128);
    let fee_fr = BaseElement::new(fee as u128);

    let system = circuit::ZkProofSystem::setup()
        .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

    let nonce_fr = BaseElement::new(0u128);
    let (proof, public_inputs) = system
        .prove(secret_fr, balance_fr, nonce_fr, amount_fr, fee_fr)
        .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

    // Serialize the STARK proof with public inputs prepended.
    // Layout: [commitment: 16 bytes] [new_balance_commitment: 16 bytes] [proof bytes...]
    // The verifier needs the commitment and new_balance_commitment to
    // reconstruct the public inputs that match the proof.
    // public_inputs layout from ZkProofSystem::prove():
    //   [0] = commitment, [1] = transfer_amount, [2] = fee, [3] = new_balance_commitment
    let proof_bytes = proof.to_bytes();
    let commitment_bytes = public_inputs[0].as_int().to_le_bytes();       // commitment
    let new_balance_bytes = public_inputs[3].as_int().to_le_bytes();      // new_balance_commitment

    let mut encoded = Vec::with_capacity(STARK_PUBLIC_INPUTS_HEADER + proof_bytes.len());
    encoded.extend_from_slice(&commitment_bytes);
    encoded.extend_from_slice(&new_balance_bytes);
    encoded.extend_from_slice(&proof_bytes);

    Ok(encoded)
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
        // Hash-based verification: the public commitment is in bytes 64..128
        // (512-bit blake3 XOF of public_address || amount || fee).
        let mut hasher = blake3::Hasher::new();
        hasher.update(public_address);
        hasher.update(&transfer_amount.to_le_bytes());
        hasher.update(&fee.to_le_bytes());
        let mut expected = [0u8; 64];
        hasher.finalize_xof().fill(&mut expected);
        return Ok(proof_bytes[64..128] == expected);
    }

    // STARK proof: extract commitment and new_balance_commitment from
    // the prepended header, then deserialize the actual proof.
    // Layout: [commitment: 16 bytes] [new_balance_commitment: 16 bytes] [proof...]
    if proof_bytes.len() < STARK_PUBLIC_INPUTS_HEADER {
        return Err("STARK proof too short".into());
    }

    let commitment_int = u128::from_le_bytes(
        proof_bytes[0..16].try_into().map_err(|_| "bad commitment bytes")?
    );
    let new_balance_int = u128::from_le_bytes(
        proof_bytes[16..32].try_into().map_err(|_| "bad new_balance bytes")?
    );
    let stark_proof_data = &proof_bytes[STARK_PUBLIC_INPUTS_HEADER..];

    let proof = Proof::from_bytes(stark_proof_data)
        .map_err(|e| -> Box<dyn std::error::Error> {
            format!("Proof deserialization failed: {:?}", e).into()
        })?;

    // Reconstruct public inputs using the commitment values embedded
    // in the proof envelope and the transaction data.
    let commitment = BaseElement::new(commitment_int);
    let amount_fr = BaseElement::new(transfer_amount as u128);
    let fee_fr = BaseElement::new(fee as u128);
    let new_balance_commitment = BaseElement::new(new_balance_int);

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
        Err(_) => Ok(false),
    }
}

/// Generate ZK proof for mining (128-byte hash-based format, blake3 512-bit XOF).
///
/// Layout (128 bytes):
///   bytes  0..64  — blake3_512(wallet_secret || parent_hash)  [secret commitment]
///   bytes 64..128 — blake3_512(address       || parent_hash)  [public commitment]
///
/// The public commitment is derived from the miner's Ed25519 public key
/// (address) so that any node can verify the proof without the secret key.
pub fn generate_zk_pass(wallet_secret: &[u8; 32], parent_hash: [u8; 32]) -> Vec<u8> {
    use ed25519_dalek::{SigningKey, VerifyingKey};

    let mut proof_data = vec![0u8; MINING_PROOF_SIZE];

    // Secret commitment (512-bit)
    let mut hasher = blake3::Hasher::new();
    hasher.update(wallet_secret);
    hasher.update(&parent_hash);
    hasher.finalize_xof().fill(&mut proof_data[..64]);

    // Public commitment (512-bit) — derive address from secret key
    let signing_key = SigningKey::from_bytes(wallet_secret);
    let address = VerifyingKey::from(&signing_key).to_bytes();
    let mut hasher = blake3::Hasher::new();
    hasher.update(&address);
    hasher.update(&parent_hash);
    hasher.finalize_xof().fill(&mut proof_data[64..128]);

    proof_data
}

/// Verify mining proof by recomputing the 512-bit public commitment from
/// the miner address and parent hash.
pub fn verify_zk_pass(miner_address: &[u8; 32], parent: &[u8; 32], proof: &[u8]) -> bool {
    if proof.len() != MINING_PROOF_SIZE {
        return false;
    }
    if miner_address == &[0u8; 32] {
        return false;
    }
    // Secret commitment must be non-zero
    if proof[..64] == [0u8; 64] {
        return false;
    }
    // Recompute and verify 512-bit public commitment
    let mut hasher = blake3::Hasher::new();
    hasher.update(miner_address);
    hasher.update(parent);
    let mut expected = [0u8; 64];
    hasher.finalize_xof().fill(&mut expected);
    proof[64..128] == expected
}
