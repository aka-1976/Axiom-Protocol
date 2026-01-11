use crate::transaction::{Address, Transaction};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize};
use std::fs;

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
        let signing_key = SigningKey::generate(&mut rng);
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

    /// Generates a Zero-Knowledge Transaction.
    /// Proves ownership and balance without revealing the 'Secret Key'.
    pub fn create_private_tx(&self, to: Address, amount: u64, fee: u64, nonce: u64) -> Transaction {
        // 1. Prepare the Witness (Private Data)
        let _secret = self.secret_key; // Hidden from the network
        
        // 2. Generate ZK-Proof logic
        // In a production QBT node, this calls a circuit (e.g., arkworks/groth16)
        // to prove: Hash(Secret) == self.address AND balance >= (amount + fee).
        // For now, we create a placeholder proof that we will fill with 
        // the 'bellman' crate in the next step.
        let zk_proof_placeholder = vec![0u8; 64]; 

        Transaction {
            from: self.address,
            to,
            amount,
            fee,
            nonce,
            zk_proof: zk_proof_placeholder,
        }
    }

    /// Sign data for non-transactional network messages (P2P handshakes)
    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        let signing_key = SigningKey::from_bytes(&self.secret_key);
        let signature = ed25519_dalek::Signer::sign(&signing_key, message);
        signature.to_bytes().to_vec()
    }
}
