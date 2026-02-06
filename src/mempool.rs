// src/mempool.rs - AXIOM Protocol Production Transaction Pool

use std::collections::{HashMap, HashSet, BTreeMap};
use sha2::Digest;
use crate::{transaction::Transaction, error::{AxiomError, Result}};

pub type Address = [u8; 32];

const DEFAULT_MAX_SIZE: usize = 100_000;
const DEFAULT_MAX_TX_SIZE: usize = 100_000;

/// Production-grade transaction mempool
pub struct Mempool {
    /// All transactions indexed by hash
    transactions: HashMap<[u8; 32], Transaction>,
    /// Transactions sorted by fee (high to low)
    by_fee: BTreeMap<u64, HashSet<[u8; 32]>>,
    /// Transactions grouped by sender
    by_sender: HashMap<Address, Vec<[u8; 32]>>,
    /// Nullifiers to prevent double-spend
    nullifiers: HashSet<[u8; 32]>,
    /// Maximum mempool size
    max_size: usize,
    /// Maximum transaction size
    max_tx_size: usize,
}

impl Mempool {
    pub fn new() -> Self {
        Self {
            transactions: HashMap::new(),
            by_fee: BTreeMap::new(),
            by_sender: HashMap::new(),
            nullifiers: HashSet::new(),
            max_size: DEFAULT_MAX_SIZE,
            max_tx_size: DEFAULT_MAX_TX_SIZE,
        }
    }
    
    pub fn with_capacity(max_size: usize, max_tx_size: usize) -> Self {
        Self {
            transactions: HashMap::with_capacity(max_size),
            by_fee: BTreeMap::new(),
            by_sender: HashMap::new(),
            nullifiers: HashSet::new(),
            max_size,
            max_tx_size,
        }
    }
    
    /// Add transaction to mempool
    pub fn add(&mut self, tx: Transaction) -> Result<()> {
        let hash = tx.hash();
        
        // Calculate size
        let tx_size = bincode::serialize(&tx)
            .map_err(|e| AxiomError::SerializationError(e.to_string()))?
            .len();
        
        // Check size limit
        if tx_size > self.max_tx_size {
            return Err(AxiomError::TransactionTooLarge {
                size: tx_size,
                max: self.max_tx_size,
            });
        }
        
        // Check for duplicates
        if self.transactions.contains_key(&hash) {
            return Err(AxiomError::DuplicateTransaction);
        }
        
        // Generate nullifier (hash of from + nonce)
        let nullifier = {
            let mut hasher = sha2::Sha256::new();
            hasher.update(tx.from);
            hasher.update(tx.nonce.to_le_bytes());
            let result = hasher.finalize();
            let mut n = [0u8; 32];
            n.copy_from_slice(&result);
            n
        };
        
        // Check nullifier (double-spend protection)
        if self.nullifiers.contains(&nullifier) {
            return Err(AxiomError::NullifierUsed);
        }
        
        // Check mempool capacity
        if self.transactions.len() >= self.max_size {
            // Try to evict lowest fee transaction
            if let Some((&lowest_fee, _)) = self.by_fee.iter().next() {
                if tx.fee <= lowest_fee {
                    return Err(AxiomError::FeeTooLow {
                        min: lowest_fee + 1,
                        actual: tx.fee,
                    });
                }
                self.evict_lowest_fee();
            }
        }
        
        // Add to indexes
        self.by_fee
            .entry(tx.fee)
            .or_default()
            .insert(hash);
        
        self.by_sender
            .entry(tx.from)
            .or_default()
            .push(hash);
        
        self.nullifiers.insert(nullifier);
        self.transactions.insert(hash, tx);
        
        Ok(())
    }
    
    /// Get transactions for mining (highest fee first)
    pub fn get_for_mining(&self, max_count: usize) -> Vec<Transaction> {
        let mut result: Vec<Transaction> = Vec::with_capacity(max_count);
        
        // Iterate from highest fee to lowest
        for (_, hashes) in self.by_fee.iter().rev() {
            for hash in hashes {
                if let Some(tx) = self.transactions.get(hash) {
                    result.push(tx.clone());
                    if result.len() >= max_count {
                        return result;
                    }
                }
            }
        }
        
        result
    }
    
    /// Get transaction by hash
    pub fn get(&self, hash: &[u8; 32]) -> Option<&Transaction> {
        self.transactions.get(hash)
    }
    
    /// Check if transaction exists
    pub fn contains(&self, hash: &[u8; 32]) -> bool {
        self.transactions.contains_key(hash)
    }
    
