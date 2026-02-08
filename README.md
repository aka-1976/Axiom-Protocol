# AXIOM Protocol - Privacy-First Blockchain with AI Security

**Status**: ✅ Production Mainnet | **Version**: v4.0.0 (Consolidated Release) | **Network**: Active | **Consensus**: VDF + Blake3 PoW | **Supply**: 124M Fixed

> 🚀 **v4.0.0 Consolidated Release**: All v3.x features unified—5-layer AI threat detection, Guardian-enforced consensus, modular Discv5/libp2p networking, AI Oracle integration, SDK, explorer, bridge contracts, and full production cleanup. Mainnet-ready and validated.

---

## 🎯 What is AXIOM?

AXIOM is a production-grade, privacy-preserving blockchain with:
- **Fixed Supply**: 124,000,000 AXM (immutable, no pre-mine)
- **Fair Consensus**: VDF (30-minute blocks) + PoW (Blake3) hybrid
- **Mandatory Privacy**: ZK-SNARKs (Groth16) on all transactions
- **Eternal Guardian**: Sovereign network sentinel ensuring 24/7 consensus
- **Zero Governance**: Purely mathematical, no tokens, no directors

---

## ⚡ 60-Second Getting Started

```bash
# 1. Clone & build (2 minutes)
git clone https://github.com/Ghost-84M/Axiom-Protocol.git
cd Axiom-Protocol
cargo build --release

# 2. Run the node (instantly connects to mainnet)
./target/release/axiom-node

# 3. Verify syncing (in another terminal)
watch -n 5 './target/release/axiom-node status'
```

**That's it!** Your node automatically connects, syncs the blockchain, and maintains consensus through the Sovereign Guardian with integrated AI security.

---


## 🚀 v4.0.0: Consolidated Production Release

**Release Date:** February 8, 2026

v4.0.0 consolidates all v3.x line upgrades into a single production-validated release.

### Highlights

- **AI Oracle** (`ai-oracle/`): Claude API integration for on-chain AI oracle services
- **AI Enhancement Module** (`axiom-ai-enhancement/`): Extended AI security and analytics
- **Axiom SDK** (`axiom-sdk/`): Developer SDK for building on Axiom Protocol
- **Bridge Contracts** (`bridge-contracts/`): Cross-chain bridge infrastructure (Hardhat)
- **Block Explorer** (`explorer-backend/`, `explorer-frontend/`): Full-stack blockchain explorer
- **Production Config** (`config/bootstrap.toml`): Mainnet bootstrap configuration
- **Systemd Services** (`contrib/`): Production deployment services and logrotate

### Consolidated from Previous Releases

**v3.2.0 — Network Cleanup**
- All legacy code and duplicate network logic removed
- Network stack fully rewritten for production
- Clean modular libp2p/discv5, deduplicated `behaviour.rs`

**v3.1.0 — Discv5 Network & Modular Upgrade**
- Production-grade Discv5/libp2p networking stack (Kademlia, Ping, Identify, Gossipsub, mDNS, Noise, TCP, DNS, etc.)
- Modular network services: config, discv5_service, peer_manager, behaviour, gossip_handler, event_handler
- Peer manager with reputation, LRU cache, ban logic, and metrics
- Metrics module for Prometheus-style node metrics
- Integration tests for network stack validation

**v3.0.0 — AI Guardian Security**
- 5-Layer AI Threat Detection System
- Guardian-Enforced Consensus Optimization
- All 11 critical/medium protocol and test issues fixed
- 78 tests passing, 0 failed
- Guardian Safety Manifest, Multi-Layer Security Engine, AI Guardian Bridge

---

## 💻 System Requirements

| Requirement | Minimum | Recommended |
|-------------|---------|-------------|
| **CPU** | 2 cores | 4+ cores |
| **RAM** | 2 GB | 4+ GB |
| **Storage** | 10 GB | 50 GB SSD |
| **Network** | 1 Mbps | 10+ Mbps |
| **OS** | Linux/macOS/WSL | Ubuntu 20.04+ |

---

## 🔗 Network Setup

### Default Configuration (Recommended)
```bash
# Pre-configured to connect to mainnet bootstrap
./target/release/axiom-node
```
- Automatically discovers bootstrap node: `34.10.172.20:6000`
- Syncs entire blockchain history
- Joins consensus with other nodes

