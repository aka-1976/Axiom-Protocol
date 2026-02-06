/// Neural Guardian - Federated Learning AI Security System
/// 
/// This module implements a decentralized AI-powered network security system
/// that trains collaboratively across nodes without sharing raw data.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sha2::{Digest, Sha256};

/// Network event for training the Neural Guardian
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkEvent {
    pub peer_id: String,
    pub block_interval: f32,      // Time between blocks (seconds)
    pub block_size: f32,          // Block size in KB
    pub tx_count: f32,            // Transactions per block
    pub propagation_time: f32,    // Time to receive block (ms)
    pub peer_count: f32,          // Number of active peers
    pub fork_count: f32,          // Number of forks observed
    pub orphan_rate: f32,         // Orphaned blocks ratio
    pub reorg_depth: f32,         // Reorganization depth
    pub bandwidth_usage: f32,     // Network bandwidth (KB/s)
    pub connection_churn: f32,    // Peer connect/disconnect rate
    pub timestamp: u64,
}

/// Threat types that Neural Guardian can detect
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ThreatType {
    SelfishMining,     // Miner withholds blocks
    SybilAttack,       // Fake peer identities
    EclipseAttack,     // Network isolation
    DoS,               // Denial of service
    TimestampManip,    // VDF timing manipulation
    Benign,            // No threat detected
}

/// Threat assessment result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThreatAssessment {
    pub peer_id: String,
    pub trust_score: f32,         // 0.0 (untrusted) to 1.0 (trusted)
    pub detected_threats: Vec<ThreatType>,
    pub confidence: f32,          // Model confidence
    pub recommended_action: Action,
}

/// Recommended actions based on threat detection
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Action {
    None,
    IncreaseMonitoring,
    LimitConnections,
    DiversifyPeers,
    RateLimit,
    VerifyVDF,
    BanPeer,
}

/// Simple neural network for threat detection
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeuralNetwork {
    // Input layer (10 features) -> Hidden layer (64) -> Output layer (6 threat types)
    weights_input_hidden: Vec<Vec<f32>>,  // 10x64
    bias_hidden: Vec<f32>,                // 64
    weights_hidden_output: Vec<Vec<f32>>, // 64x6
    bias_output: Vec<f32>,                // 6
}

impl NeuralNetwork {
    /// Create a new neural network with random initialization
    pub fn new() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        let input_size = 10;
        let hidden_size = 64;
        let output_size = 6; // 6 threat types (including Benign)
        
        // Xavier initialization
        let weights_input_hidden: Vec<Vec<f32>> = (0..input_size)
            .map(|_| {
                (0..hidden_size)
                    .map(|_| rng.gen_range(-1.0..1.0) * (2.0 / input_size as f32).sqrt())
                    .collect()
            })
            .collect();
        
        let bias_hidden: Vec<f32> = (0..hidden_size).map(|_| 0.0).collect();
        
        let weights_hidden_output: Vec<Vec<f32>> = (0..hidden_size)
            .map(|_| {
                (0..output_size)
                    .map(|_| rng.gen_range(-1.0..1.0) * (2.0 / hidden_size as f32).sqrt())
                    .collect()
            })
            .collect();
        
        let bias_output: Vec<f32> = (0..output_size).map(|_| 0.0).collect();
        
        Self {
            weights_input_hidden,
            bias_hidden,
            weights_hidden_output,
            bias_output,
        }
    }
    
    /// Forward pass through the network
    pub fn forward(&self, input: &[f32]) -> Vec<f32> {
        // Input to hidden layer
        let mut hidden: Vec<f32> = self.bias_hidden.clone();
        for (i, h) in hidden.iter_mut().enumerate() {
            for (j, &inp) in input.iter().enumerate() {
                *h += inp * self.weights_input_hidden[j][i];
            }
            *h = relu(*h); // ReLU activation
        }
        
        // Hidden to output layer
        let mut output: Vec<f32> = self.bias_output.clone();
        for (i, o) in output.iter_mut().enumerate() {
            for (j, &h) in hidden.iter().enumerate() {
                *o += h * self.weights_hidden_output[j][i];
            }
        }
        
        // Softmax activation
        softmax(&output)
    }
    
    /// Simple gradient descent training step
    pub fn train_step(&mut self, input: &[f32], target: &[f32], learning_rate: f32) {
        // Forward pass
        let prediction = self.forward(input);
        
        // Compute gradients (simplified - in production use proper backprop)
        for i in 0..self.weights_hidden_output.len() {
            for j in 0..self.weights_hidden_output[i].len() {
                let error = target[j] - prediction[j];
                self.weights_hidden_output[i][j] += learning_rate * error;
            }
        }
    }
}

