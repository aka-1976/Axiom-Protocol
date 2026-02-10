// src/ai_core/mod.rs
// AI Core module - Multi-layer security and threat detection

pub mod multi_layer_security;
pub mod production_ml;

pub use multi_layer_security::{
    MultiLayerSecurityEngine,
    TransactionRiskProfile,
    ThreatAssessment,
    ThreatType,
    RiskLevel,
    SecurityAction,
    SecurityConfig,
};
