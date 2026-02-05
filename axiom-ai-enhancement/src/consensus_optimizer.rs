// Adaptive Consensus Parameter Optimizer
// Uses PID control theory and statistical analysis (no ML dependencies)

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

const HISTORY_SIZE: usize = 1000;
const TARGET_BLOCK_TIME: u64 = 1800; // 30 minutes in seconds
const ADJUSTMENT_INTERVAL: usize = 144; // ~3 days at 30min blocks

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub hashrate: f64,
    pub block_time: u64,
    pub peer_count: u32,
    pub mempool_size: usize,
    pub avg_tx_fee: f64,
    pub chain_height: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub parameter: String,
    pub current_value: f64,
    pub suggested_value: f64,
    pub confidence: f32,
    pub rationale: String,
    pub expected_improvement: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusParameters {
    pub difficulty: u64,
    pub vdf_iterations: u64,
    pub block_time_target: u64,
    pub max_block_size: u64,
    pub min_gas_price: u64,
}

impl Default for ConsensusParameters {
    fn default() -> Self {
        Self {
            difficulty: 1_000_000,
            vdf_iterations: 1000,
            block_time_target: TARGET_BLOCK_TIME,
            max_block_size: 1_000_000,
            min_gas_price: 1000,
        }
    }
}

pub struct AdaptiveConsensusOptimizer {
    metrics_history: VecDeque<NetworkMetrics>,
    parameter_history: VecDeque<ConsensusParameters>,
    current_params: ConsensusParameters,
    pid_difficulty: PIDController,
    pid_gas_price: PIDController,
}

/// PID Controller for smooth parameter adjustments
struct PIDController {
    kp: f64, // Proportional gain
    ki: f64, // Integral gain
    kd: f64, // Derivative gain
    integral: f64,
    prev_error: f64,
}

impl PIDController {
    fn new(kp: f64, ki: f64, kd: f64) -> Self {
        Self {
            kp,
            ki,
            kd,
            integral: 0.0,
            prev_error: 0.0,
        }
    }

    fn update(&mut self, error: f64, dt: f64) -> f64 {
        self.integral += error * dt;
        let derivative = (error - self.prev_error) / dt;
        self.prev_error = error;

        self.kp * error + self.ki * self.integral + self.kd * derivative
    }

    fn reset(&mut self) {
        self.integral = 0.0;
        self.prev_error = 0.0;
    }
}

impl AdaptiveConsensusOptimizer {
    pub fn new() -> Self {
        Self {
            metrics_history: VecDeque::with_capacity(HISTORY_SIZE),
            parameter_history: VecDeque::with_capacity(HISTORY_SIZE),
            current_params: ConsensusParameters::default(),
            pid_difficulty: PIDController::new(0.5, 0.1, 0.05),
            pid_gas_price: PIDController::new(0.3, 0.05, 0.02),
        }
    }

    pub fn record_metrics(&mut self, metrics: NetworkMetrics) {
        self.metrics_history.push_back(metrics);
        if self.metrics_history.len() > HISTORY_SIZE {
            self.metrics_history.pop_front();
        }
    }

