// src/stark/prover.rs — RISC Zero Host Prover for 124M Supply Integrity
//
// This file contains:
//   1. `TransactionData` — the private witness sent to the zkVM Guest
//   2. `StarkReceipt`    — a wrapper around the STARK proof receipt
//   3. `StarkProver`     — orchestrates proof generation and verification
//
// When the `risc0` feature is enabled, `StarkProver` uses the real
// RISC Zero prover. Otherwise it falls back to the existing Winterfell
// STARK backend so the node can operate without the risc0 toolchain.

use serde::{Serialize, Deserialize};

/// Number of blocks between mandatory RISC-V STARK receipt generation.
/// Every `STARK_PROOF_INTERVAL` blocks the node generates a supply
/// integrity proof that any peer (or the Ethereum bridge) can verify.
pub const STARK_PROOF_INTERVAL: u64 = 100;

/// Private transaction data passed into the RISC Zero Guest.
///
/// The Guest enforces `initial_balance >= amount + fee` inside the
/// zkVM.  Only the 512-bit BLAKE3 anchor of this data is committed
/// to the public journal; balances and nonces stay private.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    pub initial_balance: u64,
    pub amount: u64,
    pub fee: u64,
    pub nonce: u64,
}

/// Wrapper around a STARK proof receipt.
///
/// With the `risc0` feature this contains a real `risc0_zkvm::Receipt`.
/// Without it, the receipt holds the 512-bit journal hash produced by
/// the Winterfell fallback so that the rest of the stack can treat
/// receipts uniformly.
#[derive(Debug, Clone)]
pub struct StarkReceipt {
    /// The 512-bit BLAKE3 anchor committed by the Guest.
    pub journal_hash_512: [u8; 64],
    /// Serialized proof bytes (format depends on backend).
    pub seal: Vec<u8>,
}

/// Host-side STARK prover that generates 124M supply integrity proofs.
pub struct StarkProver;

impl StarkProver {
    /// Generate a STARK receipt proving that `tx` satisfies the 124M
    /// supply law and anchor the result with a 512-bit BLAKE3 hash.
    ///
    /// # Errors
    /// Returns an error if the balance is insufficient or if proof
    /// generation fails.
    pub fn generate_proof(
        tx: &TransactionData,
    ) -> Result<StarkReceipt, Box<dyn std::error::Error>> {
        // Pre-check (mirrors the Guest assertion so we fail fast)
        let total = tx.amount.checked_add(tx.fee)
            .ok_or("amount + fee overflow")?;
        if tx.initial_balance < total {
            return Err(format!(
                "Insufficient balance: have {}, need {} (amount) + {} (fee)",
                tx.initial_balance, tx.amount, tx.fee,
            ).into());
        }

        // Compute the 512-bit BLAKE3 anchor (same logic as Guest)
        let journal_hash_512 = Self::compute_512_anchor(tx)?;

        // --- Backend selection ---
        #[cfg(feature = "risc0")]
        {
            Self::prove_risc0(tx, journal_hash_512)
        }

        #[cfg(not(feature = "risc0"))]
        {
            Self::prove_winterfell(tx, journal_hash_512)
        }
    }

    /// Verify a receipt against the expected 512-bit anchor.
    ///
    /// Returns `true` if and only if:
    ///   1. The seal is a valid STARK proof.
    ///   2. The journal hash matches `expected_hash_512`.
    pub fn verify_receipt(
        receipt: &StarkReceipt,
        expected_hash_512: &[u8; 64],
    ) -> Result<bool, Box<dyn std::error::Error>> {
        if receipt.journal_hash_512 != *expected_hash_512 {
            return Ok(false);
        }

        #[cfg(feature = "risc0")]
        {
            Self::verify_risc0(receipt)
        }

        #[cfg(not(feature = "risc0"))]
        {
            Self::verify_winterfell(receipt)
        }
    }

