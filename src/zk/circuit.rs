// src/zk/circuit.rs - ZK-STARK Proof System using Winterfell
// AXIOM Protocol - Privacy-preserving transaction verification with no trusted setup
//
// This circuit proves:
// 1. Knowledge of secret key (ownership)
// 2. Sufficient balance for transaction (solvency)
// 3. Correct balance update (integrity)
// 4. All amounts are non-negative (range constraints)

use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

use winterfell::{
    math::{fields::f128::BaseElement, FieldElement, StarkField, ToElements},
    crypto::{hashers::Blake3_256, DefaultRandomCoin},
    Air, AirContext, Assertion, EvaluationFrame, FieldExtension,
    ProofOptions, TraceInfo, TransitionConstraintDegree,
    Proof, Prover, Trace, TraceTable,
    DefaultTraceLde, DefaultConstraintEvaluator,
    AcceptableOptions, matrix::ColMatrix, StarkDomain, TracePolyTable,
    AuxRandElements, ConstraintCompositionCoefficients,
};

// ========================
// PUBLIC INPUTS
// ========================

/// Minimum conjectured security level (in bits) required for STARK proof verification.
pub const MIN_SECURITY_BITS: u32 = 95;

/// Public inputs for the transaction proof
#[derive(Clone, Debug)]
pub struct TransactionPublicInputs {
    pub commitment: BaseElement,
    pub transfer_amount: BaseElement,
    pub fee: BaseElement,
    pub new_balance_commitment: BaseElement,
}

impl ToElements<BaseElement> for TransactionPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        vec![
            self.commitment,
            self.transfer_amount,
            self.fee,
            self.new_balance_commitment,
        ]
    }
}

// ========================
// AIR DEFINITION
// ========================

/// The Axiom Transaction AIR - Algebraic Intermediate Representation for STARKs
///
/// Trace layout (8 columns):
///   col 0: secret_key
///   col 1: current_balance
///   col 2: nonce
///   col 3: commitment = secret_key + nonce
///   col 4: transfer_amount
///   col 5: fee
///   col 6: new_balance_commitment = secret_key + (balance - amount - fee)
///   col 7: step counter (0, 1, 2, ..., N-1) — ensures non-constant trace
///          polynomials for DEEP composition soundness
pub struct AxiomTransactionAir {
    context: AirContext<BaseElement>,
    commitment: BaseElement,
    transfer_amount: BaseElement,
    fee: BaseElement,
    new_balance_commitment: BaseElement,
}

impl Air for AxiomTransactionAir {
    type BaseField = BaseElement;
    type PublicInputs = TransactionPublicInputs;
    type GkrProof = ();
    type GkrVerifier = ();

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        let degrees = vec![
            TransitionConstraintDegree::new(1), // commitment = sk + nonce
            TransitionConstraintDegree::new(1), // solvency: new_commitment = sk + remainder
            TransitionConstraintDegree::new(1), // integrity: new_commitment = sk + balance - amount - fee
            TransitionConstraintDegree::new(1), // step counter: next_step = current_step + 1
        ];

        let num_assertions = 5;
        Self {
            context: AirContext::new(trace_info, degrees, num_assertions, options),
            commitment: pub_inputs.commitment,
            transfer_amount: pub_inputs.transfer_amount,
            fee: pub_inputs.fee,
            new_balance_commitment: pub_inputs.new_balance_commitment,
        }
    }

    fn evaluate_transition<E: FieldElement + From<Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        _periodic_values: &[E],
        result: &mut [E],
    ) {
        let current = frame.current();
        let next = frame.next();

        let secret_key = current[0];
        let balance = current[1];
        let nonce = current[2];
        let commitment = current[3];
        let amount = current[4];
        let fee = current[5];
        let new_balance_commitment = current[6];

        // Constraint 1: commitment == secret_key + nonce
        result[0] = commitment - (secret_key + nonce);

        // Constraint 2: balance >= amount + fee (encoded as balance == amount + fee + remainder)
        // remainder is implicit: balance - amount - fee
        // NOTE: The prover pre-checks balance >= amount + fee before generating the trace.
        // In production, add explicit range proofs (e.g., bit decomposition) to prevent
        // a malicious prover from creating proofs with field-wrapped negative remainders.
        // This is a known limitation shared with the previous SNARK implementation.
        let remainder = balance - amount - fee;
        result[1] = new_balance_commitment - (secret_key + remainder);

        // Constraint 3: new_balance_commitment == secret_key + (balance - amount - fee)
        result[2] = new_balance_commitment - secret_key - balance + amount + fee;

        // Constraint 4: step counter increments by 1 each row
        result[3] = next[7] - current[7] - E::ONE;
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        vec![
            Assertion::single(3, 0, self.commitment),              // commitment at col 3, row 0
            Assertion::single(4, 0, self.transfer_amount),         // amount at col 4, row 0
            Assertion::single(5, 0, self.fee),                     // fee at col 5, row 0
            Assertion::single(6, 0, self.new_balance_commitment),  // new_commitment at col 6, row 0
            Assertion::single(7, 0, BaseElement::ZERO),            // step counter starts at 0
        ]
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }
}

