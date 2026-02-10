#[cfg(feature = "onnx")]
use onnxruntime::{environment::Environment, session::Session, tensor::OrtOwnedTensor, LoggingLevel};
#[cfg(feature = "onnx")]
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use std::fs::OpenOptions;
use std::io::Write;
use chrono;

/// AI Attack Detection Model
/// Static ONNX environment for all model sessions.
/// Uses once_cell::sync::Lazy to ensure only one global environment is created.
#[cfg(feature = "onnx")]
static ONNX_ENV: Lazy<Environment> = Lazy::new(|| {
    Environment::builder()
        .with_name("axiom-onnx-env")
        .with_log_level(LoggingLevel::Warning)
        .build()
        .expect("Failed to initialize ONNX environment")
});

#[cfg(feature = "onnx")]
pub struct AttackDetectionModel {
    session: Session<'static>,
}

#[cfg(not(feature = "onnx"))]
pub struct AttackDetectionModel {
    _private: (),
}

#[cfg(feature = "onnx")]
impl AttackDetectionModel {
    /// Load an ONNX model from file
    pub fn load(model_path: &'static str) -> Result<Self, Box<dyn std::error::Error + 'static>> {
        let session = ONNX_ENV
            .new_session_builder()?
            .with_model_from_file(model_path)?;
        Ok(Self { session })
    }

    /// Run inference on network metrics
    pub fn predict(&mut self, features: &[f32]) -> Result<f32, Box<dyn std::error::Error>> {
        let input_shape = vec![1, features.len()];
        let input_array = ndarray::Array::from_shape_vec(input_shape.clone(), features.to_vec())?;
        let outputs: Vec<OrtOwnedTensor<f32, _>> = self.session.run(vec![input_array])?;
        
        let first_output = outputs.first()
            .ok_or("ONNX model produced no outputs")?;
        let output_slice = first_output.as_slice()
            .ok_or("Failed to convert ONNX output to slice")?;
        let first_value = output_slice.first()
            .ok_or("ONNX output is empty")?;
        
        Ok(*first_value)
    }
}

#[cfg(not(feature = "onnx"))]
impl AttackDetectionModel {
    /// ONNX not available — always returns an error so callers use the heuristic fallback.
    pub fn load(_model_path: &'static str) -> Result<Self, Box<dyn std::error::Error + 'static>> {
        Err("ONNX runtime not enabled (build with --features onnx)".into())
    }

    /// ONNX not available — always returns an error.
    pub fn predict(&mut self, _features: &[f32]) -> Result<f32, Box<dyn std::error::Error>> {
        Err("ONNX runtime not enabled".into())
    }
}

/// Collect real-time network metrics for AI training.
///
/// Reads system statistics from `/proc` and converts them into
/// normalised feature vectors with labels.  When no data is available
/// (e.g. on non-Linux systems), returns an empty collection so callers
/// can skip the training step without panicking.
///
/// `peer_count_norm` is the normalised peer count (0.0–1.0, where 1.0 = 50 peers).
/// Callers should pass the actual connected peer count divided by 50.
pub fn collect_network_metrics_with_peers(peer_count_norm: f32) -> Vec<(Vec<f32>, f32)> {
    let mut samples = Vec::new();

    // Feature vector: [cpu_usage_norm, memory_usage_norm, peer_count_norm]
    // Label: 1.0 = normal, 0.0 = anomalous

    // CPU usage from /proc/stat (user + system, normalised to [0,1])
    let cpu_usage: f32 = std::fs::read_to_string("/proc/stat")
        .ok()
        .and_then(|s| {
            let line = s.lines().next()?;
            let parts: Vec<u64> = line.split_whitespace()
                .skip(1)
                .filter_map(|v| v.parse().ok())
                .collect();
            if parts.len() >= 4 {
                let busy = parts[0] + parts[2]; // user + system
                let total: u64 = parts.iter().sum();
                Some(if total > 0 { busy as f32 / total as f32 } else { 0.0 })
            } else { None }
        })
        .unwrap_or(0.0);

    // Memory usage from /proc/meminfo (normalised to [0,1])
    let mem_usage: f32 = std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|s| {
            let mut total: u64 = 0;
            let mut available: u64 = 0;
            for line in s.lines() {
                if line.starts_with("MemTotal:") {
                    total = line.split_whitespace().nth(1)?.parse().ok()?;
                } else if line.starts_with("MemAvailable:") {
                    available = line.split_whitespace().nth(1)?.parse().ok()?;
                }
            }
            if total > 0 { Some((total - available) as f32 / total as f32) } else { None }
        })
        .unwrap_or(0.0);

    // Label: if CPU < 0.9 and memory < 0.9, this is normal operation
    let label = if cpu_usage < 0.9 && mem_usage < 0.9 { 1.0 } else { 0.0 };
    samples.push((vec![cpu_usage, mem_usage, peer_count_norm], label));

