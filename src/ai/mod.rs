// src/ai/mod.rs - AI Oracle Network
pub mod oracle;

pub use oracle::{
    OracleQuery,
    OracleResponse,
    OracleConsensus,
    OracleNode,
    OracleConsensusManager,
    query_oracle,
    validate_ai_inference,
};
