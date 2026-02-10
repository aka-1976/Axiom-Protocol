// src/validation/ml_transaction_validator.rs
// Production ML-based transaction validation with feature extraction,
// anomaly detection, and incremental retraining.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;

use parking_lot::RwLock;

use crate::ai_core::production_ml::ProductionMLStack;
use crate::block::Block;
use crate::transaction::Transaction;

// ---------------------------------------------------------------------------
// Feature Extraction
// ---------------------------------------------------------------------------

/// Extracts a fixed-size numeric feature vector from a [`Transaction`].
///
/// Features produced (in order):
///   0. amount_zscore   – z-score of `amount` vs running statistics
///   1. fee_zscore      – z-score of `fee` vs running statistics
///   2. amount_fee_ratio – `amount / (fee + 1)`
///   3. hour            – hour-of-day from `nonce` modelled as cyclic proxy
///   4. day             – day-of-week proxy (nonce mod 7)
///   5. sender_entropy  – Shannon-style byte entropy of sender address
///   6. receiver_entropy – Shannon-style byte entropy of receiver address
///   7. tx_size         – serialised transaction size (zk_proof + signature)
///   8. nonce_norm      – normalised nonce
///   9. proof_ratio     – ratio of zk_proof length to (signature length + 1)
#[derive(Clone)]
pub struct TransactionFeatureExtractor {
    amount_mean: f64,
    amount_m2: f64,
    fee_mean: f64,
    fee_m2: f64,
    count: u64,
}

impl TransactionFeatureExtractor {
    pub fn new() -> Self {
        TransactionFeatureExtractor {
            amount_mean: 0.0,
            amount_m2: 0.0,
            fee_mean: 0.0,
            fee_m2: 0.0,
            count: 0,
        }
    }

    /// Welford's online algorithm for updating running mean / variance.
    pub fn update_statistics(&mut self, tx: &Transaction) {
        self.count += 1;
        let n = self.count as f64;

        let amount = tx.amount as f64;
        let delta_a = amount - self.amount_mean;
        self.amount_mean += delta_a / n;
        let delta_a2 = amount - self.amount_mean;
        self.amount_m2 += delta_a * delta_a2;

        let fee = tx.fee as f64;
        let delta_f = fee - self.fee_mean;
        self.fee_mean += delta_f / n;
        let delta_f2 = fee - self.fee_mean;
        self.fee_m2 += delta_f * delta_f2;
    }

    fn amount_std(&self) -> f64 {
        if self.count < 2 {
            return 1.0;
        }
        (self.amount_m2 / (self.count as f64 - 1.0)).sqrt().max(1.0)
    }

    fn fee_std(&self) -> f64 {
        if self.count < 2 {
            return 1.0;
        }
        (self.fee_m2 / (self.count as f64 - 1.0)).sqrt().max(1.0)
    }

    /// Produce a 10-element feature vector from a single transaction.
    pub fn extract_features(&self, tx: &Transaction) -> Vec<f64> {
        let amount = tx.amount as f64;
        let fee = tx.fee as f64;

        let amount_zscore = (amount - self.amount_mean) / self.amount_std();
        let fee_zscore = (fee - self.fee_mean) / self.fee_std();
        let amount_fee_ratio = amount / (fee + 1.0);

        // Cyclic time proxies derived from nonce (deterministic stand-in)
        let hour = (tx.nonce % 24) as f64 / 24.0;
        let day = (tx.nonce % 7) as f64 / 7.0;

        let sender_entropy = byte_entropy(&tx.from);
        let receiver_entropy = byte_entropy(&tx.to);

        let tx_size = (tx.zk_proof.len() + tx.signature.len()) as f64;
        let nonce_norm = (tx.nonce as f64).ln().max(0.0) / 20.0; // soft normalisation
        let proof_ratio = tx.zk_proof.len() as f64 / (tx.signature.len() as f64 + 1.0);

        vec![
            amount_zscore,
            fee_zscore,
            amount_fee_ratio,
            hour,
            day,
            sender_entropy,
            receiver_entropy,
            tx_size,
            nonce_norm,
            proof_ratio,
        ]
    }
}

/// Shannon byte-entropy of a 32-byte address, normalised to [0, 1].
fn byte_entropy(addr: &[u8; 32]) -> f64 {
    let mut counts = [0u32; 256];
    for &b in addr.iter() {
        counts[b as usize] += 1;
    }
    let n = addr.len() as f64;
    let entropy: f64 = counts
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / n;
            -p * p.log2()
        })
        .sum();
    // Maximum entropy for 32 bytes = log2(32) ≈ 5.0
    entropy / 5.0
}

// ---------------------------------------------------------------------------
// ML Transaction Validator
// ---------------------------------------------------------------------------

/// Production transaction validator backed by the [`ProductionMLStack`].
///
/// Thread-safe via `Arc<RwLock<..>>` wrappers; designed for concurrent
/// validation across multiple Tokio tasks.
pub struct MLTransactionValidator {
    ml_stack: Arc<RwLock<ProductionMLStack>>,
    feature_extractor: Arc<RwLock<TransactionFeatureExtractor>>,
    normal_tx_buffer: Arc<RwLock<VecDeque<Vec<f64>>>>,
    anomaly_threshold: f64,
    buffer_capacity: usize,
    retrain_interval: u64,
    blocks_since_retrain: Arc<RwLock<u64>>,
}

impl MLTransactionValidator {
    /// Create a new validator with the given anomaly threshold.
    pub fn new(anomaly_threshold: f64) -> Self {
        MLTransactionValidator {
            ml_stack: Arc::new(RwLock::new(ProductionMLStack::new())),
            feature_extractor: Arc::new(RwLock::new(TransactionFeatureExtractor::new())),
            normal_tx_buffer: Arc::new(RwLock::new(VecDeque::with_capacity(10_000))),
            anomaly_threshold,
            buffer_capacity: 10_000,
            retrain_interval: 1000,
            blocks_since_retrain: Arc::new(RwLock::new(0)),
        }
    }

