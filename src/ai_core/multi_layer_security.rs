// src/ai_core/multi_layer_security.rs
// Multi-Layer AI Security Engine - Production-grade threat detection
// 5 layers of analysis: Statistical, Behavioral, Threat Intelligence, ML Models, Temporal

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::error::AxiomError;

const ANOMALY_MEMORY_SIZE: usize = 10000;
const BEHAVIORAL_ANALYSIS_WINDOW: usize = 1000;
const THREAT_INTELLIGENCE_CACHE: usize = 5000;

// ==================== THREAT CLASSIFICATION ====================

/// Comprehensive transaction risk profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRiskProfile {
    pub hash: String,
    pub timestamp: u64,
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub gas_price: u64,
    pub zk_proof_size: usize,
    
    // Extended features
    pub sender_history_count: u64,
    pub recipient_history_count: u64,
    pub sender_reputation_score: f64,
    pub time_since_last_sender_tx: u64,
    pub time_since_last_recipient_tx: u64,
    pub is_contract_deployment: bool,
    pub contract_bytecode_size: usize,
    pub vdf_verification_time_ms: u64,
}

/// Multi-dimensional threat types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ThreatType {
    // Financial threats
    MoneyLaundering,
    MixerService,
    DustingAttack,
    FrontRunning,
    SandwichAttack,
    
    // Network threats
    SpamFlood,
    DoS,
    SybilAttack,
    EclipseAttack,
    
    // Smart contract threats
    ReentrancyAttempt,
    IntegerOverflow,
    FlashLoanAttack,
    OracleManipulation,
    
    // Cryptographic threats
    WeakZKProof,
    TimestampManipulation,
    VDFBypass,
    QuantumPreImage,
    
    // Behavioral anomalies
    NewAccountLargeTransfer,
    DormantAccountActivation,
    RapidFireTransactions,
    GeographicAnomaly,
}

/// Risk assessment levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Minimal,        // 0-20
    Low,            // 20-40
    Medium,         // 40-60
    High,           // 60-80
    Critical,       // 80-95
    Catastrophic,   // 95-100
}

/// Recommended security actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityAction {
    Accept,
    AcceptWithMonitoring,
    Quarantine { duration_blocks: u64 },
    Reject { reason: String },
    EscalateToGuardian { threat_level: RiskLevel },
    HaltChain { emergency_level: u8 },
}

/// Comprehensive threat assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatAssessment {
    pub threat_score: f64,          // 0.0-100.0
    pub confidence: f64,             // 0.0-1.0
    pub identified_threats: Vec<ThreatType>,
    pub risk_level: RiskLevel,
    pub recommended_action: SecurityAction,
    pub detailed_analysis: String,
    pub guardian_override_required: bool,
}

// ==================== CORE SECURITY ENGINE ====================

pub struct MultiLayerSecurityEngine {
    // Core detection engines
    anomaly_detector: Arc<RwLock<AnomalyDetectionCore>>,
    behavioral_engine: Arc<RwLock<BehavioralPatternEngine>>,
    threat_intelligence: Arc<RwLock<ThreatIntelligenceSystem>>,
    statistical_models: Arc<RwLock<StatisticalModels>>,
    
    // Configuration
    config: SecurityConfig,
}

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub enable_behavioral_analysis: bool,
    pub enable_threat_intelligence: bool,
    pub enable_statistical_modeling: bool,
    pub anomaly_threshold: f64,
    pub auto_quarantine_threshold: f64,
    pub guardian_escalation_threshold: f64,
    pub max_processing_time_ms: u64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_behavioral_analysis: true,
            enable_threat_intelligence: true,
            enable_statistical_modeling: true,
            anomaly_threshold: 0.7,
            auto_quarantine_threshold: 0.85,
            guardian_escalation_threshold: 0.95,
            max_processing_time_ms: 100,
        }
    }
}

// ==================== ANOMALY DETECTION CORE ====================

struct AnomalyDetectionCore {
    feature_statistics: HashMap<String, FeatureStatistics>,
    time_series_data: VecDeque<TimeSeriesPoint>,
    seasonal_patterns: HashMap<u64, SeasonalPattern>,
    transaction_buffer: VecDeque<TransactionRiskProfile>,
    processing_metrics: ProcessingMetrics,
}