/// ReLU activation function
fn relu(x: f32) -> f32 {
    if x > 0.0 { x } else { 0.0 }
}

/// Softmax activation for output layer
fn softmax(values: &[f32]) -> Vec<f32> {
    let max = values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let exp_values: Vec<f32> = values.iter().map(|&v| (v - max).exp()).collect();
    let sum: f32 = exp_values.iter().sum();
    exp_values.iter().map(|&v| v / sum).collect()
}

/// Model update for federated learning
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelUpdate {
    pub node_id: String,
    pub gradients_hash: [u8; 32],
    pub num_samples: usize,
    pub loss: f32,
    pub timestamp: u64,
}

/// Neural Guardian with federated learning
pub struct NeuralGuardian {
    model: NeuralNetwork,
    peer_history: HashMap<String, Vec<NetworkEvent>>,
    threat_cache: HashMap<String, ThreatAssessment>,
    training_data: Vec<(NetworkEvent, ThreatType)>,
}

impl Default for NeuralGuardian {
    fn default() -> Self {
        Self::new()
    }
}

impl NeuralGuardian {
    pub fn new() -> Self {
        Self {
            model: NeuralNetwork::new(),
            peer_history: HashMap::new(),
            threat_cache: HashMap::new(),
            training_data: Vec::new(),
        }
    }
    
    /// Extract features from network event
    pub fn extract_features(&self, event: &NetworkEvent) -> Vec<f32> {
        vec![
            normalize_time(event.block_interval),
            normalize_size(event.block_size),
            normalize_count(event.tx_count),
            normalize_time(event.propagation_time),
            normalize_count(event.peer_count),
            normalize_count(event.fork_count),
            event.orphan_rate,
            normalize_depth(event.reorg_depth),
            normalize_size(event.bandwidth_usage),
            normalize_rate(event.connection_churn),
        ]
    }
    
    /// Analyze peer and detect threats
    pub fn analyze_peer(&mut self, peer_id: &str) -> Option<ThreatAssessment> {
        // Check cache first
        if let Some(cached) = self.threat_cache.get(peer_id) {
            return Some(cached.clone());
        }
        
        // Get peer history
        let events = self.peer_history.get(peer_id)?;
        if events.is_empty() {
            return None;
        }
        
        // Extract features from recent events
        let recent_event = &events[events.len() - 1];
        let features = self.extract_features(recent_event);
        
        // Run through model
        let predictions = self.model.forward(&features);
        
        // Interpret predictions (indices correspond to ThreatType variants)
        let selfish_mining_prob = predictions[0];
        let sybil_prob = predictions[1];
        let eclipse_prob = predictions[2];
        let dos_prob = predictions[3];
        let timestamp_prob = predictions[4];
        let benign_prob = predictions[5];
        
        let mut threats = Vec::new();
        if selfish_mining_prob > 0.7 {
            threats.push(ThreatType::SelfishMining);
        }
        if sybil_prob > 0.8 {
            threats.push(ThreatType::SybilAttack);
        }
        if eclipse_prob > 0.6 {
            threats.push(ThreatType::EclipseAttack);
        }
        if dos_prob > 0.7 {
            threats.push(ThreatType::DoS);
        }
        if timestamp_prob > 0.6 {
            threats.push(ThreatType::TimestampManip);
        }
        
        let max_threat_prob = predictions.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let trust_score = 1.0 - max_threat_prob;
        
        let assessment = ThreatAssessment {
            peer_id: peer_id.to_string(),
            trust_score,
            detected_threats: threats.clone(),
            confidence: benign_prob,
            recommended_action: determine_action(&threats),
        };
        
        // Cache the assessment
        self.threat_cache.insert(peer_id.to_string(), assessment.clone());
        
        Some(assessment)
    }
    