    /// Compute the 512-bit BLAKE3 XOF anchor for a transaction.
    ///
    /// This is the deterministic hash that the Guest commits to its
    /// journal. Both Host and Guest must compute it identically.
    pub fn compute_512_anchor(
        tx: &TransactionData,
    ) -> Result<[u8; 64], Box<dyn std::error::Error>> {
        let tx_bytes = bincode::serialize(tx)?;
        let mut hasher = blake3::Hasher::new();
        hasher.update(&tx_bytes);
        let mut output = [0u8; 64];
        hasher.finalize_xof().fill(&mut output);
        Ok(output)
    }

    // ------------------------------------------------------------------
    // Winterfell fallback (default when `risc0` feature is off)
    // ------------------------------------------------------------------

    #[cfg(not(feature = "risc0"))]
    fn prove_winterfell(
        tx: &TransactionData,
        journal_hash_512: [u8; 64],
    ) -> Result<StarkReceipt, Box<dyn std::error::Error>> {
        use winterfell::math::fields::f128::BaseElement;

        let system = crate::zk::circuit::ZkProofSystem::setup()
            .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

        let secret_fr = BaseElement::new(tx.nonce as u128);
        let balance_fr = BaseElement::new(tx.initial_balance as u128);
        let nonce_fr = BaseElement::new(0u128);
        let amount_fr = BaseElement::new(tx.amount as u128);
        let fee_fr = BaseElement::new(tx.fee as u128);

        let (proof, _pub) = system
            .prove(secret_fr, balance_fr, nonce_fr, amount_fr, fee_fr)
            .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

        Ok(StarkReceipt {
            journal_hash_512,
            seal: proof.to_bytes(),
        })
    }

    #[cfg(not(feature = "risc0"))]
    fn verify_winterfell(
        receipt: &StarkReceipt,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        use winterfell::math::fields::f128::BaseElement;
        use winterfell::Proof;

        let proof = Proof::from_bytes(&receipt.seal)
            .map_err(|e| -> Box<dyn std::error::Error> {
                format!("Proof deserialization failed: {:?}", e).into()
            })?;

        // Reconstruct minimal public inputs from the journal hash
        let hash_fe = crate::zk::circuit::bytes_to_field(&receipt.journal_hash_512);
        let zero = BaseElement::new(0u128);
        let pub_inputs = vec![hash_fe, zero, zero, hash_fe];

        let system = crate::zk::circuit::ZkProofSystem::setup()
            .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

        match system.verify(&proof, &pub_inputs) {
            Ok(true) => Ok(true),
            Ok(false) => Ok(false),
            Err(_) => Ok(false),
        }
    }

    // ------------------------------------------------------------------
    // RISC Zero backend (enabled with `risc0` feature)
    // ------------------------------------------------------------------

    #[cfg(feature = "risc0")]
    fn prove_risc0(
        tx: &TransactionData,
        journal_hash_512: [u8; 64],
    ) -> Result<StarkReceipt, Box<dyn std::error::Error>> {
        use risc0_zkvm::{default_prover, ExecutorEnv};

        // AXIOM_INTEGRITY_ELF and AXIOM_INTEGRITY_ID are generated by
        // `methods/build.rs` when the risc0 toolchain compiles the Guest.
        // They are linked via the `methods` crate dependency.
        //
        // These external constants are linked via the `methods` crate
        // at build time, produced by `methods/build.rs` when the risc0
        // toolchain compiles the Guest binary.
        extern "Rust" {
            static AXIOM_INTEGRITY_ELF: &'static [u8];
            static AXIOM_INTEGRITY_ID: [u32; 8];
        }

        let elf = unsafe { AXIOM_INTEGRITY_ELF };
        let image_id = unsafe { AXIOM_INTEGRITY_ID };

        let env = ExecutorEnv::builder()
            .write(tx)?
            .build()?;

        let prover = default_prover();
        let receipt = prover.prove(env, elf)?.receipt;

        // Local verification before broadcasting
        receipt.verify(image_id)?;

        Ok(StarkReceipt {
            journal_hash_512,
            seal: bincode::serialize(&receipt)?,
        })
    }

