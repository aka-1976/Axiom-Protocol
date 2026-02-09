# Changelog - AXIOM Protocol

All notable changes to this project will be documented in this file.

## [4.1.0] - 2026-02-08

### 🔐 512-Bit Security Upgrade
- Upgraded protocol-wide hashing from 256-bit to 512-bit using BLAKE3 XOF mode
- Added `axiom_hash_512()` function for 512-bit extended output hashing
- Added `GENESIS_ANCHOR_512` constant for chain identity verification
- Added `Block::calculate_hash_512()` for 512-bit block hashing
- Added custom `serde_bytes_64` module for serializing 64-byte arrays

### 🤖 Deterministic AI Oracle
- Added `query_oracle()` for deterministic AI oracle seal generation
- Local Ollama integration at temperature 0 / seed 42 for reproducible output
- Automatic BLAKE3-512 fallback when model is unavailable
- Oracle seal now wired into mining flow for per-block AxiomPulse broadcast

### AxiomPulse Upgrade
- `block_hash` field widened from `[u8; 32]` to `[u8; 64]` (512-bit)
- Added `oracle_seal: [u8; 64]` for AI Oracle proof

### ⚡ ZK-STARK Migration (Winterfell 0.9)
- Migrated from `StarkProof` to `Proof`, removed `HashFunction`
- Updated `verify()` to use 3 type parameters + `AcceptableOptions`
- Expanded `Prover` trait with `HashFn`, `RandomCoin`, `TraceLde`, `ConstraintEvaluator`
- Expanded `Air` trait with `GkrProof`, `GkrVerifier`
- Replaced `BaseElement::from(u128)` with `BaseElement::new(u128)`
- Extracted `MIN_SECURITY_BITS` shared constant

### 🛠️ Build Fixes
- Fixed 32+ winterfell 0.9 compilation errors in ZK module
- Fixed pre-existing main.rs build issues (extra brace, missing imports, error conversion)
- All code compiles with zero errors

## [4.0.0] - 2026-02-08

### Consolidated Production Release
- Unified all v3.x features into single production release
- AI Oracle, AI Enhancement Module, Axiom SDK, Bridge Contracts, Block Explorer
- Production config, Systemd services, full documentation

## [1.0.0] - 2025-01-20

### 🎉 Initial Release - AXIOM Protocol

#### Rebranded from Qubit Protocol
- Complete rebranding to AXIOM Protocol
- New visual identity and messaging
- Updated binary signature: AXIOM in ASCII

#### Core Features
- ✅ ZK-STARK privacy (mandatory for all transactions)
- ✅ VDF + PoW hybrid consensus
- ✅ Neural Guardian AI security
- ✅ 84M AXM fixed supply
- ✅ 20 AXM initial block reward
- ✅ 840,000 block halving interval

#### Production Features
- Complete error handling system (60+ error types)
- Production logging with rotation
- Configuration management (TOML-based)
- Mempool with fee-based ordering
- Multi-language SDKs (Python, JavaScript, Rust)
- Block explorer (React + Actix)
- AI Oracle Network

#### Testing
- 50+ comprehensive tests passing
- Network stress testing completed
- ZK proof generation/verification tested

### Upgrade Notes
If migrating from Qubit Protocol:
1. Backup your wallet keys
2. Run rebranding script: `./rebrand-to-axiom.sh`
3. Rebuild: `cargo clean && cargo build --release`
4. Update configuration: Use `axiom.toml`