### For 5-Node Genesis Miner Setup
```bash
export AXIOM_BOOTSTRAP_PEERS="192.168.1.100:6000,192.168.1.101:6000,192.168.1.102:6000,192.168.1.103:6000,192.168.1.104:6000"
./target/release/axiom-node
```

### Custom Bootstrap via Environment
```bash
export AXIOM_BOOTSTRAP_PEERS="/ip4/YOUR_IP/tcp/6000"
./target/release/axiom-node
```

### Systemd Service (24/7 Production)
```bash
sudo cp contrib/axiom-guardian.service /etc/systemd/system/
sudo systemctl enable axiom-guardian
sudo systemctl start axiom-guardian
sudo journalctl -u axiom-guardian -f  # Watch logs
```

**Comprehensive Guide**: [docs/NETWORK_CONSENSUS.md](docs/NETWORK_CONSENSUS.md)

---

## 🛡️ Sovereign Guardian - Eternal Monitor

The **Guardian Sentinel** maintains network sovereignty 24/7:

```
Active Mode (Normal):        Deep Sleep Mode (Silence):
└─ 60s heartbeats           └─ 1h verification cycles
   ├─ Threat detection         ├─ 124M supply cap check
   ├─ Peer monitoring          ├─ Chain integrity verify
   └─ Network health           └─ Consensus validation
```

### What the Guardian Protects (Even During Complete Silence)
✅ **124M Supply Cap** - Verified every hour  
✅ **Chain Integrity** - Merkle roots checked automatically  
✅ **Peer Network** - 4+ nodes stay connected  
✅ **Consensus Rules** - No unauthorized forks  
✅ **Sovereignty** - Exit code 0 = "Sovereignty Maintained"  

### Guardian Logs
```
[14:24:01] 💚 Heartbeat | Supply: 124M | Idle: 1m | Mode: Active
[15:25:01] 🌙 Deep Sleep | Idle: 1h
[15:25:01] ✓ 124M supply maintained | ✓ Peers: 4/4 connected
[Shutdown] 🛑 SIGTERM received | Graceful shutdown complete.
```

---

## 💳 Wallet Operations

### Create & Manage Your Wallet
```bash
# Build wallet tool
cargo build --release --bin axiom-wallet

# View wallet
./target/release/axiom-wallet show
# Output: Address (hex): ba37f7d0a37a257d455f16b4f9d99ef37aba4a66...

# Check balance
./target/release/axiom-wallet balance
# Output: 💰 Balance: 250.00000000 AXM

# Send transaction
./target/release/axiom-wallet send <recipient> <amount> <fee>

# Backup wallet (CRITICAL!)
cp wallet.dat ~/wallet-backup-$(date +%Y%m%d).dat
chmod 600 wallet.dat
```

### Wallet Features
- 🔑 **Ed25519 Cryptography**: 32-byte keys (same as Solana, Cardano)
- 🔒 **Self-Custodial**: You control the private key
- 🛡️ **ZK-SNARK Privacy**: Balance never revealed on blockchain  
- 💾 **Single File**: wallet.dat (self-contained)
- ⚡ **Auto-Generated**: Created on first node run
- 🚫 **No Recovery**: Lost wallet = lost AXM (no centralized recovery)

### ⚠️ Security Critical
- **Never share** `wallet.dat`
- **Backup immediately**: `cp wallet.dat ~/backups/wallet-$(date +%Y%m%d).dat`
- **Lost wallet = lost AXM**: No recovery possible
- **Keep secure**: `chmod 600 wallet.dat`

---

## ⛏️ Mining & Economics

### Block Production
| Parameter | Value |
|-----------|-------|
| **Block Time** | 1800 seconds (30 minutes, VDF-enforced) |
| **Initial Reward** | 50 AXM per block |
| **Halving Interval** | 1,240,000 blocks (~70.7 years)  |
| **Total Supply** | 124,000,000 AXM (fixed, immutable) |
| **Pre-mine** | 0 AXM (100% earned through PoW) |

### Mining Economics
```
Era 1 (0-70y):    50 AXM/block → 62,000,000 total
Era 2 (70-141y):  25 AXM/block → 93,000,000 total
Era 3 (141-212y): 12.5 AXM/block → 108,500,000 total
...continuing... → 124,000,000 AXM maximum
```

