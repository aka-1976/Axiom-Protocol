// Axiom Protocol AI Enhancement Library
// Production-ready AI security and optimization

pub mod anomaly_detector;
pub mod consensus_optimizer;
pub mod contract_auditor;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub enable_anomaly_detection: bool,
    pub enable_contract_auditing: bool,
    pub enable_consensus_optimization: bool,
    pub anomaly_threshold: f64,
    pub audit_min_score: u8,
    pub optimization_confidence_min: f32,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            enable_anomaly_detection: true,
            enable_contract_auditing: true,
            enable_consensus_optimization: true,
            anomaly_threshold: 0.7,
            audit_min_score: 70,
            optimization_confidence_min: 0.7,
        }
    }
}

/// Main AI Oracle integrating all components
pub struct AIOracle {
    config: AIConfig,
    anomaly_detector: Option<Arc<RwLock<anomaly_detector::AnomalyDetector>>>,
    contract_auditor: Option<Arc<RwLock<contract_auditor::ContractAuditor>>>,
    consensus_optimizer: Option<Arc<RwLock<consensus_optimizer::AdaptiveConsensusOptimizer>>>,
}

impl AIOracle {
    pub fn new(config: AIConfig) -> Self {
        let mut oracle = Self {
            config: config.clone(),
            anomaly_detector: None,
            contract_auditor: None,
            consensus_optimizer: None,
        };

        oracle.initialize_modules();
        oracle
    }

    fn initialize_modules(&mut self) {
        if self.config.enable_anomaly_detection {
            let detector = anomaly_detector::AnomalyDetector::new();
            self.anomaly_detector = Some(Arc::new(RwLock::new(detector)));
        }

        if self.config.enable_contract_auditing {
            let auditor = contract_auditor::ContractAuditor::new();
            self.contract_auditor = Some(Arc::new(RwLock::new(auditor)));
        }

        if self.config.enable_consensus_optimization {
            let optimizer = consensus_optimizer::AdaptiveConsensusOptimizer::new();
            self.consensus_optimizer = Some(Arc::new(RwLock::new(optimizer)));
        }
    }

    /// Check transaction for anomalies
    pub fn check_transaction_anomaly(
        &self,
        features: &anomaly_detector::TransactionFeatures,
    ) -> Result<anomaly_detector::AnomalyScore, String> {
        if let Some(detector) = &self.anomaly_detector {
            let score = detector.read().check_transaction(features);

            // Apply threshold check
            if score.score > self.config.anomaly_threshold && score.is_anomaly {
                return Ok(score);
            }

            Ok(score)
        } else {
            Err("Anomaly detector not enabled".to_string())
        }
    }

    /// Audit smart contract bytecode
    pub fn audit_contract(
        &self,
        bytecode: &[u8],
    ) -> Result<contract_auditor::AuditReport, String> {
        if let Some(auditor) = &self.contract_auditor {
            let report = auditor.read().audit_contract(bytecode);

            // Check if report meets minimum score
            if report.overall_score < self.config.audit_min_score {
                return Err(format!(
                    "Contract failed security audit with score {}",
                    report.overall_score
                ));
            }

            Ok(report)
        } else {
            Err("Contract auditor not enabled".to_string())
        }
    }

    /// Record network metrics for consensus optimization
    pub fn record_network_metrics(
        &self,
        metrics: consensus_optimizer::NetworkMetrics,
    ) -> Result<(), String> {
        if let Some(optimizer) = &self.consensus_optimizer {
            optimizer.write().record_metrics(metrics);
            Ok(())
        } else {
            Err("Consensus optimizer not enabled".to_string())
        }
    }

    /// Get consensus parameter suggestions
    pub fn get_optimization_suggestions(
        &self,
    ) -> Result<Vec<consensus_optimizer::OptimizationSuggestion>, String> {
        if let Some(optimizer) = &self.consensus_optimizer {
            let suggestions = optimizer.write().suggest_all_optimizations()?;

            // Filter by confidence threshold
            let filtered: Vec<_> = suggestions
                .into_iter()
                .filter(|s| s.confidence >= self.config.optimization_confidence_min)
                .collect();

            Ok(filtered)
        } else {
            Err("Consensus optimizer not enabled".to_string())
        }
    }

    /// Apply consensus optimization suggestion
    pub fn apply_optimization(
        &self,
        suggestion: &consensus_optimizer::OptimizationSuggestion,
    ) -> Result<(), String> {
        if let Some(optimizer) = &self.consensus_optimizer {
            optimizer.write().apply_suggestion(suggestion);
            Ok(())
        } else {
            Err("Consensus optimizer not enabled".to_string())
        }
    }

    /// Get current consensus parameters
    pub fn get_consensus_parameters(
        &self,
    ) -> Result<consensus_optimizer::ConsensusParameters, String> {
        if let Some(optimizer) = &self.consensus_optimizer {
            Ok(optimizer.read().get_current_parameters())
        } else {
            Err("Consensus optimizer not enabled".to_string())
        }
    }

    /// Get anomaly detection statistics
    pub fn get_anomaly_statistics(&self) -> Result<(f64, f64, f64, f64, f64, f64), String> {
        if let Some(detector) = &self.anomaly_detector {
            Ok(detector.read().get_statistics())
        } else {
            Err("Anomaly detector not enabled".to_string())
        }
    }

    /// Reset consensus optimization controllers
    pub fn reset_consensus_controllers(&self) -> Result<(), String> {
        if let Some(optimizer) = &self.consensus_optimizer {
            optimizer.write().reset_controllers();
            Ok(())
        } else {
            Err("Consensus optimizer not enabled".to_string())
        }
    }

    /// Get configuration
    pub fn get_config(&self) -> &AIConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: AIConfig) {
        self.config = config;
        self.initialize_modules();
    }
}

impl Default for AIOracle {
    fn default() -> Self {
        Self::new(AIConfig::default())
    }
}

// Re-export commonly used types
pub use anomaly_detector::{AnomalyScore, AnomalySeverity, TransactionFeatures};
pub use consensus_optimizer::{ConsensusParameters, NetworkMetrics, OptimizationSuggestion};
pub use contract_auditor::{AuditReport, Vulnerability, VulnerabilityType};