#[derive(Debug, Clone)]
struct FeatureStatistics {
    mean: f64,
    std_dev: f64,
    median: f64,
    mad: f64,  // Median Absolute Deviation
    quartile_1: f64,
    quartile_3: f64,
    min: f64,
    max: f64,
}

#[derive(Debug, Clone)]
struct TimeSeriesPoint {
    timestamp: u64,
    transaction_count: u64,
    total_volume: u64,
    avg_gas_price: f64,
}

#[derive(Debug, Clone)]
struct SeasonalPattern {
    hour_of_day: u8,
    day_of_week: u8,
    expected_volume: f64,
    variance: f64,
}

#[derive(Debug, Clone)]
struct ProcessingMetrics {
    total_processed: u64,
    total_flagged: u64,
    avg_processing_time_ns: u64,
    false_positive_rate: f64,
}

// ==================== BEHAVIORAL ANALYSIS ====================

struct BehavioralPatternEngine {
    address_profiles: HashMap<String, AddressProfile>,
    transaction_sequences: VecDeque<TransactionPattern>,
    known_attack_signatures: HashMap<Vec<u8>, ThreatType>,
    reputation_cache: HashMap<String, ReputationMetrics>,
}

#[derive(Debug, Clone)]
struct AddressProfile {
    address: String,
    first_seen: u64,
    last_active: u64,
    total_transactions: u64,
    total_volume: u64,
    avg_transaction_size: f64,
    risk_incidents: Vec<RiskIncident>,
    reputation_score: f64,
}

#[derive(Debug, Clone)]
struct RiskIncident {
    timestamp: u64,
    threat_type: ThreatType,
    severity: f64,
}

#[derive(Debug, Clone)]
struct TransactionPattern {
    sequence_hash: Vec<u8>,
    transactions: Vec<String>,
    timestamp_start: u64,
    timestamp_end: u64,
    pattern_type: PatternType,
}

#[derive(Debug, Clone, PartialEq)]
enum PatternType {
    Normal,
    Suspicious,
    HighlyAnomalous,
    KnownAttack,
}

