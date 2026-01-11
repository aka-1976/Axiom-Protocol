use ark_ff::PrimeField;
use ark_relations::lc;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError, Variable};

#[derive(Clone, Copy)]
pub struct QubitTransactionCircuit<F: PrimeField> {
    pub secret_key: Option<F>,
    pub current_balance: Option<F>,
    pub public_address: Option<F>,
    pub transfer_amount: Option<F>,
    pub fee: Option<F>,
}

impl<F: PrimeField> ConstraintSynthesizer<F> for QubitTransactionCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        let secret_key_var = cs.new_witness_variable(|| self.secret_key.ok_or(SynthesisError::AssignmentMissing))?;
        let balance_var = cs.new_witness_variable(|| self.current_balance.ok_or(SynthesisError::AssignmentMissing))?;
        let address_var = cs.new_input_variable(|| self.public_address.ok_or(SynthesisError::AssignmentMissing))?;
        let amount_var = cs.new_input_variable(|| self.transfer_amount.ok_or(SynthesisError::AssignmentMissing))?;
        let fee_var = cs.new_input_variable(|| self.fee.ok_or(SynthesisError::AssignmentMissing))?;

        cs.enforce_constraint(lc!() + secret_key_var, lc!() + Variable::One, lc!() + address_var)?;

        let remainder_val = if let (Some(b), Some(a), Some(f)) = (self.current_balance, self.transfer_amount, self.fee) {
            Some(b - (a + f))
        } else { None };

        let remainder_var = cs.new_witness_variable(|| remainder_val.ok_or(SynthesisError::AssignmentMissing))?;
        cs.enforce_constraint(lc!() + amount_var + fee_var + remainder_var, lc!() + Variable::One, lc!() + balance_var)?;
        Ok(())
    }
}

#[allow(dead_code)]
pub fn generate_circuit_address(_secret: &[u8; 32]) -> [u8; 32] {
    [0u8; 32]
}
