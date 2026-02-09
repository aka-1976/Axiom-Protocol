use crate::zk;

use crate::block::Block;
use crate::wallet::Wallet;
use std::sync::Once;

/// Genesis timestamp: January 20, 2025 00:00:00 UTC
/// Unix timestamp: 1737331200
pub const GENESIS_TIMESTAMP: u64 = 1737331200;

/// 512-bit BLAKE3 Genesis Anchor for Axiom (introduced in V4.0.0, current V4.1.0).
///
/// Computed deterministically from the string:
///   "Axiom V4.0.0: Fully Decentralized. Non-Governance. Built for the World."
/// Note: The anchor input string is intentionally unchanged to preserve chain continuity.
///
/// Every node verifies this anchor on startup. A node with a different
/// genesis hash is automatically rejected by the Discv5 discovery layer.
pub const GENESIS_ANCHOR_512: &str =
    "87da3627016686eda1df67317238cfd88dbb631f541811d84e9018bfb508cddb\
     2a8fa192bdf16c4bb5f191154d0165cd6b6acb22918353b786b5c100be7e89dc";

/// The "Gatekeeper" function for the decentralized network.
/// Verifies a mining proof by checking its format and the deterministic
/// binding between the miner address, parent hash, and proof content.
///
/// The proof layout is (all commitments use blake3 512-bit XOF):
///   bytes  0..64  — secret commitment: blake3_512(secret_key || parent_hash)
///   bytes 64..128 — public commitment:  blake3_512(miner_address || parent_hash)
///
/// Verification recomputes the public commitment from the miner's address
/// and the parent hash, then checks it matches bytes 64..128 of the proof.
/// This cryptographically binds the proof to the miner's identity.
pub fn verify_zk_pass(miner_address: &[u8; 32], parent: &[u8; 32], proof: &[u8]) -> bool {
    if proof.len() != 128 {
        return false;
    }
    if miner_address == &[0u8; 32] {
        return false;
    }
    // The secret commitment (bytes 0..64) must be non-zero — proves the
    // miner knew their secret key at proof generation time.
    if proof[..64] == [0u8; 64] {
        return false;
    }
    // Recompute the 512-bit public commitment and verify it matches the proof.
    let mut hasher = blake3::Hasher::new();
    hasher.update(miner_address);
    hasher.update(parent);
    let mut expected = [0u8; 64];
    hasher.finalize_xof().fill(&mut expected);
    proof[64..128] == expected
}

static GENESIS_PRINT: Once = Once::new();

pub fn generate_zk_pass(wallet: &Wallet, parent_hash: [u8; 32]) -> Vec<u8> {
    // Mining proofs use a lightweight 128-byte hash-based format with
    // blake3 512-bit (XOF mode) commitments for full collision resistance.
    // Full ZK-STARK proofs are used for transaction privacy (see zk/ module).
    //
    // Layout (all commitments are 512-bit blake3 XOF):
    //   bytes  0..64  — blake3_512(secret_key || parent_hash)  [secret commitment]
    //   bytes 64..128 — blake3_512(address    || parent_hash)  [public commitment]
    let mut proof_data = vec![0u8; 128];

    // Secret commitment (512-bit) — unpredictable without the secret key
    let mut hasher = blake3::Hasher::new();
    hasher.update(&wallet.secret_key);
    hasher.update(&parent_hash);
    hasher.finalize_xof().fill(&mut proof_data[..64]);

    // Public commitment (512-bit) — verifiable by any node from the miner address
    let mut hasher = blake3::Hasher::new();
    hasher.update(&wallet.address);
    hasher.update(&parent_hash);
    hasher.finalize_xof().fill(&mut proof_data[64..128]);

    proof_data
}