    /// Remove transaction (after mining or expiry)
    pub fn remove(&mut self, hash: &[u8; 32]) -> Option<Transaction> {
        if let Some(tx) = self.transactions.remove(hash) {
            // Remove from fee index
            if let Some(hashes) = self.by_fee.get_mut(&tx.fee) {
                hashes.remove(hash);
                if hashes.is_empty() {
                    self.by_fee.remove(&tx.fee);
                }
            }
            
            // Remove from sender index
            if let Some(hashes) = self.by_sender.get_mut(&tx.from) {
                hashes.retain(|h| h != hash);
                if hashes.is_empty() {
                    self.by_sender.remove(&tx.from);
                }
            }
            
            // Remove nullifier
            let nullifier = {
                let mut hasher = sha2::Sha256::new();
                hasher.update(tx.from);
                hasher.update(tx.nonce.to_le_bytes());
                let result = hasher.finalize();
                let mut n = [0u8; 32];
                n.copy_from_slice(&result);
                n
            };
            self.nullifiers.remove(&nullifier);
            
            Some(tx)
        } else {
            None
        }
    }
    
    /// Remove multiple transactions (batch operation)
    pub fn remove_batch(&mut self, hashes: &[[u8; 32]]) {
        for hash in hashes {
            self.remove(hash);
        }
    }
    
    /// Get all transactions from a sender
    pub fn get_by_sender(&self, sender: &Address) -> Vec<Transaction> {
        self.by_sender
            .get(sender)
            .map(|hashes: &Vec<[u8; 32]>| {
                hashes.iter()
                    .filter_map(|hash| self.transactions.get(hash).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Evict lowest fee transaction
    fn evict_lowest_fee(&mut self) {
        if let Some((_, hashes)) = self.by_fee.iter().next() {
            if let Some(&hash) = hashes.iter().next() {
                self.remove(&hash);
            }
        }
    }
    
    /// Get mempool size
    pub fn len(&self) -> usize {
        self.transactions.len()
    }
    
    /// Check if mempool is empty
    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }
    
    /// Get total fees in mempool
    pub fn total_fees(&self) -> u64 {
        self.transactions.values().map(|tx| tx.fee).sum()
    }
    
    /// Clear all transactions
    pub fn clear(&mut self) {
        self.transactions.clear();
        self.by_fee.clear();
        self.by_sender.clear();
        self.nullifiers.clear();
    }
    
    /// Get mempool statistics
    pub fn stats(&self) -> MempoolStats {
        MempoolStats {
            size: self.len(),
            total_fees: self.total_fees(),
            unique_senders: self.by_sender.len(),
            highest_fee: self.by_fee.keys().next_back().copied().unwrap_or(0),
            lowest_fee: self.by_fee.keys().next().copied().unwrap_or(0),
        }
    }
}

impl Default for Mempool {
    fn default() -> Self {
        Self::new()
    }
}

/// Mempool statistics
#[derive(Debug, Clone)]
pub struct MempoolStats {
    pub size: usize,
    pub total_fees: u64,
    pub unique_senders: usize,
    pub highest_fee: u64,
    pub lowest_fee: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_transaction(amount: u64, fee: u64, nonce: u64) -> Transaction {
        Transaction {
            from: [1u8; 32],
            to: [2u8; 32],
            amount,
            fee,
            nonce,
            zk_proof: vec![],
            signature: vec![],
        }
    }
    
    #[test]
    fn test_mempool_add() {
        let mut mempool = Mempool::new();
        let tx = create_test_transaction(100, 10, 0);
        
        assert!(mempool.add(tx).is_ok());
        assert_eq!(mempool.len(), 1);
    }
    
    #[test]
    fn test_mempool_duplicate() {
        let mut mempool = Mempool::new();
        let tx = create_test_transaction(100, 10, 0);
        
        assert!(mempool.add(tx.clone()).is_ok());
        assert!(mempool.add(tx).is_err());
    }
    
    #[test]
    fn test_mempool_fee_ordering() {
        let mut mempool = Mempool::new();
        
        assert!(mempool.add(create_test_transaction(100, 5, 0)).is_ok(), "Failed to add tx with fee 5");
        assert!(mempool.add(create_test_transaction(100, 10, 1)).is_ok(), "Failed to add tx with fee 10");
        assert!(mempool.add(create_test_transaction(100, 1, 2)).is_ok(), "Failed to add tx with fee 1");
        
        let txs = mempool.get_for_mining(3);
        assert_eq!(txs[0].fee, 10);
        assert_eq!(txs[1].fee, 5);
        assert_eq!(txs[2].fee, 1);
    }
    
    #[test]
    fn test_mempool_eviction() {
        let mut mempool = Mempool::with_capacity(2, DEFAULT_MAX_TX_SIZE);
        
        assert!(mempool.add(create_test_transaction(100, 5, 0)).is_ok(), "Failed to add tx with fee 5");
        assert!(mempool.add(create_test_transaction(100, 10, 1)).is_ok(), "Failed to add tx with fee 10");
        
        // This should evict the lowest fee (5)
        assert!(mempool.add(create_test_transaction(100, 15, 2)).is_ok(), "Failed to add tx with fee 15");
        
        assert_eq!(mempool.len(), 2);
        let stats = mempool.stats();
        assert_eq!(stats.lowest_fee, 10);
    }
}
