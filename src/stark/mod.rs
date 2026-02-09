// src/stark/mod.rs — RISC Zero zkVM STARK Proving Module
//
// This module provides the host-side interface for generating and verifying
// RISC Zero STARK proofs that enforce the 124M supply integrity law.
//
// Architecture:
//   Guest (methods/guest/src/main.rs) — runs inside the zkVM, enforces the law
//   Host  (this module)               — orchestrates proof generation & verification
//
// The `risc0` Cargo feature gates the actual RISC Zero prover. When
// disabled the module still compiles (types are always available) but
// proof generation requires the feature.

pub mod prover;

pub use prover::{TransactionData, StarkProver, StarkReceipt};