### How Mining Works
1. **Wait for VDF Period**: 1800 seconds (30 min) time-lock
2. **Bundle Transactions**: Select up to 100 txs from mempool
3. **Hash & Prove**: Blake3 PoW hash with difficulty target
4. **Broadcast**: Gossipsub network propagates block
5. **Validate**: Peers verify consensus rules → accept/reject
6. **Reward**: 50 AXM to miner (halves every 1.24M blocks)

---

## 🔐 Privacy & Cryptography

### Mandatory Transaction Privacy
Every transaction includes:
- **Pedersen Commitments**: Hide transaction amounts
- **ElGamal Encryption**: Hide recipient identities
- **ZK-SNARK Proof**: Prove balance preservation without revealing values
- **Ed25519 Signature**: Authenticate transaction author

### Why This Matters
- No observer can see your balance or transactions
- Blockchain is auditable but unintelligible to outsiders
- You can prove transaction history only to those you choose (view keys)
- Anonymous by default, not opt-in

### Cryptographic Primitives
| Component | Algorithm | Curve | Purpose |
|-----------|-----------|-------|---------|
| **Signatures** | Ed25519 | - | Transaction authentication |
| **Commitments** | Pedersen | - | Hide amounts |
| **Encryption** | ElGamal | BLS12-381 | Hide recipients |
| **ZK-SNARK** | Groth16 | BLS12-381 | Prove correctness |
| **Hash (PoW)** | Blake3 | - | Mining target |
| **Hash (State)** | Blake3 | - | Block integrity |

---

## 🌐 Networking & Consensus

### Peer Discovery
- **mDNS**: Local network discovery (automatic)
- **DHT (Kademlia)**: Global peer discovery
- **Bootstrap Peers**: Explicit configuration for genesis phase
- **Gossipsub**: Efficient block/transaction propagation

### Network Requirements
- **Mainnet Bootstrap**: 34.10.172.20:6000 (always available)
- **Min Peers**: 2+ for regular nodes, 4/5 for genesis miners
- **Connection**: Automatic via libp2p Noise protocol (encrypted)
- **Firewall**: Port 6000/tcp must be accessible

### Split-Brain Prevention
All nodes validate:
1. **Identical Genesis Block**: Prevents different starting points
2. **Longest Chain**: Automatic fork resolution
3. **VDF Timestamps**: Tiebreaker for equal-length chains
4. **Bootstrap Peers**: Ensures all nodes find each other

**See**: [docs/NETWORK_CONSENSUS.md](docs/NETWORK_CONSENSUS.md) - Complete networking guide with recovery procedures

---

## 🏗️ Architecture Overview

```
User-Facing Layer
└─ Wallet (cli):    Create keys, sign transactions, check balance
   
Application Layer  
├─ Mining:         VDF timer → PoW solver → Block broadcast
├─ Networking:     libp2p P2P gossipsub propagation
├─ Mempool:        Pending transaction queue
├─ AI Security:    5-layer threat detection + anomaly prevention
└─ Guardian:       Eternal sentinel monitor (60s active / 1h sleep) + AI approval

Consensus Layer
├─ VDF:            Wesolowski proof (1800s sequencing)
├─ PoW:            Blake3 hash with difficulty adjustment
├─ Chain:          Timechain blocks + state management
└─ Validation:     ZK-SNARK proof verification

Storage Layer
├─ State:          Account balances (sled database)
├─ Blocks:         Blockchain history (bincode serialized)
└─ Config:         Genesis parameters (immutable)
```

### Core Modules
| Module | Purpose |
|--------|---------|
| `chain.rs` | Blockchain state, fork resolution, VDF validation |
| `block.rs` | Block structure, Blake3 PoW hashing |
| `transaction.rs` | Transaction definition, signature validation |
| `network.rs` | libp2p P2P, gossipsub, peer management |
| `guardian_sentinel.rs` | Eternal monitor, heartbeat scheduling |
| `network_config.rs` | Bootstrap configuration, peer discovery |
| `ai_core/` | 5-layer threat detection, anomaly analysis |
| `guardian_enhancement/` | AI-enforced consensus optimization, circuit breaker |
| `zk/` | ZK-SNARK circuit definitions (Groth16) |
| `vdf.rs` | VDF proof generation and verification |

---

## 📚 Documentation

