use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer};
use sha2::{Sha256, Digest};
use crate::types::Address;
use crate::transaction::Transaction;
use crate::error::{AxiomError, Result};

/// Axiom wallet with keypair
pub struct Wallet {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    address: Address,
}

impl std::fmt::Debug for Wallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Wallet")
            .field("address", &self.address)
            .finish_non_exhaustive()
    }
}

impl Wallet {
    /// Create a new random wallet
    pub fn new() -> Self {
        use rand::rngs::OsRng;
        
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        
        // Address = SHA256(public_key)
        let mut hasher = Sha256::new();
        hasher.update(verifying_key.as_bytes());
        let hash = hasher.finalize();
        
        let mut addr_bytes = [0u8; 32];
        addr_bytes.copy_from_slice(&hash);
        let address = Address(addr_bytes);
        
        Self {
            signing_key,
            verifying_key,
            address,
        }
    }
    
    /// Import wallet from secret key bytes
    pub fn from_secret_key(secret_bytes: [u8; 32]) -> Result<Self> {
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();
        
        let mut hasher = Sha256::new();
        hasher.update(verifying_key.as_bytes());
        let hash = hasher.finalize();
        
        let mut addr_bytes = [0u8; 32];
        addr_bytes.copy_from_slice(&hash);
        let address = Address(addr_bytes);
        
        Ok(Self {
            signing_key,
            verifying_key,
            address,
        })
    }
    
    /// Export secret key (keep this PRIVATE!)
    pub fn export_secret_key(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }
    
    /// Get wallet address
    pub fn address(&self) -> Address {
        self.address
    }
    
    /// Get address as hex string
    pub fn address_hex(&self) -> String {
        self.address.to_hex()
    }
    
    /// Get verifying (public) key
    pub fn verifying_key(&self) -> &VerifyingKey {
        &self.verifying_key
    }
    
    /// Create a new transaction
    pub fn create_transaction(
        &self,
        to: &str,
        amount: u64,
        fee: u64,
    ) -> Result<Transaction> {
        let to_addr = Address::from_hex(to)
            .map_err(|e| AxiomError::InvalidAddress(e))?;
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Create transaction structure
        let mut tx = Transaction {
            from: self.address,
            to: to_addr,
            amount,
            fee,
            nonce: 0, // Should be fetched from network
            timestamp,
            signature: vec![],
        };
        
        // Sign transaction
        let signature = self.sign_transaction(&tx)?;
        tx.signature = signature.to_bytes().to_vec();
        
        Ok(tx)
    }
    
    /// Sign a transaction
    fn sign_transaction(&self, tx: &Transaction) -> Result<Signature> {
        let message = tx.signing_message();
        let signature = self.signing_key.sign(&message);
        Ok(signature)
    }
    
    /// Sign arbitrary data
    pub fn sign(&self, data: &[u8]) -> Signature {
        self.signing_key.sign(data)
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wallet_creation() {
        let wallet = Wallet::new();
        let addr = wallet.address_hex();
        assert!(addr.starts_with("axm1"));
    }
    
    #[test]
    fn test_wallet_import_export() {
        let wallet1 = Wallet::new();
        let secret = wallet1.export_secret_key();
        
        let wallet2 = Wallet::from_secret_key(secret).unwrap();
        assert_eq!(wallet1.address(), wallet2.address());
    }
    
    #[test]
    fn test_create_transaction() {
        let wallet = Wallet::new();
        let to = Address([2u8; 32]).to_hex();
        
        let tx = wallet.create_transaction(&to, 1_000_000, 100_000).unwrap();
        assert_eq!(tx.amount, 1_000_000);
        assert_eq!(tx.fee, 100_000);
        assert!(!tx.signature.is_empty());
    }
}
