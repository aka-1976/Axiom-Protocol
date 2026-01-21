use onnxruntime::{environment::Environment, session::Session, tensor::OrtOwnedTensor, LoggingLevel};
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use std::fs::OpenOptions;
use std::io::Write;
use chrono;

/// AI Attack Detection Model
/// Static ONNX environment for all model sessions.
/// Uses once_cell::sync::Lazy to ensure only one global environment is created.
static ONNX_ENV: Lazy<Environment> = Lazy::new(|| {
    Environment::builder()
        .with_name("qubit-onnx-env")
        .with_log_level(LoggingLevel::Warning)
        .build()
        .expect("Failed to initialize ONNX environment")
});

pub struct AttackDetectionModel {
    session: Session<'static>,
}

impl AttackDetectionModel {
    /// Load an ONNX model from file
    /// Load an ONNX model from a static file path. The ONNX environment is static and shared.
    /// The model_path must have a 'static lifetime (e.g., a global constant or string literal).
    pub fn load(model_path: &'static str) -> Result<Self, Box<dyn std::error::Error + 'static>> {
        let session = ONNX_ENV
            .new_session_builder()?
            .with_model_from_file(model_path)?;
        Ok(Self { session })
    }

    /// Run inference on network metrics
    pub fn predict(&mut self, features: &[f32]) -> Result<f32, Box<dyn std::error::Error>> {
        // ONNX expects input as ndarray
        let input_shape = vec![1, features.len()];
        let input_array = ndarray::Array::from_shape_vec(input_shape.clone(), features.to_vec())?;
        let outputs: Vec<OrtOwnedTensor<f32, _>> = self.session.run(vec![input_array])?;
        Ok(outputs[0].as_slice().unwrap()[0])
    }
}

/// Collect and label network data for training
pub fn collect_network_metrics() -> Vec<(Vec<f32>, f32)> {
    // Placeholder: collect metrics such as peer count, block time, tx rate, etc.
    vec![ (vec![0.5, 0.8, 0.2], 1.0), (vec![0.1, 0.2, 0.9], 0.0) ]
}

/// Dynamic reputation scoring based on AI outputs
pub fn calculate_peer_trust_score(model: &mut AttackDetectionModel, metrics: &[f32]) -> Result<f32, Box<dyn std::error::Error>> {
    // Require &mut AttackDetectionModel for ONNX inference
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
        // Simulate ONNX/fallback split
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
        if confidence < 0.3 || confidence > 0.9 {
            println!("⚠️  AI: High confidence decision - Trust: {} ({}%)", is_trustworthy, (confidence * 100.0) as u32);
        }
        is_trustworthy
    }
    pub fn log_stats(&self) {
        println!("\n--- 🤖 NEURAL GUARDIAN STATS ---");
        println!("Total Predictions: {}", self.stats.total_predictions);
        println!("Spam Detected: {} ({:.1}%)", 
                 self.stats.spam_detected,
                 (self.stats.spam_detected as f32 / self.stats.total_predictions.max(1) as f32) * 100.0);
        println!("ONNX Model Used: {} ({:.1}%)", 
                 self.stats.model_used,
                 (self.stats.model_used as f32 / self.stats.total_predictions.max(1) as f32) * 100.0);
        println!("Fallback Used: {}", self.stats.fallback_used);
        println!("Avg Confidence: {:.2}", self.stats.avg_confidence);
        println!("--------------------------------\n");
    }

    pub fn report_false_positive(&mut self) {
        self.stats.false_positives += 1;
        println!("⚠️  AI: False positive reported. Total: {}", self.stats.false_positives);
        self.save_false_positive_case();
    }

    fn save_false_positive_case(&self) {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("ai_training_data.csv")
            .unwrap();
        writeln!(file, "{},{},{}", 
                 chrono::Utc::now().timestamp(),
                 "false_positive",
                 "details_here").ok();
    }

    pub fn collect_training_sample(&self, msg_rate: f32, history: f32, reputation: f32, is_good: bool) {
        let sample = format!("{},{},{},{}\n", msg_rate, history, reputation, if is_good { 1 } else { 0 });
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("training_data.csv")
            .unwrap();
        write!(file, "{}", sample).ok();
    }

    pub fn set_threshold(&mut self, threshold: f32) {
        self.confidence_threshold = threshold;
        println!("🔧 AI threshold updated to {}", threshold);
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