### For First-Time Users
1. **[README.md](#)** (this file) - Architecture & features overview
2. **[docs/NETWORK_CONSENSUS.md](docs/NETWORK_CONSENSUS.md)** - Comprehensive networking guide

### For Node Operators
1. **[docs/NETWORK_CONSENSUS.md](docs/NETWORK_CONSENSUS.md)** - Comprehensive networking guide
2. **[contrib/axiom-guardian.service](contrib/axiom-guardian.service)** - Systemd service setup
3. **[SECURITY.md](SECURITY.md)** - Security policy

### For Developers
1. **[TECHNICAL_SPEC.md](TECHNICAL_SPEC.md)** - Implementation details
2. **[POW_SPECIFICATION.md](POW_SPECIFICATION.md)** - PoW algorithm specification
3. **[WHITEPAPER.md](WHITEPAPER.md)** - Complete technical specification
4. **[docs/ECONOMICS_TOKENOMICS.md](docs/ECONOMICS_TOKENOMICS.md)** - Supply economics

### For Protocol Researchers
1. **[WHITEPAPER.md](WHITEPAPER.md)** - Mathematical proofs (500+ pages)
2. **[docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md)** - Threat model & analysis
3. **[docs/124M-SOVEREIGN-SUPPLY-UPGRADE.md](docs/124M-SOVEREIGN-SUPPLY-UPGRADE.md)** - Supply cap design

---

## 🛠️ Building from Source

### Full Build
```bash
# Clone repository
git clone https://github.com/Ghost-84M/Axiom-Protocol.git
cd Axiom-Protocol

# Build release binary (optimized)
cargo build --release
./target/release/axiom-node

# Run tests
cargo test

# Check code quality
cargo clippy
cargo fmt --check
```

### Individual Components
```bash
# Build just the wallet tool
cargo build --release --bin axiom-wallet
./target/release/axiom-wallet show

# Build explorer
cd explorer && cargo build --release
./target/release/explorer

# Build PoW mining tool
cd pow-mining && cargo build --release
./target/release/pow-miner
```

---

## 📊 Node Status & Monitoring

### Check Node Status
```bash
# Full status
./target/release/axiom-node status

# Connected peers
./target/release/axiom-node peers

# Continuous monitoring
watch -n 5 './target/release/axiom-node status'
```

### Expected Output
```
╔══════════════════════════════════════╗
║ AXIOM NODE STATUS                    ║
╠══════════════════════════════════════╣
║ Height: 42                           ║
║ Connected Peers: 3/50                ║
║ Sync Status: IN SYNC ✅              ║
║ Balance: 1,050.00 AXM                ║
║ Mode: Active (60s heartbeats)        ║
╚══════════════════════════════════════╝
```

---

## 🚨 Troubleshooting

### Node Won't Start
```bash
# Check dependencies
rustc --version  # Should be 1.70+
cargo --version

# Check if port 6000 is in use
lsof -i :6000

# Change port (temporary)
AXIOM_PORT=6005 ./target/release/axiom-node
```

### Node Won't Sync
```bash
# Check connectivity
telnet 34.10.172.20 6000

# Check logs
tail -f ~/.axiom/logs.txt | grep -i sync

# Reset blockchain (re-syncs from scratch)
pkill axiom-node
rm -rf ~/.axiom/blocks/
./target/release/axiom-node
```

### Forked from Network (Different Chain)
```bash
# If node has different blocks than peers:
pkill axiom-node
rm -rf ~/.axiom/blocks/
./target/release/axiom-node
# Node will sync correct chain from bootstrap peer
```

**Full Troubleshooting**: [docs/NETWORK_CONSENSUS.md#troubleshooting](docs/NETWORK_CONSENSUS.md#troubleshooting)

---

## 🤝 Community & Contribution

- **Discord**: TBD
- **GitHub Issues**: [Report bugs](https://github.com/Ghost-84M/Axiom-Protocol/issues)
- **Security**: Report to security@axiom-protocol.io (PGP key in SECURITY.md)

---

## 📜 License

- **Protocol Code**: MIT License (full source available)
- **WHITEPAPER.md**: Academic publication (CC-BY-4.0)
- **Documentation**: CC-BY-4.0

---

## 🔗 Quick Links

| Link | Purpose |
|------|---------|
| [docs/NETWORK_CONSENSUS.md](docs/NETWORK_CONSENSUS.md) | Comprehensive networking |
| [TECHNICAL_SPEC.md](TECHNICAL_SPEC.md) | Implementation details |
| [WHITEPAPER.md](WHITEPAPER.md) | Complete specification |
| [POW_SPECIFICATION.md](POW_SPECIFICATION.md) | PoW algorithm |
| [docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md) | Security model |
| [CHANGELOG.md](CHANGELOG.md) | Release history |

---

## 🤖 AI Security System (v4.0.0)

The integrated 5-layer AI threat detection system enhances the Guardian sentinel with advanced anomaly detection and consensus optimization while maintaining all immutable constraints.

### 5-Layer Detection Architecture

**Layer 1: Statistical Anomaly Detection**
- Z-Score analysis for transaction outliers
- Modified Z-Score for robustness
- Interquartile Range (IQR) detection
- Mahalanobis distance for multivariate analysis

**Layer 2: Behavioral Pattern Analysis**
- Address reputation scoring
- Transaction sequence analysis
- Attack pattern matching (front-running, sybil attacks, etc.)
- Rapid transaction detection

**Layer 3: Threat Intelligence**
- Malicious address database
- Known attack signature database
- Threat escalation tracking
- Historical attack pattern matching

**Layer 4: Machine Learning Models**
- Isolation Forest for anomaly detection
- Local Outlier Factor (LOF) for density-based detection
- One-Class SVM for boundary detection
- DBSCAN clustering for network anomalies

**Layer 5: Temporal Pattern Analysis**
- Rapid-fire transaction detection
- Seasonal pattern analysis (future enhancement)
- Time-of-day anomaly detection (future enhancement)
- Temporal clustering analysis

### Guardian-AI Integration

The Guardian sentinel enforces AI decisions with mandatory validation:

- **Threat Scoring**: Combines all 5 layers into single 0-1 anomaly score
- **Guardian Gates**: MANDATORY approval for all parameter changes
- **Circuit Breaker**: Auto-activates on catastrophic threats (>0.95 score)
- **Emergency Override**: Manual intervention available via 2-of-3 validator signatures
- **Immutable Constraints**: Supply cap, block time, parameter bounds enforced regardless of AI

### Performance Metrics

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| CPU Overhead | < 4.5% | 3.2% | ✅ Exceeded |
| Memory Impact | < 170 MB | 165 MB | ✅ Exceeded |
| Transaction Latency | < 6.5 ms | 4.2 ms | ✅ Exceeded |
| Threat Detection | > 90% | 92.3% | ✅ Exceeded |
| False Positives | < 5% | 3.2% | ✅ Exceeded |

### Integration Points

The AI system integrates seamlessly at three key points:

1. **Transaction Validation** - AI threat scoring before acceptance
2. **Consensus Optimization** - PID controllers adjust parameters every 144 blocks
3. **Network Defense** - Circuit breaker activates on catastrophic threats

All integration points preserve immutable constraints and Guardian approval requirements.

---

## ⚖️ AXIOM vs Bitcoin

| Aspect | Bitcoin | AXIOM |
|--------|---------|-------|
| **Supply** | 21M | 124M |
| **Governance** | SegWit debates | None (math only) |
| **Privacy** | Optional (Mixers) | Mandatory (ZK-SNARKs) |
| **Block Time** | 10 min (variable) | 30 min (VDF-enforced) |
| **Consensus** | PoW only | VDF + PoW hybrid |
| **Scalability** | L2 solutions | Native privacy |
| **Premine** | None | None |

---

## 📈 Project Status

✅ **v4.0.0 Consolidated Release** - All v3.x features unified, production-ready for mainnet
✅ **Mainnet Live** - Active since February 2025  
✅ **Core Features** - VDF, PoW, ZK-SNARKs, 5-layer AI threat detection
✅ **Networking** - Modular Discv5/libp2p P2P with bootstrap nodes  
✅ **Guardian Sentinel** - 24/7 consensus enforcement + AI approval
✅ **AI Oracle** - Claude API integration for on-chain AI services
✅ **SDK & Explorer** - Developer SDK and full-stack block explorer
✅ **Bridge Contracts** - Cross-chain bridge infrastructure
✅ **Documentation** - Complete technical specification + deployment procedures
✅ **Performance** - All metrics exceeded (3.2% CPU, 165MB memory, 4.2ms latency)
✅ **Security** - All 11 critical/medium issues fixed
🔄 **Phase 2** - Cross-chain bridges (Q2 2026)  

---

**Version**: 4.0.0 (Consolidated Production Release)  
**Last Updated**: February 8, 2026  
**Status**: Production Mainnet + AI Security Ready  
**Network Health**: 4+ connected peers, fully synchronized, AI systems active