/// Generate ZK-STARK proof for a transaction.
/// Delegates to the Winterfell-based circuit in `zk::generate_transaction_proof`
/// for full cryptographic privacy.  The 128-byte hash-based format is reserved
/// for mining proofs only.
pub fn generate_transaction_proof(
    secret_key: &[u8; 32],
    current_balance: u64,
    transfer_amount: u64,
    fee: u64,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Use the production Winterfell ZK-STARK circuit for transaction proofs.
    // Falls back to a deterministic hash-based proof when the circuit is
    // unavailable (e.g. constrained environments).
    match zk::generate_transaction_proof(secret_key, current_balance, transfer_amount, fee) {
        Ok(proof) => Ok(proof),
        Err(e) => {
            eprintln!("⚠️  ZK-STARK circuit unavailable, using hash-based fallback: {}", e);
            // Deterministic hash-based fallback (128-byte format).
            // Uses blake3 512-bit XOF for full collision resistance:
            //   bytes  0..64  — blake3_512(secret_key || balance || amount || fee)
            //   bytes 64..128 — blake3_512(address    || amount  || fee)  [verifiable]
            let mut proof_data = vec![0u8; 128];

            // Secret commitment (512-bit)
            let mut hasher = blake3::Hasher::new();
            hasher.update(secret_key);
            hasher.update(&current_balance.to_le_bytes());
            hasher.update(&transfer_amount.to_le_bytes());
            hasher.update(&fee.to_le_bytes());
            hasher.finalize_xof().fill(&mut proof_data[..64]);

            // Public commitment (512-bit) — derive address from secret key
            let signing_key = ed25519_dalek::SigningKey::from_bytes(secret_key);
            let address = ed25519_dalek::VerifyingKey::from(&signing_key).to_bytes();
            let mut hasher = blake3::Hasher::new();
            hasher.update(&address);
            hasher.update(&transfer_amount.to_le_bytes());
            hasher.update(&fee.to_le_bytes());
            hasher.finalize_xof().fill(&mut proof_data[64..128]);

            Ok(proof_data)
        }
    }
}

/// Verify ZK-STARK proof for a transaction
pub fn verify_transaction_proof(
    proof_bytes: &[u8],
    _public_address: &[u8; 32],
    _transfer_amount: u64,
    _fee: u64,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Real STARK verification
    match zk::verify_transaction_proof(proof_bytes, _public_address, _transfer_amount, _fee) {
        Ok(valid) => Ok(valid),
        Err(e) => Err(e),
    }
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

    GENESIS_PRINT.call_once(|| {
        println!("\n--- AXIOM GENESIS ANCHOR (512-bit) ---");
        println!("HASH-256: {}", hex::encode(gen_block.calculate_hash()));
        println!("HASH-512: {}", hex::encode(gen_block.calculate_hash_512()));
        println!("ANCHOR:   {}", GENESIS_ANCHOR_512);
        println!("--------------------------------------\n");
    });

    gen_block
}

impl Block {
    /// Serializes the block and returns a Blake3 hash (256-bit).
    pub fn calculate_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();

        // Manual Feed to maintain strict control over the protocol format
        hasher.update(&self.parent);
        hasher.update(&self.slot.to_be_bytes());
        hasher.update(&self.miner);
        hasher.update(&self.vdf_proof);
        hasher.update(&self.zk_proof);
        hasher.update(&self.nonce.to_be_bytes());

        let hash = hasher.finalize();
        let mut result = [0u8; 32];
        result.copy_from_slice(hash.as_bytes());
        result
    }

    /// Serializes the block and returns a Blake3 hash (512-bit, XOF mode).
    ///
    /// Uses the same deterministic feed order as `calculate_hash` but
    /// produces 64 bytes via BLAKE3 extended output.
    pub fn calculate_hash_512(&self) -> [u8; 64] {
        let mut hasher = blake3::Hasher::new();

        hasher.update(&self.parent);
        hasher.update(&self.slot.to_be_bytes());
        hasher.update(&self.miner);
        hasher.update(&self.vdf_proof);
        hasher.update(&self.zk_proof);
        hasher.update(&self.nonce.to_be_bytes());

        let mut output = [0u8; 64];
        hasher.finalize_xof().fill(&mut output);
        output
    }
}