    /// Suggest difficulty adjustment using PID control
    pub fn suggest_difficulty_adjustment(&mut self) -> Result<OptimizationSuggestion, String> {
        if self.metrics_history.len() < 10 {
            return Err("Insufficient metrics history".to_string());
        }

        let recent_metrics: Vec<&NetworkMetrics> = self
            .metrics_history
            .iter()
            .rev()
            .take(ADJUSTMENT_INTERVAL)
            .collect();

        if recent_metrics.is_empty() {
            return Err("No recent metrics".to_string());
        }

        // Calculate average block time
        let avg_block_time = recent_metrics.iter().map(|m| m.block_time).sum::<u64>() as f64
            / recent_metrics.len() as f64;

        let target = self.current_params.block_time_target as f64;
        let error = (avg_block_time - target) / target; // Normalized error

        // Calculate hashrate trend
        let hashrate_trend = self.calculate_hashrate_trend(&recent_metrics);

        // Use PID controller for smooth adjustment
        let pid_output = self.pid_difficulty.update(error, 1.0);

        // Calculate adjustment factor (limit to ±50% per adjustment)
        let adjustment = (1.0 + pid_output).max(0.5).min(1.5);

        // Apply hashrate compensation
        let hashrate_adjustment = 1.0 + (hashrate_trend * 0.1);

        let current_difficulty = self.current_params.difficulty as f64;
        let suggested_difficulty =
            (current_difficulty * adjustment * hashrate_adjustment).max(1000.0);

        // Calculate expected improvement
        let expected_improvement =
            ((target - avg_block_time).abs() / target * 100.0).min(100.0) as f32;

        // Calculate confidence based on data stability
        let confidence = self.calculate_confidence(&recent_metrics);

        Ok(OptimizationSuggestion {
            parameter: "difficulty".to_string(),
            current_value: current_difficulty,
            suggested_value: suggested_difficulty,
            confidence,
            rationale: format!(
                "Avg block time: {:.0}s (target: {:.0}s), Hashrate trend: {:.2}%, PID output: {:.3}",
                avg_block_time,
                target,
                hashrate_trend * 100.0,
                pid_output
            ),
            expected_improvement,
        })
    }

    /// Suggest VDF adjustment based on security requirements
    pub fn suggest_vdf_adjustment(&self) -> Result<OptimizationSuggestion, String> {
        if self.metrics_history.len() < 10 {
            return Err("Insufficient metrics history".to_string());
        }

        let recent_metrics: Vec<&NetworkMetrics> =
            self.metrics_history.iter().rev().take(50).collect();

        // VDF iterations should scale with hashrate for security
        let avg_hashrate =
            recent_metrics.iter().map(|m| m.hashrate).sum::<f64>() / recent_metrics.len() as f64;

        // Baseline: 1000 iterations at 1 TH/s
        let baseline_hashrate = 1e12;
        let hashrate_ratio = (avg_hashrate / baseline_hashrate).ln();

        let current_vdf = self.current_params.vdf_iterations as f64;
        let suggested_vdf = (1000.0 * (1.0 + hashrate_ratio * 0.1))
            .max(500.0)
            .min(5000.0);

        let confidence = if recent_metrics.len() >= 50 {
            0.8
        } else {
            0.6
        };

        Ok(OptimizationSuggestion {
            parameter: "vdf_iterations".to_string(),
            current_value: current_vdf,
            suggested_value: suggested_vdf,
            confidence,
            rationale: format!(
                "VDF adjusted for hashrate: {:.2} TH/s",
                avg_hashrate / 1e12
            ),
            expected_improvement: 5.0,
        })
    }

    /// Suggest gas price adjustment based on mempool congestion
    pub fn suggest_gas_price_adjustment(&mut self) -> Result<OptimizationSuggestion, String> {
        if self.metrics_history.len() < 10 {
            return Err("Insufficient metrics history".to_string());
        }

        let recent_metrics: Vec<&NetworkMetrics> =
            self.metrics_history.iter().rev().take(50).collect();

        // Calculate average mempool size
        let avg_mempool = recent_metrics.iter().map(|m| m.mempool_size).sum::<usize>() as f64
            / recent_metrics.len() as f64;

        // Calculate average transaction fee
        let avg_tx_fee =
            recent_metrics.iter().map(|m| m.avg_tx_fee).sum::<f64>() / recent_metrics.len() as f64;

        // Target mempool size: 500 transactions
        let target_mempool = 500.0;
        let error = (avg_mempool - target_mempool) / target_mempool;

        // Use PID controller
        let pid_output = self.pid_gas_price.update(error, 1.0);

        // Calculate congestion factor
        let congestion_factor = (avg_mempool / target_mempool).max(0.5).min(2.0);

        let current_min_gas = self.current_params.min_gas_price as f64;
        let suggested_gas =
            (current_min_gas * congestion_factor * (1.0 + pid_output * 0.2)).max(100.0);

        let confidence = self.calculate_confidence(&recent_metrics);

        Ok(OptimizationSuggestion {
            parameter: "min_gas_price".to_string(),
            current_value: current_min_gas,
            suggested_value: suggested_gas,
            confidence,
            rationale: format!(
                "Mempool: {:.0} txs (target: {:.0}), Avg fee: {:.6}, Congestion: {:.2}x",
                avg_mempool, target_mempool, avg_tx_fee, congestion_factor
            ),
            expected_improvement: 10.0,
        })
    }