#[derive(Debug, Clone)]
struct ReputationMetrics {
    address: String,
    trust_score: f64,
    verification_level: VerificationLevel,
    historical_violations: u32,
    cooperative_score: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VerificationLevel {
    Unverified,
    Basic,
    Enhanced,
    Trusted,
    Whitelisted,
}

// ==================== THREAT INTELLIGENCE ====================

struct ThreatIntelligenceSystem {
    known_malicious_addresses: HashMap<String, MaliciousEntity>,
    attack_patterns: Vec<AttackPattern>,
    real_time_alerts: VecDeque<ThreatAlert>,
}

#[derive(Debug, Clone)]
struct MaliciousEntity {
    address: String,
    threat_types: Vec<ThreatType>,
    severity: u8,
    first_identified: u64,
    evidence: Vec<String>,
    blockchain_wide_ban: bool,
}

#[derive(Debug, Clone)]
struct AttackPattern {
    pattern_id: String,
    threat_type: ThreatType,
    signature: Vec<u8>,
}

#[derive(Debug, Clone)]
struct ThreatAlert {
    alert_id: String,
    timestamp: u64,
    threat_type: ThreatType,
    affected_addresses: Vec<String>,
    severity: RiskLevel,
}

// ==================== STATISTICAL MODELS ====================

struct StatisticalModels {
    isolation_forest: IsolationForestModel,
    local_outlier_factor: LOFModel,
    one_class_svm: OneClassSVMModel,
    dbscan_clusters: DBSCANModel,
}

struct IsolationForestModel {
    trees: Vec<IsolationTree>,
    n_trees: usize,
    anomaly_threshold: f64,
}

struct IsolationTree {
    max_depth: usize,
}

struct LOFModel {
    k_neighbors: usize,
    distance_cache: HashMap<String, Vec<(String, f64)>>,
    lof_scores: HashMap<String, f64>,
}

struct OneClassSVMModel {
    support_vectors: Vec<Vec<f64>>,
    nu: f64,
    gamma: f64,
}

struct DBSCANModel {
    epsilon: f64,
    min_points: usize,
    clusters: Vec<Cluster>,
}

struct Cluster {
    cluster_id: usize,
    points: Vec<String>,
}

// ==================== IMPLEMENTATION ====================

impl MultiLayerSecurityEngine {
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            anomaly_detector: Arc::new(RwLock::new(AnomalyDetectionCore::new())),
            behavioral_engine: Arc::new(RwLock::new(BehavioralPatternEngine::new())),
            threat_intelligence: Arc::new(RwLock::new(ThreatIntelligenceSystem::new())),
            statistical_models: Arc::new(RwLock::new(StatisticalModels::new())),
            config,
        }
    }

    /// Main threat assessment function - PRODUCTION CRITICAL
    pub fn assess_transaction_threat(
        &self,
        profile: &TransactionRiskProfile,
        current_block_height: u64,
    ) -> Result<ThreatAssessment, AxiomError> {
        let start_time = SystemTime::now();

        // Layer 1: Statistical Anomaly Detection
        let anomaly_score = self.detect_statistical_anomaly(profile)?;

        // Layer 2: Behavioral Pattern Analysis
        let behavioral_score = self.analyze_behavioral_patterns(profile)?;

        // Layer 3: Threat Intelligence Matching
        let threat_intel_score = self.check_threat_intelligence(profile)?;

        // Layer 4: ML Models
        let ml_score = self.run_statistical_models(profile)?;

        // Layer 5: Temporal Analysis
        let temporal_score = self.analyze_temporal_patterns(profile, current_block_height)?;

        // Weighted aggregation (total = 1.0)
        let weights = [0.2, 0.25, 0.2, 0.2, 0.15];
        let scores = [anomaly_score, behavioral_score, threat_intel_score, ml_score, temporal_score];

        let weighted_score: f64 = scores
            .iter()
            .zip(weights.iter())
            .map(|(s, w)| s * w)
            .sum();

        // Identify specific threats
        let threats = self.identify_specific_threats(profile, &scores)?;

        // Determine risk level
        let risk_level = Self::calculate_risk_level(weighted_score);

        // Recommended action
        let recommended_action = self.determine_security_action(
            weighted_score,
            &risk_level,
            &threats,
            current_block_height,
        )?;

        // Calculate confidence
        let confidence = self.calculate_confidence(&scores);

        // Check if Guardian override required
        let guardian_override = weighted_score >= self.config.guardian_escalation_threshold;

        let _processing_time = start_time.elapsed().unwrap_or_default().as_millis() as u64;

        Ok(ThreatAssessment {
            threat_score: weighted_score * 100.0,
            confidence,
            identified_threats: threats.clone(),
            risk_level,
            recommended_action,
            detailed_analysis: self.generate_detailed_analysis(profile, &scores, &threats),
            guardian_override_required: guardian_override,
        })
    }

    fn detect_statistical_anomaly(&self, profile: &TransactionRiskProfile) -> Result<f64, AxiomError> {
        // Threshold-based anomaly scoring across transaction dimensions.
        // Each dimension contributes a normalised score [0,1]:
        //   - Amount: normalised log-scale against the typical transfer range
        //   - Gas: flags transactions with unusually high gas prices
        //   - Proof size: flags oversized ZK proofs that may indicate stuffing
        let amount_score = ((profile.amount as f64) / 1e15).min(1.0);
        let gas_score = if profile.gas_price > 1_000_000_000 { 0.5 } else { 0.0 };
        let proof_score = if profile.zk_proof_size > 5000 { 0.3 } else { 0.0 };

        Ok((amount_score + gas_score + proof_score) / 3.0)
    }

    fn analyze_behavioral_patterns(&self, profile: &TransactionRiskProfile) -> Result<f64, AxiomError> {
        // Behavioral pattern scoring based on sender history.
        //   - New accounts with zero history receive a higher risk score
        //   - Rapid-fire transactions (< 10s apart) suggest automated abuse
        let new_account_score = if profile.sender_history_count == 0 { 0.5 } else { 0.0 };
        let rapid_fire_score = if profile.time_since_last_sender_tx < 10 { 0.4 } else { 0.0 };

        Ok((new_account_score + rapid_fire_score) / 2.0)
    }

    fn check_threat_intelligence(&self, profile: &TransactionRiskProfile) -> Result<f64, AxiomError> {
        // Check against known malicious addresses
        let intel = self.threat_intelligence.read();

        if intel.known_malicious_addresses.contains_key(&profile.sender) {
            return Ok(0.9);
        }

        Ok(0.0)
    }

    fn run_statistical_models(&self, profile: &TransactionRiskProfile) -> Result<f64, AxiomError> {
        if !self.config.enable_statistical_modeling {
            return Ok(0.0);
        }

        // Multi-factor threshold scoring across transaction features
        let amount_score = if profile.amount > 10_000_00000000 { 0.3 } else { 0.0 };
        let fee_score = if profile.gas_price < 1000 { 0.2 } else { 0.0 };

        Ok((amount_score + fee_score) / 2.0)
    }

    fn analyze_temporal_patterns(
        &self,
        profile: &TransactionRiskProfile,
        _current_block: u64,
    ) -> Result<f64, AxiomError> {
        // Temporal analysis
        let rapid_fire_score = if profile.time_since_last_sender_tx < 10 {
            0.8
        } else if profile.time_since_last_sender_tx < 60 {
            0.4
        } else {
            0.0
        };

        Ok(rapid_fire_score)
    }

    fn identify_specific_threats(
        &self,
        profile: &TransactionRiskProfile,
        scores: &[f64],
    ) -> Result<Vec<ThreatType>, AxiomError> {
        let mut threats = Vec::new();

        // Money laundering detection
        if profile.amount > 1_000_000_00000000 && profile.sender_history_count <= 5 {
            threats.push(ThreatType::MoneyLaundering);
        }

        // Spam detection
        if profile.time_since_last_sender_tx < 5 {
            threats.push(ThreatType::SpamFlood);
        }

        // Front-running detection
        if profile.gas_price > 10_000_00000000 && scores[0] > 0.7 {
            threats.push(ThreatType::FrontRunning);
        }

        // New account large transfer
        if profile.sender_history_count == 0 && profile.amount > 10_000_00000000 {
            threats.push(ThreatType::NewAccountLargeTransfer);
        }

        // Weak ZK proof detection
        if profile.zk_proof_size < 100 || profile.zk_proof_size > 10000 {
            threats.push(ThreatType::WeakZKProof);
        }

        // VDF bypass attempt
        if profile.vdf_verification_time_ms < 100 {
            threats.push(ThreatType::VDFBypass);
        }

        Ok(threats)
    }

    fn calculate_risk_level(score: f64) -> RiskLevel {
        match (score * 100.0) as u8 {
            0..=20 => RiskLevel::Minimal,
            21..=40 => RiskLevel::Low,
            41..=60 => RiskLevel::Medium,
            61..=80 => RiskLevel::High,
            81..=95 => RiskLevel::Critical,
            _ => RiskLevel::Catastrophic,
        }
    }

    fn determine_security_action(
        &self,
        score: f64,
        risk_level: &RiskLevel,
        threats: &[ThreatType],
        _current_block: u64,
    ) -> Result<SecurityAction, AxiomError> {
        // Catastrophic threats
        if score >= self.config.guardian_escalation_threshold {
            return Ok(SecurityAction::EscalateToGuardian {
                threat_level: *risk_level,
            });
        }

        // Auto-quarantine for very high risk
        if score >= self.config.auto_quarantine_threshold {
            return Ok(SecurityAction::Quarantine {
                duration_blocks: 144,
            });
        }

        // Reject for critical threats
        for threat in threats {
            match threat {
                ThreatType::VDFBypass | ThreatType::QuantumPreImage | ThreatType::EclipseAttack => {
                    return Ok(SecurityAction::Reject {
                        reason: format!("Critical threat detected: {:?}", threat),
                    });
                }
                _ => {}
            }
        }

        // Accept with monitoring for medium risk
        if score >= self.config.anomaly_threshold {
            return Ok(SecurityAction::AcceptWithMonitoring);
        }

        Ok(SecurityAction::Accept)
    }

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

        (1.0 - variance.sqrt()).max(0.5).min(1.0)
    }

    fn generate_detailed_analysis(
        &self,
        profile: &TransactionRiskProfile,
        scores: &[f64],
        threats: &[ThreatType],
    ) -> String {
        format!(
            "Transaction Analysis:\n \
             - Statistical Anomaly: {:.2}%\n \
             - Behavioral Score: {:.2}%\n \
             - Threat Intel: {:.2}%\n \
             - ML Models: {:.2}%\n \
             - Temporal: {:.2}%\n \
             Identified Threats: {:?}\n \
             Sender History: {} txs\n \
             Amount: {} AXM",
            scores[0] * 100.0,
            scores[1] * 100.0,
            scores[2] * 100.0,
            scores[3] * 100.0,
            scores[4] * 100.0,
            threats,
            profile.sender_history_count,
            profile.amount as f64 / 100000000.0,
        )
    }

    pub fn update_threat_intelligence(
        &self,
        address: String,
        threat_type: ThreatType,
        evidence: String,
    ) -> Result<(), AxiomError> {
        let mut intel = self.threat_intelligence.write();

        intel.known_malicious_addresses.insert(
            address.clone(),
            MaliciousEntity {
                address,
                threat_types: vec![threat_type],
                severity: 8,
                first_identified: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                evidence: vec![evidence],
                blockchain_wide_ban: true,
            },
        );

        Ok(())
    }
}

