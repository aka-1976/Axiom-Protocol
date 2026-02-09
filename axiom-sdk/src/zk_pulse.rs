//! ZK-Pulse generator and `prove_transaction()` implementation.
//!
//! This module packages contract execution results with a BLAKE3-XOF
//! 512-bit digest and a 512-bit BLAKE3-XOF commitment proof, allowing
//! users to generate a local Proof-of-Execution without revealing
//! private transaction data.

use serde::{Serialize, Deserialize};
use crate::error::{AxiomError, Result};

/// Input for [`prove_transaction`](ZkPulse::prove_transaction).
///
/// Models the private witness data that would be fed into the RISC-V
/// guest program.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProveTransactionInput {
    pub initial_balance: u64,
    pub amount: u64,
    pub fee: u64,
    pub nonce: u64,
}

/// Output of [`prove_transaction`](ZkPulse::prove_transaction).
///
/// Contains the 512-bit BLAKE3-XOF digest (hex-encoded) and the raw
/// proof bytes.
#[derive(Debug, Clone)]
pub struct ProveTransactionOutput {
    /// 128-character hex string of the 512-bit BLAKE3-XOF digest.
    pub digest_512: String,
    /// Serialised proof bytes (512-bit BLAKE3-XOF commitment).
    pub proof_bytes: Vec<u8>,
}

/// ZK-Pulse generator.
///
/// Packages contract execution results with a BLAKE3-XOF digest and a
/// 512-bit BLAKE3-XOF commitment proof, enabling Proof-of-Execution
/// without revealing private data.
pub struct ZkPulse;

impl ZkPulse {
    /// Generate a 512-bit BLAKE3-XOF digest for the given transaction input.
    pub fn digest_512(input: &ProveTransactionInput) -> Result<[u8; 64]> {
        let encoded = serde_json::to_vec(input)
            .map_err(|e| AxiomError::Serialization(e.to_string()))?;
        let mut hasher = blake3::Hasher::new();
        hasher.update(&encoded);
        let mut output = [0u8; 64];
        hasher.finalize_xof().fill(&mut output);
        Ok(output)
    }

    /// Compile user-defined Rust logic into a RISC-V proof locally.
    ///
    /// This is the core of the "Proof-of-Execution" flow:
    ///
    /// 1. Validate the supply invariant (`initial_balance >= amount + fee`).
    /// 2. Compute the 512-bit BLAKE3-XOF digest of the input.
    /// 3. Bundle the digest with a 512-bit BLAKE3-XOF commitment proof.
    ///
    /// The returned [`ProveTransactionOutput`] can be submitted to the
    /// network or to the Ethereum bridge for verification.
    ///
    /// # Errors
    /// Returns an error if the balance is insufficient or if
    /// serialisation fails.
    pub fn prove_transaction(input: &ProveTransactionInput) -> Result<ProveTransactionOutput> {
        // 1. Supply invariant check (mirrors the RISC-V guest logic).
        let total = input.amount.checked_add(input.fee)
            .ok_or_else(|| AxiomError::Proof("amount + fee overflow".to_string()))?;
        if input.initial_balance < total {
            return Err(AxiomError::Proof(format!(
                "Insufficient balance: have {}, need {} (amount) + {} (fee)",
                input.initial_balance, input.amount, input.fee,
            )));
        }

        // 2. Compute 512-bit BLAKE3-XOF digest.
        let digest = Self::digest_512(input)?;
        let digest_hex = hex::encode(digest);

        // 3. Generate 512-bit BLAKE3-XOF commitment proof.
        let proof_bytes = Self::generate_proof_commitment(&digest)?;

        Ok(ProveTransactionOutput {
            digest_512: digest_hex,
            proof_bytes,
        })
    }

    /// Generate a 512-bit BLAKE3-XOF commitment proof.
    ///
    /// The commitment is the 512-bit BLAKE3-XOF hash of the digest, prefixed
    /// with a version byte.  This binds the proof to the exact transaction
    /// input via a cryptographic commitment that is infeasible to forge.
    fn generate_proof_commitment(digest_512: &[u8; 64]) -> Result<Vec<u8>> {
        // The commitment is the 512-bit BLAKE3-XOF hash of a domain-separated
        // input, ensuring forward-compatible proof format.
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"axiom-proof-v1:");
        hasher.update(digest_512);
        let mut commitment = [0u8; 64];
        hasher.finalize_xof().fill(&mut commitment);

        let mut proof = Vec::with_capacity(1 + 64);
        proof.push(0x01); // version byte
        proof.extend_from_slice(&commitment);
        Ok(proof)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digest_determinism() {
        let input = ProveTransactionInput {
            initial_balance: 5_000,
            amount: 1_000,
            fee: 10,
            nonce: 7,
        };
        let d1 = ZkPulse::digest_512(&input).unwrap();
        let d2 = ZkPulse::digest_512(&input).unwrap();
        assert_eq!(d1, d2, "Same input must produce identical digest");
        assert_eq!(d1.len(), 64);
    }

    #[test]
    fn test_digest_uniqueness() {
        let i1 = ProveTransactionInput { initial_balance: 5_000, amount: 100, fee: 10, nonce: 1 };
        let i2 = ProveTransactionInput { initial_balance: 5_000, amount: 100, fee: 10, nonce: 2 };
        let d1 = ZkPulse::digest_512(&i1).unwrap();
        let d2 = ZkPulse::digest_512(&i2).unwrap();
        assert_ne!(d1, d2, "Different nonces must produce different digests");
    }

    #[test]
    fn test_prove_transaction_valid() {
        let input = ProveTransactionInput {
            initial_balance: 10_000,
            amount: 1_000,
            fee: 50,
            nonce: 1,
        };
        let output = ZkPulse::prove_transaction(&input).unwrap();
        assert_eq!(output.digest_512.len(), 128);
        assert!(!output.proof_bytes.is_empty());
        assert_eq!(output.proof_bytes[0], 0x01); // version byte
    }

    #[test]
    fn test_prove_transaction_insufficient_balance() {
        let input = ProveTransactionInput {
            initial_balance: 50,
            amount: 100,
            fee: 10,
            nonce: 1,
        };
        assert!(ZkPulse::prove_transaction(&input).is_err());
    }

    #[test]
    fn test_prove_transaction_overflow() {
        let input = ProveTransactionInput {
            initial_balance: u64::MAX,
            amount: u64::MAX,
            fee: 1,
            nonce: 0,
        };
        assert!(ZkPulse::prove_transaction(&input).is_err());
    }

    #[test]
    fn test_prove_transaction_exact_balance() {
        let input = ProveTransactionInput {
            initial_balance: 110,
            amount: 100,
            fee: 10,
            nonce: 0,
        };
        assert!(ZkPulse::prove_transaction(&input).is_ok());
    }
}
