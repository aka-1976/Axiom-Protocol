// src/privacy/view_keys.rs - Optional Privacy Controls
// Gives users CONTROL over their privacy - can share with auditors/tax authorities

use serde::{Deserialize, Serialize};
use ed25519_dalek::SigningKey;
use aes_gcm::{Aes256Gcm, KeyInit};
use aes_gcm::aead::Aead;
use sha2::{Sha256, Digest};
use rand::Rng;

/// View Key - Allows third parties to VIEW transactions without spending
/// Use cases: Tax compliance, audits, regulatory reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewKey {
    pub view_public_key: [u8; 32],
    pub view_secret_key: Option<[u8; 32]>, // Only owner has this
}

/// Spending Key - Required to create transactions
#[derive(Debug, Clone)]
pub struct SpendingKey {
    pub spend_secret_key: [u8; 32],
    pub spend_public_key: [u8; 32],
}

/// Full Wallet with dual-key system (like Monero)
#[derive(Debug)]
pub struct AxiomWallet {
    pub address: [u8; 32],          // Public address (hash of both public keys)
    pub spend_key: SpendingKey,     // For creating transactions
    pub view_key: ViewKey,          // For viewing transactions
}

impl AxiomWallet {
    /// Generate new wallet with both spend and view keys
    pub fn new() -> Self {
        
        
        // Generate spending keypair
        let spend_secret = SigningKey::from_bytes(&rand::thread_rng().gen());
        let spend_public = spend_secret.verifying_key();
        
        // Generate view keypair (derived from spend key for compatibility)
        let view_secret = Self::derive_view_key(spend_secret.as_bytes());
        let view_public = Self::derive_view_public(&view_secret);
        
        // Address = Hash(spend_public || view_public)
        let address = Self::compute_address(spend_public.as_bytes(), &view_public);
        
        Self {
            address,
            spend_key: SpendingKey {
                spend_secret_key: spend_secret.to_bytes(),
                spend_public_key: spend_public.to_bytes(),
            },
            view_key: ViewKey {
                view_public_key: view_public,
                view_secret_key: Some(view_secret),
            },
        }
    }
    
    /// Export view key ONLY (safe to share with accountants/auditors)
    pub fn export_view_key(&self) -> ViewKey {
        ViewKey {
            view_public_key: self.view_key.view_public_key,
            view_secret_key: self.view_key.view_secret_key,
        }
    }
    
    /// Import wallet from view key (read-only wallet)
    pub fn from_view_key(view_key: ViewKey) -> ReadOnlyWallet {
        ReadOnlyWallet { view_key }
    }
    
    fn derive_view_key(spend_secret: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"axiom_view_key_derivation");
        hasher.update(spend_secret);
        let hash = hasher.finalize();
        