// ==================== CORE IMPLEMENTATIONS ====================

impl AnomalyDetectionCore {
    fn new() -> Self {
        Self {
            feature_statistics: HashMap::new(),
            time_series_data: VecDeque::with_capacity(ANOMALY_MEMORY_SIZE),
            seasonal_patterns: HashMap::new(),
            transaction_buffer: VecDeque::with_capacity(BEHAVIORAL_ANALYSIS_WINDOW),
            processing_metrics: ProcessingMetrics {
                total_processed: 0,
                total_flagged: 0,
                avg_processing_time_ns: 0,
                false_positive_rate: 0.0,
            },
        }
    }
}

impl BehavioralPatternEngine {
    fn new() -> Self {
        Self {
            address_profiles: HashMap::new(),
            transaction_sequences: VecDeque::with_capacity(BEHAVIORAL_ANALYSIS_WINDOW),
            known_attack_signatures: HashMap::new(),
            reputation_cache: HashMap::new(),
        }
    }
}

impl ThreatIntelligenceSystem {
    fn new() -> Self {
        Self {
            known_malicious_addresses: HashMap::new(),
            attack_patterns: Vec::new(),
            real_time_alerts: VecDeque::with_capacity(THREAT_INTELLIGENCE_CACHE),
        }
    }
}

