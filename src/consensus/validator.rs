// src/consensus/validator.rs
// Consensus-level block validator integrating ML anomaly detection.

use std::sync::Arc;

use crate::block::Block;
use crate::transaction::Transaction;
use crate::validation::MLTransactionValidator;

// ---------------------------------------------------------------------------
// Validation errors
// ---------------------------------------------------------------------------

/// Errors produced during consensus-level block validation.
#[derive(Debug)]
pub enum ValidationError {
    /// A transaction was rejected by the ML anomaly detector.
    MLRejection {
        tx_hash: [u8; 32],
        anomaly_score: f64,
        reason: String,
    },
    /// Generic validation failure.
    Other(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::MLRejection {
                tx_hash,
                anomaly_score,
                reason,
            } => write!(
                f,
                "ML rejection: tx={} score={:.3} reason={}",
                hex::encode(tx_hash),
                anomaly_score,
                reason
            ),
            ValidationError::Other(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}

// ---------------------------------------------------------------------------
// ConsensusValidator
// ---------------------------------------------------------------------------

/// Wraps the [`MLTransactionValidator`] for use in the consensus pipeline.
///
/// Typical usage:
/// ```ignore
/// let cv = ConsensusValidator::new(ml_validator);
/// cv.validate_block(&block)?;
/// ```
pub struct ConsensusValidator {
    ml_validator: Arc<MLTransactionValidator>,
}

impl ConsensusValidator {
    pub fn new(ml_validator: Arc<MLTransactionValidator>) -> Self {
        ConsensusValidator { ml_validator }
    }

    /// Run ML anomaly detection on every transaction in the block.
    ///
    /// Returns `Ok(())` when all transactions pass, or the first
    /// [`ValidationError::MLRejection`] encountered.
    pub fn validate_block_ml(&self, block: &Block) -> Result<(), ValidationError> {
        for tx in &block.transactions {
            let (is_valid, score, reason) = self.ml_validator.validate_transaction(tx);
            if !is_valid {
                return Err(ValidationError::MLRejection {
                    tx_hash: tx.hash(),
                    anomaly_score: score,
                    reason,
                });
            }
        }

        // Feed validated block into the ML models for incremental learning
        self.ml_validator.process_block(block);

        Ok(())
    }

    /// Convenience: validate a single transaction (delegates to ML validator).
    pub fn validate_transaction(&self, tx: &Transaction) -> Result<(), ValidationError> {
        let (is_valid, score, reason) = self.ml_validator.validate_transaction(tx);
        if !is_valid {
            return Err(ValidationError::MLRejection {
                tx_hash: tx.hash(),
                anomaly_score: score,
                reason,
            });
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::Transaction;
    use crate::block::Block;

    fn sample_tx() -> Transaction {
        Transaction {
            from: [1u8; 32],
            to: [2u8; 32],
            amount: 1_000,
            fee: 10,
            nonce: 1,
            zk_proof: vec![0u8; 256],
            signature: vec![0u8; 64],
        }
    }

    fn sample_block() -> Block {
        Block {
            parent: [0u8; 32],
            slot: 1,
            timestamp: 1800, // TARGET_TIME after genesis
            miner: [3u8; 32],
            transactions: vec![sample_tx()],
            vdf_proof: [0u8; 32],
            zk_proof: vec![0u8; 128],
            nonce: 0,
        }
    }

    #[test]
    fn test_consensus_validator_passes_untrained() {
        let ml = Arc::new(MLTransactionValidator::new(0.7));
        let cv = ConsensusValidator::new(ml);
        let block = sample_block();
        // Untrained model yields 0.5 < 0.7 → pass
        assert!(cv.validate_block_ml(&block).is_ok());
    }

    #[test]
    fn test_consensus_validator_single_tx() {
        let ml = Arc::new(MLTransactionValidator::new(0.7));
        let cv = ConsensusValidator::new(ml);
        assert!(cv.validate_transaction(&sample_tx()).is_ok());
    }

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError::MLRejection {
            tx_hash: [0xABu8; 32],
            anomaly_score: 0.85,
            reason: "ANOMALY_DETECTED".into(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("ML rejection"));
        assert!(msg.contains("0.850"));
    }
}