    /// Generate all optimization suggestions
    pub fn suggest_all_optimizations(&mut self) -> Result<Vec<OptimizationSuggestion>, String> {
        let mut suggestions = Vec::new();

        if let Ok(diff_suggestion) = self.suggest_difficulty_adjustment() {
            if self.verify_safety_bounds(&diff_suggestion) {
                suggestions.push(diff_suggestion);
            }
        }

        if let Ok(vdf_suggestion) = self.suggest_vdf_adjustment() {
            if self.verify_safety_bounds(&vdf_suggestion) {
                suggestions.push(vdf_suggestion);
            }
        }

        if let Ok(gas_suggestion) = self.suggest_gas_price_adjustment() {
            if self.verify_safety_bounds(&gas_suggestion) {
                suggestions.push(gas_suggestion);
            }
        }

        if suggestions.is_empty() {
            Err("No safe optimizations available".to_string())
        } else {
            Ok(suggestions)
        }
    }

    /// Calculate hashrate trend (positive = increasing)
    fn calculate_hashrate_trend(&self, metrics: &[&NetworkMetrics]) -> f64 {
        if metrics.len() < 2 {
            return 0.0;
        }

        let recent = metrics.first().unwrap().hashrate;
        let older = metrics.last().unwrap().hashrate;

        if older == 0.0 {
            return 0.0;
        }

        (recent - older) / older
    }

    /// Calculate confidence based on data stability
    fn calculate_confidence(&self, metrics: &[&NetworkMetrics]) -> f32 {
        if metrics.len() < 10 {
            return 0.5;
        }

        // Calculate variance in block times
        let block_times: Vec<f64> = metrics.iter().map(|m| m.block_time as f64).collect();
        let mean = block_times.iter().sum::<f64>() / block_times.len() as f64;
        let variance = block_times
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / block_times.len() as f64;

        let std_dev = variance.sqrt();
        let coefficient_of_variation = std_dev / mean;

        // Low CoV = high confidence
        (1.0 / (1.0 + coefficient_of_variation)).max(0.5).min(1.0) as f32
    }

    /// Verify suggestion is within safety bounds
    fn verify_safety_bounds(&self, suggestion: &OptimizationSuggestion) -> bool {
        let ratio = suggestion.suggested_value / suggestion.current_value;

        match suggestion.parameter.as_str() {
            "difficulty" => ratio >= 0.5 && ratio <= 2.0, // Max 2x change
            "vdf_iterations" => {
                suggestion.suggested_value >= 500.0 && suggestion.suggested_value <= 5000.0
            }
            "min_gas_price" => {
                suggestion.suggested_value >= 100.0 && suggestion.suggested_value <= 100000.0
            }
            _ => false,
        }
    }

    /// Apply a suggestion to current parameters
    pub fn apply_suggestion(&mut self, suggestion: &OptimizationSuggestion) {
        match suggestion.parameter.as_str() {
            "difficulty" => {
                self.current_params.difficulty = suggestion.suggested_value as u64;
            }
            "vdf_iterations" => {
                self.current_params.vdf_iterations = suggestion.suggested_value as u64;
            }
            "min_gas_price" => {
                self.current_params.min_gas_price = suggestion.suggested_value as u64;
            }
            _ => {}
        }

        // Record parameter change
        self.parameter_history.push_back(self.current_params.clone());
        if self.parameter_history.len() > HISTORY_SIZE {
            self.parameter_history.pop_front();
        }
    }

    /// Get current parameters
    pub fn get_current_parameters(&self) -> ConsensusParameters {
        self.current_params.clone()
    }

    /// Reset PID controllers (useful after manual intervention)
    pub fn reset_controllers(&mut self) {
        self.pid_difficulty.reset();
        self.pid_gas_price.reset();
    }
}
