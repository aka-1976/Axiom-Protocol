use thiserror::Error;

/// Axiom SDK error types
#[derive(Debug, Error)]
pub enum AxiomError {
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    
    #[error("Insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: u64, need: u64 },
    
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    #[error("Wallet error: {0}")]
    Wallet(String),

    #[error("Proof error: {0}")]
    Proof(String),
}

/// Result type for Axiom SDK operations
pub type Result<T> = std::result::Result<T, AxiomError>;
