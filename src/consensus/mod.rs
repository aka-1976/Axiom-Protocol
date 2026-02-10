// src/consensus/mod.rs - Consensus mechanisms
pub mod vdf;
pub mod lwma;
pub mod validator;

pub use vdf::{VDF, VDFProof, VDFBlockHeader};
pub use lwma::{
    calculate_lwma_difficulty,
    detect_flash_mining,
    estimate_hashrate,
    format_hashrate,
    meets_difficulty,
    difficulty_to_target,
    BlockHeader,
    TARGET_BLOCK_TIME,
    LWMA_WINDOW,
    MIN_DIFFICULTY,
};