    /// Validate a single transaction using ML anomaly detection.
    ///
    /// Returns `(is_valid, anomaly_score, reason)`.
    pub fn validate_transaction(&self, tx: &Transaction) -> (bool, f64, String) {
        let start = Instant::now();

        // 1. Extract features
        let features = self.feature_extractor.read().extract_features(tx);

        // 2. Run ML detection
        let score = self.ml_stack.read().detect_anomaly(&features);

        let latency_us = start.elapsed().as_micros();
        log::trace!("ML inference latency: {} µs", latency_us);

        // 3. Decision
        let is_valid = score <= self.anomaly_threshold;
        let reason = if is_valid {
            "OK".to_string()
        } else {
            log::warn!(
                "TRANSACTION_REJECTED: hash={} from={} to={} amount={} score={:.3}",
                hex::encode(tx.hash()),
                hex::encode(tx.from),
                hex::encode(tx.to),
                tx.amount,
                score,
            );
            "ANOMALY_DETECTED".to_string()
        };

        (is_valid, score, reason)
    }

    /// Process a validated block: update feature statistics and buffer
    /// normal transactions.  Triggers background retraining every
    /// `retrain_interval` blocks.
    pub fn process_block(&self, block: &Block) {
        // Update running statistics for every transaction in the block
        {
            let mut extractor = self.feature_extractor.write();
            let mut buffer = self.normal_tx_buffer.write();
            for tx in &block.transactions {
                extractor.update_statistics(tx);
            }
            // Clone after stats update for feature extraction
            let ext_snapshot = extractor.clone();
            for tx in &block.transactions {
                let features = ext_snapshot.extract_features(tx);
                if buffer.len() >= self.buffer_capacity {
                    buffer.pop_front();
                }
                buffer.push_back(features);
            }
        }

        // Check if retraining is needed
        let should_retrain = {
            let mut counter = self.blocks_since_retrain.write();
            *counter += 1;
            if *counter >= self.retrain_interval {
                *counter = 0;
                true
            } else {
                false
            }
        };

        if should_retrain {
            self.retrain();
        }
    }

    /// Re-train the ML stack on the current normal-transaction buffer.
    fn retrain(&self) {
        let data: Vec<Vec<f64>> = {
            let buffer = self.normal_tx_buffer.read();
            buffer.iter().cloned().collect()
        };

        if data.len() < 50 {
            log::debug!(
                "Skipping ML retrain: only {} samples (need ≥ 50)",
                data.len()
            );
            return;
        }

        log::info!("Retraining ML stack on {} samples", data.len());
        let mut stack = self.ml_stack.write();
        stack.fit(&data);
        log::info!("ML retrain complete");
    }

    /// Manually trigger a training round (useful at node startup).
    pub fn force_train(&self, data: &[Vec<f64>]) {
        let mut stack = self.ml_stack.write();
        stack.fit(data);
    }

    /// Returns `true` when the underlying ML models are trained.
    pub fn is_trained(&self) -> bool {
        self.ml_stack.read().is_trained()
    }

    /// Current anomaly threshold.
    pub fn anomaly_threshold(&self) -> f64 {
        self.anomaly_threshold
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::Transaction;

    fn sample_tx(amount: u64, fee: u64) -> Transaction {
        Transaction {
            from: [1u8; 32],
            to: [2u8; 32],
            amount,
            fee,
            nonce: 42,
            zk_proof: vec![0u8; 256],
            signature: vec![0u8; 64],
        }
    }

    #[test]
    fn test_feature_extraction_produces_10_features() {
        let ext = TransactionFeatureExtractor::new();
        let tx = sample_tx(1_000_000, 1_000);
        let features = ext.extract_features(&tx);
        assert_eq!(features.len(), 10);
    }

    #[test]
    fn test_online_statistics_update() {
        let mut ext = TransactionFeatureExtractor::new();
        for i in 1..=100 {
            ext.update_statistics(&sample_tx(i * 1000, i * 10));
        }
        assert_eq!(ext.count, 100);
        assert!(ext.amount_std() > 1.0);
        assert!(ext.fee_std() > 1.0);
    }

    #[test]
    fn test_validator_returns_valid_for_untrained() {
        let validator = MLTransactionValidator::new(0.7);
        let tx = sample_tx(1000, 10);
        let (is_valid, score, reason) = validator.validate_transaction(&tx);
        // Untrained stack returns 0.5 which is <= 0.7
        assert!(is_valid);
        assert!((score - 0.5).abs() < 0.01);
        assert_eq!(reason, "OK");
    }

    #[test]
    fn test_force_train() {
        let validator = MLTransactionValidator::new(0.7);
        let data: Vec<Vec<f64>> = (0..200)
            .map(|_| vec![0.0, 0.0, 1.0, 0.5, 0.5, 0.8, 0.8, 320.0, 0.2, 4.0])
            .collect();
        validator.force_train(&data);
        assert!(validator.is_trained());
    }

    #[test]
    fn test_byte_entropy() {
        let uniform: [u8; 32] = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
            23, 24, 25, 26, 27, 28, 29, 30, 31,
        ];
        let e = byte_entropy(&uniform);
        // High entropy for all unique bytes
        assert!(e > 0.8, "expected high entropy, got {}", e);

        let constant: [u8; 32] = [0u8; 32];
        let e = byte_entropy(&constant);
        // Zero entropy for constant array
        assert!(e < 0.01, "expected ~0 entropy, got {}", e);
    }
}