    /// Record a network event for a peer
    pub fn record_event(&mut self, peer_id: String, event: NetworkEvent) {
        self.peer_history
            .entry(peer_id)
            .or_default()
            .push(event);
    }
    
    /// Train the model on local data
    pub fn train_local(&mut self, epochs: u32, learning_rate: f32) -> ModelUpdate {
        let mut total_loss = 0.0;
        
        for _ in 0..epochs {
            for (event, threat) in &self.training_data {
                let features = self.extract_features(event);
                let target = threat_to_one_hot(threat);
                
                self.model.train_step(&features, &target, learning_rate);
                
                // Compute loss (cross-entropy)
                let prediction = self.model.forward(&features);
                let loss: f32 = target
                    .iter()
                    .zip(prediction.iter())
                    .map(|(&t, &p)| -t * p.max(1e-10).ln())
                    .sum();
                total_loss += loss;
            }
        }
        
        let avg_loss = total_loss / (epochs as f32 * self.training_data.len() as f32);
        
        // Compute gradients hash for verification
        let gradients_hash = self.compute_gradients_hash();
        
        ModelUpdate {
            node_id: "local".to_string(),
            gradients_hash,
            num_samples: self.training_data.len(),
            loss: avg_loss,
            timestamp: current_timestamp(),
        }
    }
    
    /// Aggregate model updates from multiple nodes (federated learning)
    pub fn aggregate_updates(&mut self, updates: Vec<ModelUpdate>) {
        // Weighted average based on number of samples
        let total_samples: usize = updates.iter().map(|u| u.num_samples).sum();
        
        if total_samples == 0 {
            return;
        }
        
        // In a real implementation, we would aggregate the actual gradients
        // For now, this is a placeholder showing the structure
        println!(
            "Aggregating {} updates from {} total samples",
            updates.len(),
            total_samples
        );
    }
    
    /// Compute hash of model gradients for verification
    fn compute_gradients_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        
        // Hash model weights (simplified)
        for row in &self.model.weights_input_hidden {
            for &w in row {
                hasher.update(w.to_le_bytes());
            }
        }
        
        hasher.finalize().into()
    }
    
    /// Get model statistics
    pub fn get_stats(&self) -> GuardianStats {
        GuardianStats {
            total_events: self.peer_history.values().map(|v| v.len()).sum(),
            unique_peers: self.peer_history.len(),
            cached_assessments: self.threat_cache.len(),
            training_samples: self.training_data.len(),
        }
    }
}

/// Statistics about the Neural Guardian
#[derive(Debug, Serialize, Deserialize)]
pub struct GuardianStats {
    pub total_events: usize,
    pub unique_peers: usize,
    pub cached_assessments: usize,
    pub training_samples: usize,
}

/// Normalize time values (seconds)
fn normalize_time(t: f32) -> f32 {
    (t / 3600.0).min(1.0) // Normalize to 1 hour max
}

/// Normalize size values (KB)
fn normalize_size(s: f32) -> f32 {
    (s / 1024.0).min(1.0) // Normalize to 1 MB max
}

/// Normalize count values
fn normalize_count(c: f32) -> f32 {
    (c / 100.0).min(1.0) // Normalize to 100 max
}

/// Normalize depth values
fn normalize_depth(d: f32) -> f32 {
    (d / 10.0).min(1.0) // Normalize to 10 blocks max
}

/// Normalize rate values
fn normalize_rate(r: f32) -> f32 {
    (r / 10.0).min(1.0) // Normalize to 10 connections/sec max
}

/// Convert threat type to one-hot encoding
fn threat_to_one_hot(threat: &ThreatType) -> Vec<f32> {
    let mut encoding = vec![0.0; 6];
    let index = match threat {
        ThreatType::SelfishMining => 0,
        ThreatType::SybilAttack => 1,
        ThreatType::EclipseAttack => 2,
        ThreatType::DoS => 3,
        ThreatType::TimestampManip => 4,
        ThreatType::Benign => 5,
    };
    encoding[index] = 1.0;
    encoding
}