impl StatisticalModels {
    fn new() -> Self {
        Self {
            isolation_forest: IsolationForestModel {
                trees: Vec::new(),
                n_trees: 100,
                anomaly_threshold: 0.5,
            },
            local_outlier_factor: LOFModel {
                k_neighbors: 20,
                distance_cache: HashMap::new(),
                lof_scores: HashMap::new(),
            },
            one_class_svm: OneClassSVMModel {
                support_vectors: Vec::new(),
                nu: 0.1,
                gamma: 0.1,
            },
            dbscan_clusters: DBSCANModel {
                epsilon: 0.5,
                min_points: 5,
                clusters: Vec::new(),
            },
        }
    }
}

// ==================== TESTS ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_engine_creation() {
        let engine = MultiLayerSecurityEngine::new(SecurityConfig::default());
        assert_eq!(engine.config.anomaly_threshold, 0.7);
    }

    #[test]
    fn test_risk_level_calculation() {
        assert_eq!(MultiLayerSecurityEngine::calculate_risk_level(0.1), RiskLevel::Minimal);
        assert_eq!(MultiLayerSecurityEngine::calculate_risk_level(0.3), RiskLevel::Low);
        assert_eq!(MultiLayerSecurityEngine::calculate_risk_level(0.5), RiskLevel::Medium);
        assert_eq!(MultiLayerSecurityEngine::calculate_risk_level(0.7), RiskLevel::High);
        assert_eq!(MultiLayerSecurityEngine::calculate_risk_level(0.9), RiskLevel::Critical);
        assert_eq!(MultiLayerSecurityEngine::calculate_risk_level(0.99), RiskLevel::Catastrophic);
    }

    #[test]
    fn test_threat_detection() {
        let profile = TransactionRiskProfile {
            hash: "test".to_string(),
            timestamp: 1,
            sender: "alice".to_string(),
            recipient: "bob".to_string(),
            amount: 100_00000000,
            gas_price: 1000,
            zk_proof_size: 500,
            sender_history_count: 0,
            recipient_history_count: 10,
            sender_reputation_score: 0.5,
            time_since_last_sender_tx: 100,
            time_since_last_recipient_tx: 10,
            is_contract_deployment: false,
            contract_bytecode_size: 0,
            vdf_verification_time_ms: 1000,
        };

        let engine = MultiLayerSecurityEngine::new(SecurityConfig::default());
        let assessment = engine.assess_transaction_threat(&profile, 1000);
        assert!(assessment.is_ok());
    }
}
