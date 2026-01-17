use crate::transaction::{Address, Transaction};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use rand_core::RngCore;
use serde::{Serialize, Deserialize};
use std::fs;
use crate::genesis;

#[derive(Serialize, Deserialize)]
pub struct Wallet {
    pub secret_key: [u8; 32],
    pub address: Address,
}

impl Wallet {
    /// Generates or loads a wallet from local storage.
    /// This keeps your identity strictly off-chain and local.
    pub fn load_or_create() -> Self {
        if let Ok(data) = fs::read("wallet.dat") {
            if let Ok(w) = bincode::deserialize::<Wallet>(&data) {
                return w;
            }
        }

        let mut rng = OsRng;
        let mut seed = [0u8; 32];
        rng.fill_bytes(&mut seed);
        let signing_key = SigningKey::from_bytes(&seed);
        let verifying_key = VerifyingKey::from(&signing_key);
        let address: Address = verifying_key.to_bytes();

        let wallet = Wallet {
            secret_key: signing_key.to_bytes(),
            address,
        };

        let encoded = bincode::serialize(&wallet).unwrap();
        fs::write("wallet.dat", encoded).expect("Failed to secure wallet file");
        wallet
    }

    /// Create a signed transaction with ZK proof
    pub fn create_transaction(
        &self,
        to: Address,
        amount: u64,
        fee: u64,
        nonce: u64,
        current_balance: u64,
    ) -> Result<Transaction, Box<dyn std::error::Error>> {
        // Generate ZK proof
        let zk_proof = genesis::generate_transaction_proof(
            &self.secret_key,
            current_balance,
            amount,
            fee,
        )?;

        // Create transaction data for signing
        let tx_data = Transaction::new(
            self.address,
            to,
            amount,
            fee,
            nonce,
            zk_proof,
            vec![], // Empty signature for now
        );

        // Sign the transaction
        let signature = self.sign_transaction(&tx_data)?;

        // Create final transaction with signature
        Ok(Transaction::new(
            self.address,
            to,
            amount,
            fee,
            nonce,
            tx_data.zk_proof,
            signature,
        ))
    }

    /// Sign transaction data
    fn sign_transaction(&self, tx: &Transaction) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let signing_key = SigningKey::from_bytes(&self.secret_key);

        // Create message to sign (transaction hash without signature)
        let mut tx_for_signing = tx.clone();
        tx_for_signing.signature = vec![]; // Clear signature for signing
        let message = bincode::serialize(&tx_for_signing)?;

        let signature: Signature = signing_key.sign(&message);
        Ok(signature.to_bytes().to_vec())
    }

    /// Verify transaction signature
    pub fn verify_transaction_signature(tx: &Transaction) -> Result<bool, Box<dyn std::error::Error>> {
        if tx.signature.len() != 64 {
            return Ok(false);
        }

        let verifying_key = VerifyingKey::from_bytes(&tx.from)?;

        // Create message that was signed
        let mut tx_for_verification = tx.clone();
        tx_for_verification.signature = vec![]; // Clear signature for verification
        let message = bincode::serialize(&tx_for_verification)?;

        let signature_bytes: [u8; 64] = tx.signature[..64].try_into().map_err(|_| "Invalid signature length")?;
        let signature = Signature::from_bytes(&signature_bytes);

        Ok(verifying_key.verify(&message, &signature).is_ok())
    }

    /// Get wallet address as hex string
    pub fn address_hex(&self) -> String {
        hex::encode(self.address)
    }

    /// Get balance from chain state
    pub fn get_balance(&self, chain: &crate::chain::Timechain) -> u64 {
        chain.balance(&self.address)
    }

    /// Get next nonce from chain state
    pub fn get_next_nonce(&self, chain: &crate::chain::Timechain) -> u64 {
        chain.state.next_nonce(&self.address)
    }

    /// Sign data for non-transactional network messages (P2P handshakes)
    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        let signing_key = SigningKey::from_bytes(&self.secret_key);
        let signature = ed25519_dalek::Signer::sign(&signing_key, message);
        signature.to_bytes().to_vec()
    }
}