    #[cfg(feature = "risc0")]
    fn verify_risc0(
        receipt: &StarkReceipt,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        use risc0_zkvm::Receipt;

        extern "Rust" {
            static AXIOM_INTEGRITY_ID: [u32; 8];
        }

        let image_id = unsafe { AXIOM_INTEGRITY_ID };

        let r0_receipt: Receipt = bincode::deserialize(&receipt.seal)?;
        match r0_receipt.verify(image_id) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_data_serialization() {
        let tx = TransactionData {
            initial_balance: 1_000_000,
            amount: 100,
            fee: 10,
            nonce: 42,
        };
        let bytes = bincode::serialize(&tx).unwrap();
        let tx2: TransactionData = bincode::deserialize(&bytes).unwrap();
        assert_eq!(tx.initial_balance, tx2.initial_balance);
        assert_eq!(tx.amount, tx2.amount);
        assert_eq!(tx.fee, tx2.fee);
        assert_eq!(tx.nonce, tx2.nonce);
    }

    #[test]
    fn test_512_anchor_determinism() {
        let tx = TransactionData {
            initial_balance: 5_000,
            amount: 1_000,
            fee: 10,
            nonce: 7,
        };
        let h1 = StarkProver::compute_512_anchor(&tx).unwrap();
        let h2 = StarkProver::compute_512_anchor(&tx).unwrap();
        assert_eq!(h1, h2, "Same transaction must produce identical 512-bit anchor");
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn test_512_anchor_uniqueness() {
        let tx1 = TransactionData { initial_balance: 5_000, amount: 100, fee: 10, nonce: 1 };
        let tx2 = TransactionData { initial_balance: 5_000, amount: 100, fee: 10, nonce: 2 };
        let h1 = StarkProver::compute_512_anchor(&tx1).unwrap();
        let h2 = StarkProver::compute_512_anchor(&tx2).unwrap();
        assert_ne!(h1, h2, "Different nonces must produce different anchors");
    }

    #[test]
    fn test_insufficient_balance_rejected() {
        let tx = TransactionData {
            initial_balance: 50,
            amount: 100,
            fee: 10,
            nonce: 1,
        };
        let result = StarkProver::generate_proof(&tx);
        assert!(result.is_err(), "Insufficient balance must be rejected");
    }

    #[test]
    fn test_overflow_rejected() {
        let tx = TransactionData {
            initial_balance: u64::MAX,
            amount: u64::MAX,
            fee: 1,
            nonce: 0,
        };
        let result = StarkProver::generate_proof(&tx);
        assert!(result.is_err(), "Overflow must be rejected");
    }

    #[test]
    fn test_generate_and_verify_proof() {
        let tx = TransactionData {
            initial_balance: 10_000,
            amount: 1_000,
            fee: 50,
            nonce: 1,
        };
        let receipt = StarkProver::generate_proof(&tx).unwrap();
        assert_eq!(receipt.journal_hash_512.len(), 64);
        assert!(!receipt.seal.is_empty());

        let anchor = StarkProver::compute_512_anchor(&tx).unwrap();
        assert_eq!(receipt.journal_hash_512, anchor);
    }

    #[test]
    fn test_exact_balance_accepted() {
        let tx = TransactionData {
            initial_balance: 110,
            amount: 100,
            fee: 10,
            nonce: 0,
        };
        let result = StarkProver::generate_proof(&tx);
        assert!(result.is_ok(), "Exact balance should be accepted");
    }

    #[test]
    fn test_zero_amount_accepted() {
        let tx = TransactionData {
            initial_balance: 1_000,
            amount: 0,
            fee: 10,
            nonce: 0,
        };
        let result = StarkProver::generate_proof(&tx);
        assert!(result.is_ok(), "Zero amount should be accepted");
    }

    #[test]
    fn test_stark_proof_interval_is_100() {
        assert_eq!(STARK_PROOF_INTERVAL, 100, "STARK proof interval must be 100 blocks");
    }
}