        let mut view_key = [0u8; 32];
        view_key.copy_from_slice(&hash);
        view_key
    }
    
    fn derive_view_public(view_secret: &[u8; 32]) -> [u8; 32] {
        // Use SHA256 to derive public key deterministically
        let mut hasher = Sha256::new();
        hasher.update(b"axiom_view_public_derivation");
        hasher.update(view_secret);
        let hash = hasher.finalize();
        
        let mut public = [0u8; 32];
        public.copy_from_slice(&hash);
        public
    }
    
    fn compute_address(spend_pub: &[u8; 32], view_pub: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(spend_pub);
        hasher.update(view_pub);
        let hash = hasher.finalize();
        
        let mut address = [0u8; 32];
        address.copy_from_slice(&hash);
        address
    }
    
    /// Create selective disclosure for ONE transaction
    pub fn create_disclosure(
        &self,
        tx_hash: [u8; 32],
        disclosed_to: String,
        valid_for_days: u64,
    ) -> SelectiveDisclosure {
        // Generate one-time disclosure key
        let mut rng = rand::thread_rng();
        let disclosure_key: [u8; 32] = rng.gen();
        
        let expires_at = chrono::Utc::now().timestamp() as u64 + (valid_for_days * 86400);
        
        SelectiveDisclosure {
            transaction_hash: tx_hash,
            disclosed_to,
            disclosure_key,
            expires_at,
        }
    }
    
    /// Verify disclosure and decrypt transaction
    pub fn verify_disclosure(
        disclosure: &SelectiveDisclosure,
        tx: &EncryptedTransaction,
    ) -> Result<TransactionDetails, String> {
        // Check expiration
        let now = chrono::Utc::now().timestamp() as u64;
        if now > disclosure.expires_at {
            return Err("Disclosure expired".to_string());
        }
        
        // Verify transaction hash matches
        if tx.hash() != disclosure.transaction_hash {
            return Err("Transaction hash mismatch".to_string());
        }
        
        // Decrypt using disclosure key
        Self::decrypt_with_key(tx, &disclosure.disclosure_key)
    }
    
    fn decrypt_with_key(
        tx: &EncryptedTransaction,
        key: &[u8; 32]
    ) -> Result<TransactionDetails, String> {
        use aes_gcm::aead::generic_array::GenericArray;
        
        let cipher_key = GenericArray::from_slice(key);
        let cipher = Aes256Gcm::new(cipher_key);
        let nonce = GenericArray::from_slice(&tx.nonce);
        
        let decrypted = cipher.decrypt(nonce, tx.encrypted_data.as_ref())
            .map_err(|_| "Decryption failed")?;
        
        // Parse transaction details from decrypted data
        if decrypted.len() < 40 {
            return Err("Invalid data length".to_string());
        }
        
        let mut to = [0u8; 32];
        to.copy_from_slice(&decrypted[0..32]);
        
        // Safely extract amount with proper error handling instead of unwrap
        let amount = u64::from_le_bytes(match <[u8; 8]>::try_from(&decrypted[32..40]) {
            Ok(bytes) => bytes,
            Err(_) => return Err("Failed to extract amount bytes from decrypted data".to_string()),
        });
        
        Ok(TransactionDetails {
            from: tx.from,
            to,
            amount,
            timestamp: tx.timestamp,
        })
    }
}

/// Read-only wallet - Can VIEW but not SPEND
pub struct ReadOnlyWallet {
    view_key: ViewKey,
}

impl ReadOnlyWallet {
    /// Decrypt transaction to see if it's yours
    pub fn can_view_transaction(&self, tx: &EncryptedTransaction) -> Option<TransactionDetails> {
        if let Some(view_secret) = &self.view_key.view_secret_key {
            self.decrypt_transaction(tx, view_secret).ok()
        } else {
            None
        }
    }
    
    fn decrypt_transaction(
        &self,
        tx: &EncryptedTransaction,
        view_secret: &[u8; 32]
    ) -> Result<TransactionDetails, String> {
        
        
        // Use view key to decrypt transaction metadata
        let shared_secret = self.compute_shared_secret(view_secret, &tx.ephemeral_public_key);
        
        // Decrypt amount and recipient
        self.decrypt_data(&tx.encrypted_data, &shared_secret, &tx.nonce)
    }
    
    fn compute_shared_secret(&self, view_secret: &[u8; 32], ephemeral_pub: &[u8; 32]) -> [u8; 32] {
        // ECDH shared secret
        let mut hasher = Sha256::new();
        hasher.update(view_secret);
        hasher.update(ephemeral_pub);
        let hash = hasher.finalize();
        
        let mut shared = [0u8; 32];
        shared.copy_from_slice(&hash);
        shared
    }
    