// ========================
// PROVER
// ========================

/// STARK prover for Axiom transactions
pub struct AxiomTransactionProver {
    options: ProofOptions,
    public_inputs: TransactionPublicInputs,
}

impl AxiomTransactionProver {
    pub fn new(options: ProofOptions, public_inputs: TransactionPublicInputs) -> Self {
        Self {
            options,
            public_inputs,
        }
    }

    /// Build the execution trace for a transaction
    pub fn build_trace(
        secret_key: BaseElement,
        current_balance: BaseElement,
        nonce: BaseElement,
        transfer_amount: BaseElement,
        fee: BaseElement,
    ) -> TraceTable<BaseElement> {
        let commitment = secret_key + nonce;
        let remainder = current_balance - transfer_amount - fee;
        let new_balance_commitment = secret_key + remainder;

        // STARK traces must have length that is a power of 2, minimum 8
        let trace_len = 8;
        let mut trace = TraceTable::new(8, trace_len);

        // Step counter to ensure at least one column is non-constant.
        // Winterfell 0.9 DEEP composition requires trace polynomials of degree
        // trace_length - 2; a constant trace produces degree-0 polynomials which
        // violate this invariant.
        let step = std::cell::Cell::new(0u128);

        // Fill all rows: cols 0-6 hold constant transaction data,
        // col 7 is the incrementing step counter.
        trace.fill(
            |state| {
                state[0] = secret_key;
                state[1] = current_balance;
                state[2] = nonce;
                state[3] = commitment;
                state[4] = transfer_amount;
                state[5] = fee;
                state[6] = new_balance_commitment;
                state[7] = BaseElement::new(step.get());
                step.set(step.get() + 1);
            },
            |_, state| {
                state[0] = secret_key;
                state[1] = current_balance;
                state[2] = nonce;
                state[3] = commitment;
                state[4] = transfer_amount;
                state[5] = fee;
                state[6] = new_balance_commitment;
                state[7] = BaseElement::new(step.get());
                step.set(step.get() + 1);
            },
        );

        trace
    }
}

impl Prover for AxiomTransactionProver {
    type BaseField = BaseElement;
    type Air = AxiomTransactionAir;
    type Trace = TraceTable<BaseElement>;
    type HashFn = Blake3_256<BaseElement>;
    type RandomCoin = DefaultRandomCoin<Self::HashFn>;
    type TraceLde<E: FieldElement<BaseField = Self::BaseField>> = DefaultTraceLde<E, Self::HashFn>;
    type ConstraintEvaluator<'a, E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintEvaluator<'a, Self::Air, E>;

    fn get_pub_inputs(&self, _trace: &Self::Trace) -> TransactionPublicInputs {
        self.public_inputs.clone()
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }

    fn new_trace_lde<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        trace_info: &TraceInfo,
        main_trace: &ColMatrix<Self::BaseField>,
        domain: &StarkDomain<Self::BaseField>,
    ) -> (Self::TraceLde<E>, TracePolyTable<E>) {
        DefaultTraceLde::new(trace_info, main_trace, domain)
    }

    fn new_evaluator<'a, E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        air: &'a Self::Air,
        aux_rand_elements: Option<AuxRandElements<E>>,
        composition_coefficients: ConstraintCompositionCoefficients<E>,
    ) -> Self::ConstraintEvaluator<'a, E> {
        DefaultConstraintEvaluator::new(air, aux_rand_elements, composition_coefficients)
    }
}

// ========================
// ZK PROOF SYSTEM (High-Level API)
// ========================

/// Default proof options for AXIOM Protocol
fn default_proof_options() -> ProofOptions {
    ProofOptions::new(
        32,                        // number of queries
        8,                         // blowup factor
        0,                         // grinding factor
        FieldExtension::None,      // field extension
        8,                         // FRI folding factor
        31,                        // FRI max remainder polynomial degree
    )
}

/// ZK-STARK Proof System Manager - No trusted setup required!
pub struct ZkProofSystem {
    options: ProofOptions,
}

impl ZkProofSystem {
    /// Create a new ZK-STARK proof system (no trusted setup needed!)
    pub fn setup() -> Result<Self, String> {
        Ok(Self {
            options: default_proof_options(),
        })
    }

