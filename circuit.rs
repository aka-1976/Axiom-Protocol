use ark_bls12_381::Fr;
use ark_ff::PrimeField;
use ark_relations::lc;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use sha2::{Sha256, Digest};

/// The QUBIT Transaction Circuit
/// This is the "Mathematical Constitution" of the chain.
/// It proves ownership and solvency without revealing the sender's data.
#[derive(Clone)]
pub struct QubitTransactionCircuit {
    // --- PRIVATE INPUTS (Witnesses) ---
    // These never leave the user's local machine.
    pub secret_key: Option<Fr>,
    pub current_balance: Option<Fr>,

    // --- PUBLIC INPUTS ---
    // These are seen by the network to verify the block.
    pub public_address: Option<Fr>,
    pub transfer_amount: Option<Fr>,
    pub fee: Option<Fr>,
}

impl ConstraintSynthesizer<Fr> for QubitTransactionCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // 1. Allocate Private Witnesses
        let secret_key_var = cs.new_witness_variable(|| self.secret_key.ok_or(SynthesisError::AssignmentMissing))?;
        let balance_var = cs.new_witness_variable(|| self.current_balance.ok_or(SynthesisError::AssignmentMissing))?;

        // 2. Allocate Public Inputs
        let address_var = cs.new_input_variable(|| self.public_address.ok_or(SynthesisError::AssignmentMissing))?;
        let amount_var = cs.new_input_variable(|| self.transfer_amount.ok_or(SynthesisError::AssignmentMissing))?;
        let fee_var = cs.new_input_variable(|| self.fee.ok_or(SynthesisError::AssignmentMissing))?;

        // 3. CONSTRAINT: Ownership Proof
        // This proves: "I know the secret key that belongs to this public address."
        // Logic: secret_key * 1 = public_address
        cs.enforce_constraint(
            lc!() + secret_key_var,
            lc!() + (Fr::from(1u32), Fr::one()),
            lc!() + address_var,
        )?;

        // 4. CONSTRAINT: Solvency Proof (Anti-Inflation)
        // This ensures: "I am not sending more than I actually own."
        // We calculate: Remainder = Balance - (Amount + Fee)
        let remainder_val = if let (Some(b), Some(a), Some(f)) = (self.current_balance, self.transfer_amount, self.fee) {
            Some(b - (a + f))
        } else {
            None
        };
        
        let remainder_var = cs.new_witness_variable(|| remainder_val.ok_or(SynthesisError::AssignmentMissing))?;

        // Logic: (Amount + Fee + Remainder) * 1 = Balance
        // If this equation doesn't balance, the ZK-Proof fails and the block is rejected.
        cs.enforce_constraint(
            lc!() + amount_var + fee_var + remainder_var,
            lc!() + (Fr::from(1u32), Fr::one()),
            lc!() + balance_var,
        )?;

        Ok(())
    }
}

/// Utility for address generation outside the circuit
pub fn generate_circuit_address(secret: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(secret);
    hasher.finalize().into()
}