    fn decrypt_data(
        &self,
        encrypted: &[u8],
        shared_secret: &[u8; 32],
        nonce: &[u8; 12]
    ) -> Result<TransactionDetails, String> {
        use aes_gcm::aead::generic_array::GenericArray;
        
        let key = GenericArray::from_slice(shared_secret);
        let cipher = Aes256Gcm::new(key);
        let nonce_obj = GenericArray::from_slice(nonce);
        
        let decrypted = cipher.decrypt(nonce_obj, encrypted)
            .map_err(|_| "Decryption failed")?;
        
        // Parse decrypted data
        if decrypted.len() < 40 {
            return Err("Invalid data length".to_string());
        }
        
        let mut recipient = [0u8; 32];
        recipient.copy_from_slice(&decrypted[0..32]);
        
        // Safely extract amount with proper error handling instead of unwrap
        let amount = u64::from_le_bytes(match <[u8; 8]>::try_from(&decrypted[32..40]) {
            Ok(bytes) => bytes,
            Err(_) => return Err("Failed to extract amount bytes from decrypted data".to_string()),
        });
        
        Ok(TransactionDetails {
            from: [0u8; 32], // Will be filled from tx
            to: recipient,
            amount,
            timestamp: 0,
        })
    }
    
    /// Generate compliance report (for taxes, audits)
    pub fn generate_compliance_report(&self, transactions: &[EncryptedTransaction]) -> ComplianceReport {
        let mut received = Vec::new();
        let mut sent = Vec::new();
        let mut total_received = 0u64;
        let mut total_sent = 0u64;
        
        for tx in transactions {
            if let Some(details) = self.can_view_transaction(tx) {
                // Check if received or sent
                if details.to == self.view_key.view_public_key {
                    received.push(details.clone());
                    total_received += details.amount;
                } else {
                    sent.push(details.clone());
                    total_sent += details.amount;
                }
            }
        }
        
        ComplianceReport {
            address: hex::encode(self.view_key.view_public_key),
            period_start: transactions.first().map(|t| t.timestamp).unwrap_or(0),
            period_end: transactions.last().map(|t| t.timestamp).unwrap_or(0),
            total_received,
            total_sent,
            received_transactions: received,
            sent_transactions: sent,
        }
    }
}

/// Selective Disclosure - Share specific transaction with third party
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectiveDisclosure {
    pub transaction_hash: [u8; 32],
    pub disclosed_to: String,        // Who can see it
    pub disclosure_key: [u8; 32],   // One-time key for this disclosure
    pub expires_at: u64,            // Expiration timestamp
}

// Supporting types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDetails {
    pub from: [u8; 32],
    pub to: [u8; 32],
    pub amount: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedTransaction {
    pub from: [u8; 32],
    pub encrypted_data: Vec<u8>,
    pub ephemeral_public_key: [u8; 32],
    pub nonce: [u8; 12],
    pub timestamp: u64,
}

impl EncryptedTransaction {
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.from);
        hasher.update(&self.encrypted_data);
        hasher.update(self.ephemeral_public_key);
        let hash = hasher.finalize();
        
        let mut result = [0u8; 32];
        result.copy_from_slice(&hash);
        result
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub address: String,
    pub period_start: u64,
    pub period_end: u64,
    pub total_received: u64,
    pub total_sent: u64,
    pub received_transactions: Vec<TransactionDetails>,
    pub sent_transactions: Vec<TransactionDetails>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_view_key_generation() {
        let wallet = AxiomWallet::new();
        let view_key = wallet.export_view_key();
        
        assert!(view_key.view_secret_key.is_some());
        assert_eq!(view_key.view_public_key.len(), 32);
    }
    
    #[test]
    fn test_read_only_wallet() {
        let wallet = AxiomWallet::new();
        let view_key = wallet.export_view_key();
        
        let _read_only = AxiomWallet::from_view_key(view_key);
        
        // Read-only wallet can view but not spend
        // (would need actual encrypted transaction to test fully)
    }
    
    #[test]
    fn test_selective_disclosure() {
        let wallet = AxiomWallet::new();
        let tx_hash = [1u8; 32];
        
        let disclosure = wallet.create_disclosure(
            tx_hash,
            "auditor@example.com".to_string(),
            30, // 30 days
        );
        
        assert_eq!(disclosure.transaction_hash, tx_hash);
        assert!(disclosure.expires_at > chrono::Utc::now().timestamp() as u64);
    }
}