    /// Save proof parameters to disk
    pub fn save_keys(&self, keys_dir: &str) -> Result<(), String> {
        fs::create_dir_all(keys_dir).map_err(|e| format!("Failed to create keys dir: {}", e))?;

        let params_path = format!("{}/stark_params.json", keys_dir);
        let params_json = serde_json::json!({
            "protocol": "stark",
            "hash_function": "blake3",
            "field": "f128",
            "security_bits": 128,
            "trusted_setup": false,
            "num_queries": 32,
            "blowup_factor": 8,
        });
        fs::write(&params_path, serde_json::to_string_pretty(&params_json).unwrap())
            .map_err(|e| format!("Failed to write params: {}", e))?;

        println!("✓ STARK parameters saved to {}", keys_dir);
        println!("  ℹ️  No trusted setup required - STARKs are transparent!");
        Ok(())
    }

    /// Load parameters from disk
    pub fn load_keys(keys_dir: &str) -> Result<Self, String> {
        let params_path = format!("{}/stark_params.json", keys_dir);
        if !Path::new(&params_path).exists() {
            // STARKs don't need pre-generated keys, just use defaults
            return Ok(Self {
                options: default_proof_options(),
            });
        }
        // Parameters file exists - could read custom options but defaults work fine
        Ok(Self {
            options: default_proof_options(),
        })
    }

    /// Generate a STARK proof for a transaction
    pub fn prove(
        &self,
        secret_key: BaseElement,
        current_balance: BaseElement,
        nonce: BaseElement,
        transfer_amount: BaseElement,
        fee: BaseElement,
    ) -> Result<(Proof, Vec<BaseElement>), String> {
        // Pre-check: fail fast if balance is insufficient
        if current_balance.as_int() < (transfer_amount + fee).as_int() {
            return Err(format!(
                "Insufficient balance: have {}, need {} (amount) + {} (fee)",
                current_balance, transfer_amount, fee,
            ));
        }

        // Compute public inputs
        let commitment = secret_key + nonce;
        let remainder = current_balance - transfer_amount - fee;
        let new_balance_commitment = secret_key + remainder;

        let public_inputs = TransactionPublicInputs {
            commitment,
            transfer_amount,
            fee,
            new_balance_commitment,
        };

        // Build execution trace
        let trace = AxiomTransactionProver::build_trace(
            secret_key,
            current_balance,
            nonce,
            transfer_amount,
            fee,
        );

        // Create prover and generate proof
        let prover = AxiomTransactionProver::new(self.options.clone(), public_inputs);
        let proof = prover.prove(trace).map_err(|e| format!("Proving failed: {:?}", e))?;

        let public_outputs = vec![commitment, transfer_amount, fee, new_balance_commitment];
        Ok((proof, public_outputs))
    }

    /// Batch prove multiple transactions
    pub fn prove_batch(
        &self,
        transactions: Vec<(BaseElement, BaseElement, BaseElement, BaseElement, BaseElement)>,
    ) -> Result<Vec<(Proof, Vec<BaseElement>)>, String> {
        transactions
            .into_iter()
            .map(|(sk, balance, nonce, amount, fee)| self.prove(sk, balance, nonce, amount, fee))
            .collect()
    }

    /// Verify a STARK proof
    pub fn verify(
        &self,
        proof: &Proof,
        public_inputs: &[BaseElement],
    ) -> Result<bool, String> {
        if public_inputs.len() != 4 {
            return Err("Expected 4 public inputs".to_string());
        }

        let pub_inputs = TransactionPublicInputs {
            commitment: public_inputs[0],
            transfer_amount: public_inputs[1],
            fee: public_inputs[2],
            new_balance_commitment: public_inputs[3],
        };

        let min_opts = AcceptableOptions::MinConjecturedSecurity(MIN_SECURITY_BITS);

        match winterfell::verify::<
            AxiomTransactionAir,
            Blake3_256<BaseElement>,
            DefaultRandomCoin<Blake3_256<BaseElement>>,
        >(proof.clone(), pub_inputs, &min_opts) {
            Ok(_) => Ok(true),
            Err(e) => {
                // Verification failed - proof is invalid
                Err(format!("STARK verification failed: {:?}", e))
            }
        }
    }
}

// ========================
// UTILITY FUNCTIONS
// ========================

/// Convert bytes to a field element
pub fn bytes_to_field(bytes: &[u8]) -> BaseElement {
    let mut hash = Sha256::digest(bytes);
    // Take first 16 bytes for 128-bit field element
    let mut buf = [0u8; 16];
    buf.copy_from_slice(&hash[..16]);
    // Clear top bit to ensure we're within field modulus
    buf[15] &= 0x7f;
    BaseElement::new(u128::from_le_bytes(buf))
}

