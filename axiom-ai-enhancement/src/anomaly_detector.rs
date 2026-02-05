// Production Anomaly Detection - Statistical Methods
// No external ML dependencies required - uses proven statistical techniques

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use parking_lot::RwLock;

const HISTORY_SIZE: usize = 1000;
const ZSCORE_THRESHOLD: f64 = 3.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionFeatures {
    pub amount: f64,
    pub gas_fee: f64,
    pub zk_proof_size: u64,
    pub sender_tx_count: u64,
    pub recipient_tx_count: u64,
    pub time_since_last_tx: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyScore {
    pub score: f64,
    pub is_anomaly: bool,
    pub confidence: f64,
    pub risk_factors: Vec<String>,
    pub severity: AnomalySeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct AnomalyDetector {
    history: Arc<RwLock<VecDeque<TransactionFeatures>>>,
    stats: Arc<RwLock<Statistics>>,
}

#[derive(Clone)]
struct Statistics {
    amount_mean: f64,
    amount_std: f64,
    gas_mean: f64,
    gas_std: f64,
    proof_size_mean: f64,
    proof_size_std: f64,
}

impl Default for Statistics {
    fn default() -> Self {
        Self {
            amount_mean: 100.0,
            amount_std: 50.0,
            gas_mean: 0.001,
            gas_std: 0.0005,
            proof_size_mean: 256.0,
            proof_size_std: 10.0,
        }
    }
}

impl AnomalyDetector {
    pub fn new() -> Self {
        Self {
            history: Arc::new(RwLock::new(VecDeque::with_capacity(HISTORY_SIZE))),
            stats: Arc::new(RwLock::new(Statistics::default())),
        }
    }

    /// Check transaction for anomalies using multiple statistical methods
    pub fn check_transaction(&self, features: &TransactionFeatures) -> AnomalyScore {
        let mut risk_factors = Vec::new();
        let mut scores = Vec::new();

        // 1. Z-Score Analysis (univariate outlier detection)
        let zscore_result = self.zscore_analysis(features);
        scores.push(zscore_result.0);
        if zscore_result.1 {
            risk_factors.extend(zscore_result.2);
        }

        // 2. Modified Z-Score (robust to outliers)
        let modified_zscore = self.modified_zscore_analysis(features);
        scores.push(modified_zscore.0);
        if modified_zscore.1 {
            risk_factors.extend(modified_zscore.2);
        }

        // 3. IQR Method (interquartile range)
        let iqr_result = self.iqr_analysis(features);
        scores.push(iqr_result.0);
        if iqr_result.1 {
            risk_factors.extend(iqr_result.2);
        }

        // 4. Behavioral Pattern Analysis
        let pattern_result = self.pattern_analysis(features);
        scores.push(pattern_result.0);
        if pattern_result.1 {
            risk_factors.extend(pattern_result.2);
        }

        // 5. Time-based Analysis
        let time_result = self.time_analysis(features);
        scores.push(time_result.0);
        if time_result.1 {
            risk_factors.extend(time_result.2);
        }

        // Aggregate scores (higher = more anomalous)
        let combined_score = scores.iter().sum::<f64>() / scores.len() as f64;
        let is_anomaly = !risk_factors.is_empty();
        let confidence = self.calculate_confidence(&scores);
        let severity = self.calculate_severity(combined_score, &risk_factors);

        // Update history and statistics
        self.update_history(features.clone());

        AnomalyScore {
            score: combined_score,
            is_anomaly,
            confidence,
            risk_factors,
            severity,
        }
    }

    /// Z-Score anomaly detection
    fn zscore_analysis(&self, features: &TransactionFeatures) -> (f64, bool, Vec<String>) {
        let stats = self.stats.read();
        let mut risk_factors = Vec::new();
        let mut anomaly_count = 0;
        let mut total_zscore = 0.0;

        // Amount Z-score
        let amount_z = (features.amount - stats.amount_mean).abs() / stats.amount_std;
        if amount_z > ZSCORE_THRESHOLD {
            risk_factors.push(format!(
                "Unusual transaction amount: {:.2}σ from mean",
                amount_z
            ));
            anomaly_count += 1;
        }
        total_zscore += amount_z;

        // Gas fee Z-score
        let gas_z = (features.gas_fee - stats.gas_mean).abs() / stats.gas_std;
        if gas_z > ZSCORE_THRESHOLD {
            risk_factors.push(format!("Unusual gas fee: {:.2}σ from mean", gas_z));
            anomaly_count += 1;
        }
        total_zscore += gas_z;

        // Proof size Z-score
        let proof_z = (features.zk_proof_size as f64 - stats.proof_size_mean).abs()
            / stats.proof_size_std;
        if proof_z > ZSCORE_THRESHOLD {
            risk_factors.push(format!("Unusual ZK-proof size: {:.2}σ from mean", proof_z));
            anomaly_count += 1;
        }
        total_zscore += proof_z;

        let score = (total_zscore / 3.0).min(10.0) / 10.0;
        (score, anomaly_count > 0, risk_factors)
    }

    /// Modified Z-Score using median absolute deviation (robust to outliers)
    fn modified_zscore_analysis(
        &self,
        features: &TransactionFeatures,
    ) -> (f64, bool, Vec<String>) {
        let history = self.history.read();
        if history.len() < 30 {
            return (0.0, false, vec![]);
        }

        let mut risk_factors = Vec::new();
        let mut anomaly_count = 0;

        // Calculate median and MAD for amount
        let amounts: Vec<f64> = history.iter().map(|f| f.amount).collect();
        let (median, mad) = self.calculate_median_mad(&amounts);
        let modified_z = 0.6745 * (features.amount - median).abs() / mad;

        if modified_z > 3.5 {
            risk_factors.push(format!(
                "Extreme value detected (Modified Z-Score: {:.2})",
                modified_z
            ));
            anomaly_count += 1;
        }

        let score = (modified_z / 3.5).min(1.0);
        (score, anomaly_count > 0, risk_factors)
    }

    /// IQR (Interquartile Range) method
    fn iqr_analysis(&self, features: &TransactionFeatures) -> (f64, bool, Vec<String>) {
        let history = self.history.read();
        if history.len() < 30 {
            return (0.0, false, vec![]);
        }

        let mut risk_factors = Vec::new();
        let mut anomaly_score = 0.0;

        // Amount IQR check
        let amounts: Vec<f64> = history.iter().map(|f| f.amount).collect();
        if let Some((lower, upper)) = self.calculate_iqr_bounds(&amounts) {
            if features.amount < lower || features.amount > upper {
                risk_factors.push(format!(
                    "Amount outside IQR bounds [{:.2}, {:.2}]",
                    lower, upper
                ));
                anomaly_score += 0.5;
            }
        }

        // Gas fee IQR check
        let gas_fees: Vec<f64> = history.iter().map(|f| f.gas_fee).collect();
        if let Some((lower, upper)) = self.calculate_iqr_bounds(&gas_fees) {
            if features.gas_fee < lower || features.gas_fee > upper {
                risk_factors.push(format!(
                    "Gas fee outside IQR bounds [{:.6}, {:.6}]",
                    lower, upper
                ));
                anomaly_score += 0.5;
            }
        }

        (anomaly_score.min(1.0), !risk_factors.is_empty(), risk_factors)
    }

    /// Behavioral pattern analysis
    fn pattern_analysis(&self, features: &TransactionFeatures) -> (f64, bool, Vec<String>) {
        let mut risk_factors = Vec::new();
        let mut anomaly_score = 0.0;

        // Check for suspicious patterns
        
        // Pattern 1: Very large transaction with very low gas (possible MEV/front-running)
        if features.amount > 10000.0 && features.gas_fee < 0.0001 {
            risk_factors.push("Large transaction with unusually low gas fee (MEV risk)".to_string());
            anomaly_score += 0.8;
        }

        // Pattern 2: First-time sender/recipient with large transaction
        if features.sender_tx_count <= 1 && features.amount > 1000.0 {
            risk_factors.push("New sender with large transaction (possible money laundering)".to_string());
            anomaly_score += 0.7;
        }

        if features.recipient_tx_count == 0 && features.amount > 5000.0 {
            risk_factors.push("New recipient receiving large transaction".to_string());
            anomaly_score += 0.6;
        }

        // Pattern 3: Extremely high gas (possible spam/DoS attack)
        if features.gas_fee > 0.1 {
            risk_factors.push("Extremely high gas fee (possible attack)".to_string());
            anomaly_score += 0.9;
        }

        // Pattern 4: Unusual ZK-proof size
        if features.zk_proof_size < 128 || features.zk_proof_size > 1024 {
            risk_factors.push(format!(
                "Unusual ZK-proof size: {} bytes",
                features.zk_proof_size
            ));
            anomaly_score += 0.5;
        }

        (anomaly_score.min(1.0), !risk_factors.is_empty(), risk_factors)
    }

    /// Time-based anomaly detection
    fn time_analysis(&self, features: &TransactionFeatures) -> (f64, bool, Vec<String>) {
        let mut risk_factors = Vec::new();
        let mut anomaly_score = 0.0;

        // Rapid-fire transactions (less than 10 seconds between)
        if features.time_since_last_tx < 10 {
            risk_factors.push(format!(
                "Rapid transaction sequence: {}s since last tx",
                features.time_since_last_tx
            ));
            anomaly_score += 0.7;
        }

        // Extremely delayed transaction (over 24 hours)
        if features.time_since_last_tx > 86400 {
            risk_factors.push("Long dormancy before transaction".to_string());
            anomaly_score += 0.3;
        }

        (anomaly_score.min(1.0), !risk_factors.is_empty(), risk_factors)
    }

    /// Calculate median and Median Absolute Deviation
    fn calculate_median_mad(&self, data: &[f64]) -> (f64, f64) {
        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let median = if sorted.len() % 2 == 0 {
            (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };

        let deviations: Vec<f64> = sorted.iter().map(|x| (x - median).abs()).collect();
        let mut sorted_dev = deviations.clone();
        sorted_dev.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mad = if sorted_dev.len() % 2 == 0 {
            (sorted_dev[sorted_dev.len() / 2 - 1] + sorted_dev[sorted_dev.len() / 2]) / 2.0
        } else {
            sorted_dev[sorted_dev.len() / 2]
        };

        (median, mad.max(0.0001)) // Avoid division by zero
    }

    /// Calculate IQR bounds
    fn calculate_iqr_bounds(&self, data: &[f64]) -> Option<(f64, f64)> {
        if data.len() < 4 {
            return None;
        }

        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1_idx = sorted.len() / 4;
        let q3_idx = (sorted.len() * 3) / 4;

        let q1 = sorted[q1_idx];
        let q3 = sorted[q3_idx];
        let iqr = q3 - q1;

        let lower = q1 - 1.5 * iqr;
        let upper = q3 + 1.5 * iqr;

        Some((lower, upper))
    }

    /// Calculate confidence based on variance in scores
    fn calculate_confidence(&self, scores: &[f64]) -> f64 {
        if scores.is_empty() {
            return 0.5;
        }

        let mean = scores.iter().sum::<f64>() / scores.len() as f64;
        let variance = scores
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / scores.len() as f64;

        // Low variance = high confidence
        (1.0 / (1.0 + variance)).max(0.5).min(1.0)
    }

    /// Calculate severity based on score and risk factors
    fn calculate_severity(&self, score: f64, risk_factors: &[String]) -> AnomalySeverity {
        let factor_count = risk_factors.len();

        if score > 0.8 || factor_count >= 4 {
            AnomalySeverity::Critical
        } else if score > 0.6 || factor_count >= 3 {
            AnomalySeverity::High
        } else if score > 0.4 || factor_count >= 2 {
            AnomalySeverity::Medium
        } else {
            AnomalySeverity::Low
        }
    }

    /// Update transaction history and recalculate statistics
    fn update_history(&self, features: TransactionFeatures) {
        let mut history = self.history.write();
        history.push_back(features);

        if history.len() > HISTORY_SIZE {
            history.pop_front();
        }

        // Recalculate statistics if we have enough data
        if history.len() >= 100 {
            let amounts: Vec<f64> = history.iter().map(|f| f.amount).collect();
            let gas_fees: Vec<f64> = history.iter().map(|f| f.gas_fee).collect();
            let proof_sizes: Vec<f64> = history.iter().map(|f| f.zk_proof_size as f64).collect();

            let mut stats = self.stats.write();
            stats.amount_mean = amounts.iter().sum::<f64>() / amounts.len() as f64;
            stats.gas_mean = gas_fees.iter().sum::<f64>() / gas_fees.len() as f64;
            stats.proof_size_mean = proof_sizes.iter().sum::<f64>() / proof_sizes.len() as f64;

            stats.amount_std = (amounts
                .iter()
                .map(|x| (x - stats.amount_mean).powi(2))
                .sum::<f64>()
                / amounts.len() as f64)
                .sqrt()
                .max(0.001);

            stats.gas_std = (gas_fees
                .iter()
                .map(|x| (x - stats.gas_mean).powi(2))
                .sum::<f64>()
                / gas_fees.len() as f64)
                .sqrt()
                .max(0.0001);

            stats.proof_size_std = (proof_sizes
                .iter()
                .map(|x| (x - stats.proof_size_mean).powi(2))
                .sum::<f64>()
                / proof_sizes.len() as f64)
                .sqrt()
                .max(1.0);
        }
    }

    /// Get current statistics
    pub fn get_statistics(&self) -> (f64, f64, f64, f64, f64, f64) {
        let stats = self.stats.read();
        (
            stats.amount_mean,
            stats.amount_std,
            stats.gas_mean,
            stats.gas_std,
            stats.proof_size_mean,
            stats.proof_size_std,
        )
    }

    /// Get history size
    pub fn history_size(&self) -> usize {
        self.history.read().len()
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}