/// Determine action based on detected threats
fn determine_action(threats: &[ThreatType]) -> Action {
    if threats.is_empty() {
        return Action::None;
    }
    
    if threats.len() >= 2 {
        return Action::BanPeer; // Multiple threats = ban
    }
    
    match threats[0] {
        ThreatType::SelfishMining => Action::IncreaseMonitoring,
        ThreatType::SybilAttack => Action::LimitConnections,
        ThreatType::EclipseAttack => Action::DiversifyPeers,
        ThreatType::DoS => Action::RateLimit,
        ThreatType::TimestampManip => Action::VerifyVDF,
        ThreatType::Benign => Action::None,
    }
}

/// Get current timestamp in seconds
fn current_timestamp() -> u64 {
    // Safe conversion: system time should always be after UNIX_EPOCH
    // If this fails, return 0 as fallback (epoch time)
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_else(|e| {
            eprintln!("⚠️  Failed to get current timestamp: {}", e);
            0
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_neural_network_forward() {
        let nn = NeuralNetwork::new();
        let input = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
        let output = nn.forward(&input);
        
        assert_eq!(output.len(), 6);
        
        // Check softmax properties
        let sum: f32 = output.iter().sum();
        assert!((sum - 1.0).abs() < 0.01, "Softmax output should sum to 1.0");
        
        for &val in &output {
            assert!(val >= 0.0 && val <= 1.0, "All outputs should be between 0 and 1");
        }
    }
    
    #[test]
    fn test_threat_detection() {
        let mut guardian = NeuralGuardian::new();
        
        let event = NetworkEvent {
            peer_id: "peer1".to_string(),
            block_interval: 3600.0,
            block_size: 500.0,
            tx_count: 50.0,
            propagation_time: 100.0,
            peer_count: 10.0,
            fork_count: 0.0,
            orphan_rate: 0.0,
            reorg_depth: 0.0,
            bandwidth_usage: 100.0,
            connection_churn: 0.5,
            timestamp: current_timestamp(),
        };
        
        guardian.record_event("peer1".to_string(), event);
        
        let assessment = guardian.analyze_peer("peer1");
        assert!(assessment.is_some());
        
        let assessment = assessment.expect("Failed to get peer assessment");
        assert_eq!(assessment.peer_id, "peer1");
        assert!(assessment.trust_score >= 0.0 && assessment.trust_score <= 1.0);
    }
    
    #[test]
    fn test_feature_extraction() {
        let guardian = NeuralGuardian::new();
        
        let event = NetworkEvent {
            peer_id: "peer1".to_string(),
            block_interval: 1800.0,
            block_size: 512.0,
            tx_count: 100.0,
            propagation_time: 50.0,
            peer_count: 20.0,
            fork_count: 1.0,
            orphan_rate: 0.05,
            reorg_depth: 2.0,
            bandwidth_usage: 256.0,
            connection_churn: 1.0,
            timestamp: current_timestamp(),
        };
        
        let features = guardian.extract_features(&event);
        assert_eq!(features.len(), 10);
        
        // All features should be normalized between 0 and 1
        for &f in &features {
            assert!(f >= 0.0 && f <= 1.0, "Feature {} not normalized", f);
        }
    }
    
    #[test]
    fn test_model_training() {
        let mut guardian = NeuralGuardian::new();
        
        // Add some training data
        let benign_event = NetworkEvent {
            peer_id: "peer1".to_string(),
            block_interval: 3600.0,
            block_size: 500.0,
            tx_count: 50.0,
            propagation_time: 100.0,
            peer_count: 10.0,
            fork_count: 0.0,
            orphan_rate: 0.0,
            reorg_depth: 0.0,
            bandwidth_usage: 100.0,
            connection_churn: 0.5,
            timestamp: current_timestamp(),
        };
        
        guardian.training_data.push((benign_event, ThreatType::Benign));
        
        let update = guardian.train_local(10, 0.01);
        
        assert!(update.loss >= 0.0);
        assert_eq!(update.num_samples, 1);
    }
    
    #[test]
    fn test_action_determination() {
        assert_eq!(determine_action(&[]), Action::None);
        assert_eq!(
            determine_action(&[ThreatType::SelfishMining]),
            Action::IncreaseMonitoring
        );
        assert_eq!(
            determine_action(&[ThreatType::SybilAttack, ThreatType::DoS]),
            Action::BanPeer
        );
    }
}