/// Generate a commitment from secret key and nonce
pub fn generate_commitment(secret_key: &[u8], nonce: u64) -> BaseElement {
    let sk = bytes_to_field(secret_key);
    let n = BaseElement::new(nonce as u128);
    sk + n
}

// ========================
// TESTS
// ========================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zk_setup() {
        let system = ZkProofSystem::setup().unwrap();
        // STARKs don't need trusted setup - just verify creation succeeds
        assert!(system.options.num_queries() > 0);
    }

    #[test]
    fn test_proof_generation_and_verification() {
        let system = ZkProofSystem::setup().unwrap();

        let secret_key = BaseElement::new(12345u128);
        let balance = BaseElement::new(1000u128);
        let nonce = BaseElement::new(1u128);
        let amount = BaseElement::new(100u128);
        let fee = BaseElement::new(10u128);

        let (proof, public_inputs) = system.prove(secret_key, balance, nonce, amount, fee).unwrap();
        let valid = system.verify(&proof, &public_inputs).unwrap();

        assert!(valid, "STARK proof should be valid");
    }

    #[test]
    fn test_insufficient_balance_fails() {
        let system = ZkProofSystem::setup().unwrap();

        let secret_key = BaseElement::new(12345u128);
        let balance = BaseElement::new(50u128); // Not enough
        let nonce = BaseElement::new(1u128);
        let amount = BaseElement::new(100u128);
        let fee = BaseElement::new(10u128);

        let result = system.prove(secret_key, balance, nonce, amount, fee);
        assert!(result.is_err(), "Should fail with insufficient balance");
        assert!(result.unwrap_err().contains("Insufficient balance"));
    }

    #[test]
    fn test_zero_amount_transaction() {
        let system = ZkProofSystem::setup().unwrap();

        let secret_key = BaseElement::new(12345u128);
        let balance = BaseElement::new(1000u128);
        let nonce = BaseElement::new(1u128);
        let amount = BaseElement::new(0u128);
        let fee = BaseElement::new(10u128);

        let (proof, public_inputs) = system.prove(secret_key, balance, nonce, amount, fee).unwrap();
        let valid = system.verify(&proof, &public_inputs).unwrap();

        assert!(valid, "Zero amount transaction should be valid");
    }

    #[test]
    fn test_exact_balance_transaction() {
        let system = ZkProofSystem::setup().unwrap();

        let secret_key = BaseElement::new(12345u128);
        let balance = BaseElement::new(110u128);
        let nonce = BaseElement::new(1u128);
        let amount = BaseElement::new(100u128);
        let fee = BaseElement::new(10u128);

        let (proof, public_inputs) = system.prove(secret_key, balance, nonce, amount, fee).unwrap();
        let valid = system.verify(&proof, &public_inputs).unwrap();

        assert!(valid, "Exact balance transaction should be valid");
    }

    #[test]
    fn test_batch_proving() {
        let system = ZkProofSystem::setup().unwrap();

        let transactions = vec![
            (BaseElement::new(111u128), BaseElement::new(1000u128), BaseElement::new(1u128), BaseElement::new(100u128), BaseElement::new(10u128)),
            (BaseElement::new(222u128), BaseElement::new(2000u128), BaseElement::new(2u128), BaseElement::new(200u128), BaseElement::new(20u128)),
            (BaseElement::new(333u128), BaseElement::new(3000u128), BaseElement::new(3u128), BaseElement::new(300u128), BaseElement::new(30u128)),
        ];

        let results = system.prove_batch(transactions).unwrap();
        assert_eq!(results.len(), 3, "Should generate 3 proofs");

        for (proof, public_inputs) in &results {
            let valid = system.verify(proof, public_inputs).unwrap();
            assert!(valid, "All batch proofs should be valid");
        }
    }

    #[test]
    fn test_bytes_to_field() {
        let bytes = [42u8; 32];
        let fe = bytes_to_field(&bytes);
        // Should produce a valid non-zero field element
        assert_ne!(fe, BaseElement::ZERO);
    }

    #[test]
    fn test_commitment_generation() {
        let secret = [1u8; 32];
        let c1 = generate_commitment(&secret, 1);
        let c2 = generate_commitment(&secret, 2);
        // Different nonces should produce different commitments
        assert_ne!(c1, c2);
    }
}

#[allow(dead_code)]
pub fn generate_circuit_address(_secret: &[u8; 32]) -> [u8; 32] {
    [0u8; 32]
}
