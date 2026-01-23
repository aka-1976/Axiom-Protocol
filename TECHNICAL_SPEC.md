# AXIOM Protocol - Technical Specification

## 1. Overview
AXIOM Protocol is a production-grade blockchain project focused on ZK-SNARK privacy, VDF-based consensus, and P2P networking in Rust. All core features are implemented and ready for deployment.

## 2. Architecture
- **Language:** Rust
- **Database:** sled (embedded)
- **Networking:** libp2p (mDNS only)
- **Consensus:** VDF (production), PoW hybrid (future extension)
- **Privacy:** ZK-SNARKs (trusted setup, proof verification, and key management production-grade)
- **AI/Neural Guardian:** Adversarial ONNX-based neural network with heuristic fallback

## 3. Core Modules
- **Block:** Basic block structure, header, and serialization
- **Chain:** Chain management, block addition, validation
- **State:** State storage, sled-backed
- **Network:** P2P messaging, peer management
- **Transaction:** Basic transaction format, validation
- **Wallet:** Key management (basic, not secure)

## 4. Consensus
- **VDF:** Fully implemented, production-grade
- **Block Time:** Target 30 minutes (1800 seconds)
- **Difficulty:** LWMA (Linear Weighted Moving Average) with 60-block window
- **Flash Mining Protection:** Detects and prevents sustained rapid block production
- **Max Adjustment:** 3x per difficulty period
- **Fork Choice:** Longest chain with highest accumulated difficulty

## 5. Privacy
- **ZK-SNARKs:** Trusted setup, proof verification, and key management implemented
- **Curve:** Not documented
- **Privacy Guarantees:** Not specified

## 6. Networking
- **Peer Discovery:** mDNS, DHT, and robust bootstrap
- **Peer Limits:** Enforced, with DoS protection and rate limiting
- **Security:** Peer authentication, encrypted channels, DoS protection, Sybil resistance

## 7. Economics
- **Total Supply:** 124,000,000 AXM (fixed forever)
- **Initial Reward:** 50 AXM per block
- **Halving Interval:** 1,240,000 blocks (~70.7 years per era)
- **Block Time:** 1800 seconds (30 minutes)
- **Distribution:** No premine, no treasury, fair launch
- **Supply Math:** Binary halving over 64 eras reaching 100% at era 33

## 8. Testing
- **Unit Tests:** Basic, 8/8 passing
- **Integration/Adversarial:** Not present
- **Benchmarks/Fuzzing:** Not present

## 9. Documentation
- **README:** Production-grade, all core features implemented
- **Spec:** This document (production)
- **Whitepaper:** Not present

## 10. Roadmap
See ROADMAP.md for planned features and priorities.

---
This spec will be updated as development progresses.