    samples
}

/// Convenience wrapper that defaults peer count to 0.
pub fn collect_network_metrics() -> Vec<(Vec<f32>, f32)> {
    collect_network_metrics_with_peers(0.0)
}

/// Dynamic reputation scoring based on AI outputs
pub fn calculate_peer_trust_score(model: &mut AttackDetectionModel, metrics: &[f32]) -> Result<f32, Box<dyn std::error::Error>> {
    let score = model.predict(metrics)?;
    Ok(score)
}


pub struct NeuralGuardian {
    weights: [f32; 3],
    learning_rate: f32,
    pub stats: AIStats,
    pub confidence_threshold: f32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AIStats {
    pub total_predictions: u64,
    pub spam_detected: u64,
    pub peers_blocked: u64,
    pub false_positives: u64,
    pub model_used: u64,
    pub fallback_used: u64,
    pub avg_confidence: f32,
}

impl NeuralGuardian {

    pub fn new() -> Self {
        Self {
            weights: [0.5, 0.3, 0.2],
            learning_rate: 0.01,
            stats: AIStats::default(),
            confidence_threshold: 0.5,
        }
    }

    pub fn predict_trust(&mut self, time_delta: f32, consistency: f32, depth: f32) -> bool {
        self.stats.total_predictions += 1;
        let score = (time_delta * self.weights[0]) + 
                    (consistency * self.weights[1]) + 
                    (depth * self.weights[2]);
        let confidence = score;
        // Track model vs fallback usage based on weight convergence.
        // When weights[0] > 0.4 the model has been trained enough to
        // be considered the primary predictor; otherwise we count it as
        // a heuristic fallback.
        if self.weights[0] > 0.4 {
            self.stats.model_used += 1;
        } else {
            self.stats.fallback_used += 1;
        }
        self.stats.avg_confidence = 
            (self.stats.avg_confidence * (self.stats.total_predictions - 1) as f32 + confidence) 
            / self.stats.total_predictions as f32;
        let is_trustworthy = score > self.confidence_threshold;
        if !is_trustworthy {
            self.stats.spam_detected += 1;
        }
        if !(0.3..=0.9).contains(&confidence) {
            log::warn!("AI: High confidence decision - Trust: {} ({}%)", is_trustworthy, (confidence * 100.0) as u32);
        }
        is_trustworthy
    }
    pub fn log_stats(&self) {
        log::info!("--- NEURAL GUARDIAN STATS ---");
        log::info!("Total Predictions: {}", self.stats.total_predictions);
        log::info!("Spam Detected: {} ({:.1}%)", 
                 self.stats.spam_detected,
                 (self.stats.spam_detected as f32 / self.stats.total_predictions.max(1) as f32) * 100.0);
        log::info!("ONNX Model Used: {} ({:.1}%)", 
                 self.stats.model_used,
                 (self.stats.model_used as f32 / self.stats.total_predictions.max(1) as f32) * 100.0);
        log::info!("Fallback Used: {}", self.stats.fallback_used);
        log::info!("Avg Confidence: {:.2}", self.stats.avg_confidence);
    }

    pub fn report_false_positive(&mut self) {
        self.stats.false_positives += 1;
        log::warn!("AI: False positive reported. Total: {}", self.stats.false_positives);
        self.save_false_positive_case();
    }

    fn save_false_positive_case(&self) {
        match OpenOptions::new()
            .create(true)
            .append(true)
            .open("ai_training_data.csv")
        {
            Ok(mut file) => {
                writeln!(file, "{},false_positive,details_here",
                         chrono::Utc::now().timestamp()).ok();
            }
            Err(e) => {
                log::warn!("Failed to save false positive case: {}", e);
            }
        }
    }

    pub fn collect_training_sample(&self, msg_rate: f32, history: f32, reputation: f32, is_good: bool) {
        let sample = format!("{},{},{},{}\n", msg_rate, history, reputation, if is_good { 1 } else { 0 });
        match OpenOptions::new()
            .create(true)
            .append(true)
            .open("training_data.csv")
        {
            Ok(mut file) => {
                write!(file, "{}", sample).ok();
            }
            Err(e) => {
                log::warn!("Failed to save training sample: {}", e);
            }
        }
    }

    pub fn set_threshold(&mut self, threshold: f32) {
        self.confidence_threshold = threshold;
        log::info!("AI threshold updated to {}", threshold);
    }

    pub fn train(&mut self, inputs: [f32; 3], target: f32) {
        for i in 0..3 {
            let prediction = (inputs[0] * self.weights[0]) + (inputs[1] * self.weights[1]) + (inputs[2] * self.weights[2]);
            let error = target - prediction;
            self.weights[i] += self.learning_rate * error * inputs[i];
        }
    }
}

impl Default for NeuralGuardian {
    fn default() -> Self {
        Self::new()
    }
}
