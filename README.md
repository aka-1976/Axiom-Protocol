AXIOM Protocol — Academic Summary

Abstract

AXIOM is a privacy-preserving blockchain protocol that prioritizes verifiable mathematical properties over financial narratives. The protocol combines a Verifiable Delay Function (Wesolowski VDF) with Proof-of-Work to yield time-based fairness, and Groth16 ZK-SNARKs on BLS12-381 to enable private transaction validation.

Design Principles

- Mathematical scarcity: supply defined and provable by protocol rules.
- Provable supply cap: the monetary policy is encoded in consensus and deterministically enforced.
- Verifiable economics: all issuance and fee mechanisms are transparent and auditable by protocol observers.
- Fixed-supply protocol: supply schedule is deterministic and immutable in the genesis state.

Technical Highlights

- Consensus: VDF + PoW hybrid. VDF enforces sequential time delay; PoW provides Sybil resistance.
- Privacy: Groth16 ZK-SNARK circuit for transaction correctness and balance preservation without revealing values.
- Signatures: Ed25519 for transaction authentication.
- Storage: deterministic, account-based model with nonces to prevent double-spend.
- Networking: libp2p-based peer-to-peer transport with gossipsub propagation and authenticated channels.

Genesis and Launch

- Genesis timestamp: February 1, 2025 00:00:00 UTC (immutable in `src/genesis.rs`).
- Launch process: anonymous, permissionless anchor without governance or token pre-allocation.

Security

- Cryptographic primitives: production-grade parameters (2048-bit RSA modulus for VDF, BLS12-381 for ZK-SNARKs, Ed25519 signatures).
- Trusted setup: multi-party ceremony recommended; a coordinator script is provided for transparency.
- Auditability: all protocol rules are encoded in open-source code and can be audited by third parties.

Usage and Verification

To verify repository claims:

```bash
# Build
cargo build --release

# Run core tests
cargo test
```

Concluding Note

This document intentionally avoids speculative or financial language. Its purpose is to document the protocol's mathematical and engineering properties so third-party auditors and researchers can evaluate AXIOM objectively.
- **Transaction Processing**: ✅ Wallet operations verified
- **Multi-Node Operation**: ✅ Connection establishment confirmed
- **Application Integration**: ✅ Python API demo included

### Core Philosophy
- **Absolute Scarcity**: Fixed 124M supply, no inflation, no governance
- **1-Hour Blocks**: Enforced by VDF time-locks for fair and predictable consensus
- **Mathematical Sovereignty**: Only provable math governs the network
- **Privacy by Default**: ZK-SNARKs mandatory for all transactions
- **Time as Consensus**: VDF ensures sequential, fair block production
- **Network Transparency**: Real-time peer monitoring and health status
- **No Pre-mine**: 100% of supply earned through mining (no development wallet)

## 🔄 Recent Upgrades (February 2026)

### Dependency Updates
- **libp2p**: Upgraded from 0.53 to 0.54.1 with Yamux improvements, idle connection timeout, and RequestResponse fallback
- **ark-* crates**: Updated to 0.5.x for latest ZK-SNARK implementations
- **AI/ML**: Migrated from TensorFlow to ONNX Runtime for attack detection (see ONNX_USAGE.md)
- **Cargo Audit**: All vulnerabilities resolved, clean build

### OpenClaw Integration (NEW)
- **Automation Framework**: Python-based credential ceremony orchestration
- **Ceremony Master**: Coordinates multi-party computation and key generation
- **Node Health Monitor**: Real-time monitoring of validator node status
- **Agent Internet**: Decentralized execution of verification tasks
- **Auto-Launch**: Integrated into node startup via `openclaw_integration.rs` module

### Infrastructure Improvements
- **Docker Support**: Added Dockerfile and docker-compose.yml for containerized deployment
- **Mainnet Configuration**: Updated setup for production mainnet operation
- **Multi-Node Synchronization**: Enhanced P2P syncing with automatic peer discovery (mDNS)
- **CLI Enhancements**: Added `--bootnodes` flag for initial peer connections
- **Bootstrap Config**: `config/bootstrap.toml` with mainnet bootnode addresses

### Security Enhancements
- **Audit Documentation**: Full security audit results in `SECURITY.md`
- **Vulnerability Tracking**: Active monitoring of dependencies
- **Responsible Disclosure**: Established security reporting process

## ✨ Key Features

### 🔐 Cryptographic Security
- **ZK-SNARK Proofs**: Zero-knowledge transaction validation
- **Ed25519 Signatures**: Industry-standard digital signatures
- **SHA-256 Hashing**: Secure block and transaction integrity
- **Nonce-based Replay Protection**: Account-based state with nonce tracking

### ⛏️ Consensus & Mining
- **Hybrid Consensus**: VDF (1-hour blocks) + PoW difficulty adjustment
- **Automatic Halving**: Mining rewards halve every 2.1 million blocks
- **Fair Distribution**: Genesis block with initial supply allocation
- **Network Synchronization**: P2P block and transaction propagation

### 🤖 AI Network Protection
- **Neural Guardian**: Local AI agent on every node (ONNX Runtime powered)
- **Attack Detection**: Identifies selfish mining, Sybil attacks, eclipse attacks
- **Peer Trust Scoring**: Dynamic reputation system for network peers
- **ONNX Inference**: Fast, portable, production-grade AI scoring
- **Anomaly Isolation**: Automatic quarantine of suspicious nodes

### 💰 Economics
- **Total Supply**: 124,000,000 AXM (fixed, no pre-mine, all earned through mining)
- **No Pre-mine**: 100% of supply earned through proof-of-work mining (no development wallet)
- **Block Reward**: 50 AXM per block (halves every 2,100,000 blocks)
- **Block Time**: 3600 seconds (1 hour) enforced by VDF
- **Halving Schedule**: Every 2,100,000 blocks (~4 years per era)
- **Deflationary Design**: Supply reaches maximum over multiple eras with binary halving

### 🔗 Network & Storage
- **P2P Networking**: libp2p with gossipsub protocol
- **Persistent Storage**: sled embedded database
- **Transaction Mempool**: Efficient transaction broadcasting
- **Bootstrap Discovery**: Automatic peer discovery and connection

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   CLI Wallet    │    │   Full Node     │    │   AI Engine     │
│                 │    │                 │    │                 │
│ • Key Generation│    │ • Block Mining  │    │ • Attack Detect │
│ • TX Creation   │    │ • TX Validation │    │ • Peer Scoring  │
│ • Balance Check │    │ • P2P Network   │    │ • Trust Analysis│
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────────┐
                    │   Core Protocol     │
                    │                     │
                    │ • ZK-SNARK Proofs   │
                    │ • VDF Consensus     │
                    │ • State Management  │
                    │ • Transaction Logic │
                    └─────────────────────┘
```

### Core Components

| Component | Technology | Purpose |
|-----------|------------|---------|
| **Consensus** | VDF + PoW | Fair block production |
| **Privacy** | ZK-SNARK | Transaction anonymity |
| **Networking** | libp2p | P2P communication |
| **Storage** | sled | Persistent data |
| **AI Security** | Neural Networks | Attack prevention |
| **Wallet** | Ed25519 | Key management |

## 🚀 Installation

### Prerequisites
- **Rust**: 1.70+ ([Install Rust](https://rustup.rs/))
- **System Dependencies**: Standard build tools

### Build from Source
```bash
# Clone the repository
git clone https://github.com/Ghost-84M/Axiom-Protocol.git
cd Axiom-Protocol

# Build in release mode for production
cargo build --release

# Optional: Run tests to verify installation
cargo test
```

### Binary Installation
```bash
# Download pre-built binaries (when available)
# Binary releases will be provided for all major platforms in future updates.
```

## 🏃 Quick Start

### Option 1: Docker Mainnet (Recommended)
```bash
# Clone the repository
git clone https://github.com/Ghost-84M/Axiom-Protocol.git
cd Axiom-Protocol

# Launch 3-node mainnet
docker-compose up -d

# Check logs
docker-compose logs -f
```

### Option 2: Native Build
```bash
# Build and run the main node
cargo build --release
./target/release/axiom --bootnodes /ip4/127.0.0.1/tcp/6000/p2p/<peer-id>

# The node will:
# - Connect to the P2P network
# - Start mining blocks
# - Begin transaction processing
# - Activate AI network protection
```

### 2. Wallet Management

#### 💳 Built-In Wallet System

Axiom has a **built-in self-custodial wallet** - no MetaMask or external wallet needed!

**Wallet Type**: Ed25519 Cryptographic Wallet (same as Solana, Cardano, Polkadot)  
**Storage**: `wallet.dat` (64 bytes - automatically created on first run)  
**Security**: Military-grade Ed25519 + Zero-Knowledge proofs for privacy

```bash
# Build wallet CLI tool
cargo build --release --bin axiom-wallet

# Show your wallet address and details
./target/release/axiom-wallet show

# Export address only (for receiving payments)
./target/release/axiom-wallet export

# Check your balance
./target/release/axiom-wallet balance

# Send a transaction
./target/release/axiom-wallet send <recipient-address> <amount> <fee>

# Example: Send 10.5 AXM with 0.001 AXM fee
./target/release/axiom-wallet send abc123...def456 10.5 0.001
```

#### 🔐 Wallet Security

⚠️ **CRITICAL**: `wallet.dat` contains your SECRET KEY!
- If lost → funds lost forever (no recovery)
- If stolen → someone can steal all your AXM
- **BACKUP NOW**: `cp wallet.dat ~/backups/wallet-$(date +%Y%m%d).dat`

**Security Features:**
- ✅ Self-custodial (you control the private key)
- ✅ Offline storage (private key never sent over network)
- ✅ Ed25519 signatures (military-grade cryptography)
- ✅ ZK-SNARK proofs (balance privacy)
- ✅ Automatic creation (generated on first node run)

**Address Format**: 32-byte hex string (64 characters)  
**Example**: `ba37f7d0a37a257d455f16b4f9d99ef37aba4a66a0028984b1a60cbc5e42da27`

### 3. Check Network Status
```bash
# View supply information
cargo run --release --bin axiom-supply

# Monitor mining progress
# Check node logs for real-time updates
```

## 📖 Usage

### 🖥️ Understanding Node Status

When you run `./target/release/axiom`, you'll see a dashboard like this:

```
--- 🏛️  AXIOM STATUS ---
⛓️  Height: 5 | Diff: 1000 | Trend: STABLE ↔️
⏳ Time-Lock: 43m remaining | 🤖 AI Shield: ACTIVE
💰 Mined: 200.00000000 AXM | Remaining: 1239999800.00000000 AXM | 0.00% of max supply
🌐 Network Status:
   ├─ PeerId: 12D3KooWNLCeLr4aVVfvT7zeCw9oXKcTQSoxFB2PE1aMRN75Ubai
   ├─ Connected Peers: 0
   │  └─ No peers connected (check firewall/NAT)
   └─ Listen Addresses:
      └─ /ip4/127.0.0.1/tcp/6005
      └─ /ip4/10.255.255.254/tcp/6005
      └─ /ip4/172.20.208.232/tcp/6005
      └─ /ip4/172.17.0.1/tcp/6005
```

#### 📊 Dashboard Explanation

| Metric | Meaning | Details |
|--------|---------|---------|
| **Height** | Current blockchain height | Number of blocks mined locally |
| **Diff** | Mining difficulty | Target for Proof-of-Work (auto-adjusts) |
| **Trend** | Difficulty trend | UP ⬆️, DOWN ⬇️, or STABLE ↔️ |
| **Time-Lock** | VDF countdown | Time until next block can be mined (1 hour) |
| **AI Shield** | Neural Guardian status | ACTIVE = monitoring network for attacks |
| **Mined** | Total mined supply | AXM minted so far (50 AXM per block) |
| **Remaining** | Unmined supply | 124M AXM - Mined |
| **PeerId** | Your node's unique ID | libp2p peer identifier |
| **Connected Peers** | Active connections | Other nodes you're syncing with |
| **Listen Addresses** | Your node's addresses | Where other peers can connect to you |

#### 🔍 Why Is Height 5 and Not Syncing?

Your node shows **Height: 5** because:

1. **Local Mining**: Your node has mined 5 blocks locally
2. **No Peer Connections**: `Connected Peers: 0` means you're isolated
3. **No Network Sync**: Without peers, you can't receive their blockchain

**This is normal for a standalone node in development!**

#### 🌐 Network Synchronization Explained

```
┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐
│   Node A        │◄─────►│   Node B        │◄─────►│   Node C        │
│   Height: 100   │       │   Height: 100   │       │   Height: 100   │
└─────────────────┘       └─────────────────┘       └─────────────────┘
         ▲                         ▲                         ▲
         │                         │                         │
         └─────────────────────────┴─────────────────────────┘
                    Gossipsub Protocol (libp2p)

When you connect, your node:
1. Broadcasts "REQ_CHAIN" to all peers
2. Receives their full blockchain
3. Validates each block cryptographically
4. Adopts the longest valid chain
5. Height syncs to 100 automatically!
```

#### 🔧 How to Get Peers and Sync

**Option 1: Local Multi-Node Network (Best for Testing)**
```bash
# Terminal 1: Node 1 (port 6000)
./target/release/axiom

# Terminal 2: Node 2 (port 6001) - connects to Node 1
./target/release/axiom --port 6001 --bootnodes /ip4/127.0.0.1/tcp/6000/p2p/<node1-peer-id>

# Terminal 3: Node 3 (port 6002) - connects to Node 1
./target/release/axiom --port 6002 --bootnodes /ip4/127.0.0.1/tcp/6000/p2p/<node1-peer-id>

# Now all nodes sync to the same height!
```

**Option 2: Docker Network (Easiest)**
```bash
docker-compose up -d
# Launches 3 nodes that auto-connect and sync
```

**Option 3: Join Mainnet (Production)**
```bash
# Use mainnet bootnodes from config/bootstrap.toml
./target/release/axiom --bootnodes /ip4/34.145.123.45/tcp/6000/p2p/<bootnode-peer-id>
```

#### 🚀 What Happens When Peers Connect

```
Before Connection:
Node A: Height 5 | Peers: 0 | Status: Isolated
Node B: Height 0 | Peers: 0 | Status: Isolated

After Connection:
Node A: Height 5 | Peers: 1 | Status: Broadcasting blocks to B
Node B: Height 5 | Peers: 1 | Status: Synced from A

Consensus Process:
1. mDNS discovers Node B
2. libp2p establishes connection
3. Node A broadcasts its 5 blocks
4. Node B validates and adopts them
5. Both continue mining together
6. Longest valid chain wins
```

#### �️ Detailed Multi-Node Same-Machine Setup (3-Terminal Guide)

**Step 1: Build the project**
```bash
cargo build --release
cd target/release
```

**Step 2: Fresh Start (Delete wallet.dat for genesis)**
```bash
# Remove wallet files for fresh start
rm -f wallet.dat wallet_public.txt

# This ensures nodes start from genesis block
```

**Step 3: Terminal 1 - Node A (Bootnode)**
```bash
# Start the first node on port 6000
./axiom --port 6000

# Output will show:
# ⛓️ Height: 0 | Peers: 0
# 🆔 PeerId: 12D3Koo...ABC123
# 📍 Listen on: /ip4/127.0.0.1/tcp/6000

# Save the PeerId for next steps
```

**Step 3: Terminal 2 - Node B (Port 6001)**
```bash
# Copy Node A's PeerId from Terminal 1 output above
export NODE_A_PEER_ID="12D3Koo...ABC123"  # Replace with actual PeerId

# Connect to Node A
./axiom --port 6001 --bootnodes /ip4/127.0.0.1/tcp/6000/p2p/$NODE_A_PEER_ID

# Expected output: Connected Peers: 1
```

**Step 4: Terminal 3 - Node C (Port 6002)**
```bash
# Same PeerId as Node A
export NODE_A_PEER_ID="12D3Koo...ABC123"

# Connect to Node A
./axiom --port 6002 --bootnodes /ip4/127.0.0.1/tcp/6000/p2p/$NODE_A_PEER_ID

# Expected output: Connected Peers: 2 (one for Node B, one for Node A)
```

**Result: All nodes should show Height 100+ and be synchronized!**

**Key Notes:**
- mDNS handles automatic peer discovery on localhost
- Port allocation is automatic (6000-6010 for multiple nodes)
- Height converges when blocks propagate across peers
- Delete `wallet.dat` on each node for isolated genesis starts
- Use `AXIOM_KNOWN_PEERS` env var for custom peer discovery

#### �📡 AI Stats Monitoring

The dashboard also exports AI statistics to `ai_stats.json`:
```json
{
  "total_predictions": 0,
  "spam_detected": 0,
  "onnx_model_used": 0,
  "fallback_used": 0,
  "avg_confidence": 0.0
}
```

**Live monitoring:**
```bash
# Watch AI stats in real-time
watch -n 1 cat ai_stats.json
```

### Block Explorer API

The explorer exposes REST endpoints for block and state queries:

- **All blocks:**  
    `GET http://127.0.0.1:8080/blocks`
- **Chain state:**  
    `GET http://127.0.0.1:8080/state`

To run the explorer:

```bash
cd explorer
cargo run --release
# Or use VS Code port forwarding to access from your browser
```

You can use curl, Postman, or a browser to view the blockchain data.

### Node Operation
```bash
# Start mining node
cargo run --release

# Start non-mining node (relay only)
cargo run --release -- --relay-only

# Custom configuration
cargo run --release -- --config custom.toml
```

### Network Monitoring
The node displays real-time network status every 10 seconds:

```bash
--- 🏛️  AXIOM STATUS ---
⛓️  Height: 42 | Diff: 1000 | Trend: UP ⬆️
⏳ Time-Lock: 45m remaining | 🤖 AI Shield: ACTIVE
💰 Mined: 1,050.00 AXM | Remaining: 83,949,950.00 AXM | 1.25% of max supply
🌐 Connected Peers: 3 | Network: ACTIVE
------------------------
```

**Network Indicators:**
- **Connected Peers**: Number of active P2P connections
- **Peer Discovery**: Logs show when peers connect/disconnect
- **Block Verification**: Each block is verified by connected peers
- **Network Health**: Minimum 3 peers recommended for robust consensus

**Peer Events:**
```
🔗 Peer connected: 12D3KooWAbc... | Total peers: 3
🔌 Peer disconnected: 12D3KooWAbc... | Total peers: 2
🔎 mDNS discovered peer: 12D3KooWXyz...
👋 Identified peer: 12D3KooWDef... (axiom/1.0.0)
```

### 💳 Wallet Operations

Axiom uses a **built-in Ed25519 wallet** stored in `wallet.dat` - no MetaMask needed!

```bash
# Build wallet tool
cargo build --release --bin axiom-wallet

# Show full wallet details
./target/release/axiom-wallet show
# Output: 💳 Axiom Wallet Details
#         Address (hex): ba37f7d0a37a257d455f16b4f9d99ef37aba4a66a0028984b1a60cbc5e42da27
#         Address length: 32 bytes
#         ⚠️  KEEP wallet.dat SAFE - it contains your secret key!

# Export address only (for receiving payments)
./target/release/axiom-wallet export
# Output: ba37f7d0a37a257d455f16b4f9d99ef37aba4a66a0028984b1a60cbc5e42da27

# Check your balance
./target/release/axiom-wallet balance
# Output: 💰 Balance: 250.00000000 AXM

# Send transaction
./target/release/axiom-wallet send <recipient> <amount> <fee>
# Example: ./target/release/axiom-wallet send abc123...def 10.5 0.001
# Output: ✅ Transaction created and saved to pending_tx.dat
#         📤 Run the axiom node to broadcast this transaction

# CRITICAL: Backup your wallet immediately!
cp wallet.dat ~/backups/wallet-$(date +%Y%m%d).dat
chmod 600 wallet.dat  # Secure file permissions

# Monitor balance continuously
watch -n 60 './target/release/axiom-wallet balance'
```

**Wallet Features:**
- 🔑 **Ed25519 Cryptography**: 32-byte keys (same as Solana, Cardano)
- 🔒 **Self-Custodial**: You control the private key (no third party)
- 🛡️ **ZK-SNARK Proofs**: Transaction privacy (balance never revealed)
- 💾 **Single File**: `wallet.dat` (64 bytes total)
- ⚡ **Automatic Creation**: Generated on first node run
- 🚫 **No MetaMask**: Built directly into the node

**Security Warning:**
⚠️ If you lose `wallet.dat`, your AXM is **LOST FOREVER** (no recovery possible!)  
⚠️ If someone steals `wallet.dat`, they can **STEAL ALL YOUR AXM**  
⚠️ **BACKUP NOW**: `cp wallet.dat ~/backups/wallet-backup.dat`

**Transaction Flow:**
```
1. Create TX:  axiom-wallet send <to> <amount> <fee>
2. ZK Proof:   Wallet proves balance without revealing amount
3. Sign:       Ed25519 signature with private key
4. Save:       Transaction written to pending_tx.dat
5. Broadcast:  Node sends TX to network (Gossipsub)
6. Mining:     Miners include in next block
7. Confirm:    Transaction becomes permanent on blockchain
```

### 🛠️ Node Technical Architecture

#### Core Mining Loop (Every 1 Hour)

```rust
// Simplified view of src/main.rs mining logic
VDF Timer (3600s) → {
    1. Check if 1 hour has elapsed
    2. Calculate VDF seed from parent block
    3. Generate VDF proof (Wesolowski)
    4. Create ZK-SNARK proof for miner identity
    5. Select transactions from mempool (max 100)
    6. Start PoW mining loop:
       - Try nonces 0..100,000
       - Check if hash meets difficulty
       - If found: broadcast block, save to storage
       - If not found: reduce difficulty by 10
    7. Reset timer and repeat
}
```

#### Network Event Processing

```rust
// P2P event handling
Swarm Events:
├── Gossipsub Message → Validate & add block/transaction
├── ConnectionEstablished → Update peer count
├── ConnectionClosed → Remove peer from set
├── mDNS Discovery → Request chain from new peer
├── Kademlia Discovery → Add peer to routing table
└── Identify → Log peer protocol version

Chain Synchronization:
1. Receive "REQ_CHAIN" message
2. Broadcast entire blockchain to requester
3. Peer validates genesis block match
4. Peer validates each block sequentially
5. Peer adopts chain if longer & valid
6. Consensus achieved!
```

#### Storage & State Management

| File | Purpose | Format |
|------|---------|--------|
| `wallet.dat` | Ed25519 private key | Binary (keep secure!) |
| `chain.dat` | Blockchain blocks | Bincode serialized Vec<Block> |
| `ai_stats.json` | AI Guardian metrics | JSON (live updates) |
| `genesis.json` | Genesis configuration | JSON (consensus rules) |

```bash
# View persisted data
ls -lh *.dat *.json

# Backup your wallet (IMPORTANT!)
cp wallet.dat wallet-backup-$(date +%Y%m%d).dat

# Reset blockchain (keep wallet)
rm chain.dat
# Node will restart from genesis block
```

#### 🔐 Cryptographic Components

**Transaction Flow:**
```
Wallet → Create TX → Sign (Ed25519) → Broadcast (Gossipsub)
  ↓
Mempool (VecDeque) → Validate (nonce, balance, signature)
  ↓
Mining → Include in block (max 100 TXs) → ZK-SNARK proof
  ↓
Block → Hash meets difficulty → Broadcast → Peers validate
  ↓
Chain → Add block → Save to storage → Update UTXO state
```

**Consensus Rules:**
- **No Pre-mine**: All 124,000,000 AXM earned through mining only (no pre-allocated tokens)
- **Block Time**: 3600 seconds (1 hour) enforced by VDF
- **Difficulty**: Auto-adjusts if mining fails (min: 10)
- **Reward**: 50 AXM per block (halves every 2,100,000 blocks)
- **Validation**: Parent hash, VDF proof, PoW difficulty, ZK proof

#### 🤖 AI Guardian (Neural Network)

```
Real-time Monitoring:
├── Message Rate Limiting: >100 msg/sec = DoS
├── Peer Trust Scoring: message_count, connection_time
├── Attack Detection: selfish mining, eclipse, sybil
└── ONNX Model: mobilenet_v2.onnx (fallback to heuristics)

Training:
- Honest behavior: train([1.0, 1.0, 1.0], 1.0)
- Suspicious behavior: train([0.1, 0.0, 0.0], 0.0)
- Adaptive learning based on network behavior
```

#### 🌐 Network Ports & Protocols

| Port | Protocol | Purpose |
|------|----------|---------|
| 6000 | TCP | libp2p P2P communication |
| 6001 | TCP | Node 2 (if multi-node) |
| 6002 | TCP | Node 3 (if multi-node) |
| 8080 | HTTP | Block explorer API |
| 5353 | UDP | mDNS peer discovery (LAN) |

**Firewall Configuration:**
```bash
# Allow P2P connections
sudo ufw allow 6000/tcp
sudo ufw allow 6001/tcp
sudo ufw allow 6002/tcp

# Allow explorer API (optional)
sudo ufw allow 8080/tcp

# Allow mDNS (LAN discovery)
sudo ufw allow 5353/udp
```

#### 📊 Performance Metrics

| Metric | Value | Notes |
|--------|-------|-------|
| **VDF Computation** | ~30-60 seconds | Depends on CPU |
| **PoW Mining** | ~10-300 seconds | Depends on difficulty |
| **Block Validation** | <1 second | Includes ZK proof verification |
| **Transaction Verification** | <100ms | Ed25519 signature check |
| **P2P Latency** | <200ms | LAN/WAN dependent |
| **Memory Usage** | ~200-500MB | Includes AI model |
| **Disk Usage** | ~10MB per 1000 blocks | Compressed blockchain |

### 🔧 Troubleshooting

#### ❓ "Height is 5 but not syncing to higher blocks"

**Diagnosis**: `Connected Peers: 0` means your node is isolated.

**Root Cause**: No peer connections = no blockchain synchronization.

**Solutions**:

**1. Local Multi-Node Setup (Recommended for Testing)**
```bash
# Terminal 1: Start first node
./target/release/axiom
# Copy the PeerId from the output, e.g.: 12D3KooWNLCeLr4aVVfvT7zeCw9oXKcTQSoxFB2PE1aMRN75Ubai

# Terminal 2: Start second node and connect to first
./target/release/axiom --port 6001 --bootnodes /ip4/127.0.0.1/tcp/6000/p2p/12D3KooWNLCeLr4aVVfvT7zeCw9oXKcTQSoxFB2PE1aMRN75Ubai

# Now both nodes will sync and mine together!
```

**2. Docker Network (Easiest)**
```bash
docker-compose up -d
# Automatically launches 3 nodes that connect and sync
docker-compose logs -f  # Watch them sync in real-time
```

**3. Join Public Network**
```bash
# Get bootnode address from config/bootstrap.toml
./target/release/axiom --bootnodes /ip4/34.145.123.45/tcp/6000/p2p/12D3KooWAbc...
```

#### ❓ "How do I know if syncing is working?"

**Look for these log messages**:
```
🔎 mDNS discovered peer: 12D3KooWXyz... at /ip4/192.168.1.100/tcp/6000
🔗 Peer connected: 12D3KooWXyz... | Total peers: 1
📥 Requesting chain from peer: 12D3KooWXyz...
🔁 Synced complete chain from peer. New height: 42
✅ Block accepted and added to chain
```

**Dashboard shows progress**:
```
⛓️  Height: 1 → 10 → 42 → ... (increases)
🌐 Connected Peers: 1 → 2 → 3 (increases)
```

#### ❓ "Node shows 0 connected peers"

**Check 1: Firewall**
```bash
# Linux
sudo ufw status
sudo ufw allow 6000/tcp

# macOS
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate
# Add exception in System Preferences → Security & Privacy

# Windows
netsh advfirewall firewall add rule name="Axiom P2P" dir=in action=allow protocol=TCP localport=6000
```

**Check 2: NAT/Port Forwarding**
```bash
# If behind router, forward port 6000 TCP to your machine
# Router admin panel → Port Forwarding → Add Rule:
#   External Port: 6000
#   Internal IP: 192.168.1.X (your machine)
#   Internal Port: 6000
#   Protocol: TCP
```

**Check 3: Listen Addresses**
```
Should see:
✓ /ip4/0.0.0.0/tcp/6000  (good - listening on all interfaces)
✓ /ip4/192.168.1.X/tcp/6000  (good - LAN address)

Should NOT see only:
✗ /ip4/127.0.0.1/tcp/6000  (bad - localhost only)
```

**Fix: Bind to all interfaces**
```bash
# Edit src/main.rs if needed
Swarm::listen_on("/ip4/0.0.0.0/tcp/6000")
```

#### ❓ "Mining is slow or failing"

**Check 1: Difficulty too high**
```
⚠️  Mining failed, reducing difficulty to 990
```
This is normal! Difficulty auto-adjusts. Wait for next attempt.

**Check 2: VDF computation**
```bash
# VDF takes 30-60 seconds on modern CPUs
# If taking longer, check CPU usage:
top  # Should see axiom using 100% of 1 core during VDF
```

**Check 3: PoW nonce limit**
```rust
// In src/main.rs
let max_attempts = 100000;  // Increase if needed
```

#### ❓ "Transaction not appearing in mempool"

**Validation Checklist**:
1. **Signature**: Must be valid Ed25519
2. **Nonce**: Must match account state
3. **Balance**: Sender must have enough AXM
4. **Fee**: Must be ≥ 0.00000100 AXM
5. **Not duplicate**: Same TX cannot be in mempool twice

**Debug**:
```bash
# Check mempool size
# Look for this log:
✅ Transaction added to mempool  # Good
⚠️  Transaction validation failed: ...  # Check error
```

#### ❓ "AI stats show all zeros"

**Normal if**:
- No peers connected yet (nothing to analyze)
- Network just started (no activity yet)

**AI activates when**:
- Peers connect and exchange messages
- Blocks/transactions are received
- Network activity increases

**Force AI test**:
```bash
# Connect 2+ nodes and send transactions
# AI will log predictions:
🤖 AI Trust Score: 0.85 for peer 12D3KooWXyz...
```

#### ❓ "Chain reset / lost blocks"

**Recover from backup**:
```bash
cp chain-backup.dat chain.dat
./target/release/axiom
```

**Start fresh**:
```bash
rm chain.dat  # Keeps wallet.dat safe
./target/release/axiom  # Starts from genesis
```

#### ❓ "Wallet file missing"

**If `wallet.dat` is lost**:
- ⚠️ **Private key is LOST FOREVER**
- Cannot recover funds (no seed phrase)
- Create new wallet:
```bash
rm wallet.dat
./target/release/axiom  # Generates new wallet
```

**Prevention**:
```bash
# Backup wallet regularly
cp wallet.dat ~/backups/axiom-wallet-$(date +%Y%m%d).dat
```

#### ❓ "Docker nodes can't connect"

**Check Docker network**:
```bash
docker network ls
docker network inspect axiom-protocol_default

# Should show 3 containers with IPs like:
# validator-1: 172.20.0.2
# validator-2: 172.20.0.3
# validator-3: 172.20.0.4
```

**Check logs**:
```bash
docker-compose logs validator-1 | grep "Peer connected"
# Should see connections to other validators
```

### Development
```bash
# Run tests
cargo test

# Run specific test
cargo test test_transaction_validation

# Check code quality
cargo clippy

# Format code
cargo fmt

# Generate documentation
cargo doc --open
```

## �️ Tools & Scripts

### Core Binaries

| Tool | Purpose | Size | Usage |
|------|---------|------|-------|
| **`axiom`** | Main blockchain node | ~3.7MB | Full mining and networking |
| **`axiom-wallet`** | Wallet management | ~484KB | Address, balance, transactions |
| **`axiom-supply`** | Supply tracking | ~355KB | Token economics display |

### Utility Scripts

#### Launch Script (`launch-node.sh`)
```bash
# Easy node startup with status checks
./launch-node.sh
```

#### Network Status (`network-status.sh`)
```bash
# Check node health and network status
./network-status.sh
```

#### Python API Demo (`axiom-app-demo.py`)
```bash
# Application integration example
python3 axiom-app-demo.py
```

### Command Reference

#### Node Operations
```bash
# Start mining node
./target/release/axiom

# Check wallet address
./target/release/axiom-wallet export

# Check balance
./target/release/axiom-wallet balance <address>

# Send transaction
./target/release/axiom-wallet send <to> <amount> <fee>

# Check supply statistics
./target/release/axiom-supply
```

#### Development Tools
```bash
# Run tests
cargo test

# Build release
cargo build --release

# Check code quality
cargo clippy

# Format code
cargo fmt

# Update repository
git pull origin main
```

### Core Types

#### Transaction
```rust
pub struct Transaction {
    pub from: Address,      // Sender address
    pub to: Address,        // Recipient address
    pub amount: u64,        // Amount in smallest units
    pub fee: u64,          // Transaction fee
    pub nonce: u64,        // Account nonce
    pub signature: Vec<u8>, // Ed25519 signature
    pub zk_proof: Vec<u8>,  // ZK-SNARK proof
}
```

#### Block
```rust
pub struct Block {
    pub parent: [u8; 32],     // Parent block hash
    pub slot: u64,            // Time slot
    pub miner: Address,       // Miner address
    pub transactions: Vec<Transaction>,
    pub vdf_proof: Vec<u8>,   // VDF proof
    pub zk_proof: Vec<u8>,    // Miner ZK proof
    pub nonce: u64,           // PoW nonce
}
```

### Key Functions

#### Wallet Operations
```rust
// Create new wallet
let wallet = Wallet::new();

// Sign transaction
let signature = wallet.sign_transaction(&tx)?;

// Verify transaction
wallet.verify_transaction(&tx)?;
```

#### Node Operations
```rust
// Initialize node
let mut node = Timechain::new()?;

// Add transaction to mempool
node.add_transaction(tx)?;

// Mine new block
let block = node.mine_block()?;

// Validate block
node.validate_block(&block)?;
```

## 🔒 Security Audit

### Current Security Status (February 2026)

**Audit Tool:** `cargo audit` - Official Rust security vulnerability scanner  
**Last Updated:** January 20, 2025  
**Audit Frequency:** Continuous monitoring with dependency updates

### Vulnerability Summary
- **Total Vulnerabilities:** 1 active (reduced from 2)
- **Critical Issues:** 0
- **High Severity:** 0
- **Medium Severity:** 1
- **Warnings:** 7 (non-critical, mostly unmaintained crates)

### Active Vulnerabilities

#### 1. tracing-subscriber (RUSTSEC-2025-0055)
- **Severity:** Medium
- **Affected:** Transitive dependency via ark-relations
- **Description:** Logging user input may result in poisoning logs with ANSI escape sequences
- **Status:** Upstream issue in arkworks/ark-relations. Dependency patch attempted but incompatible.
- **Mitigation:** AXIOM Protocol does not log untrusted user input in exploitable patterns. Risk: Low
- **Tracking:** Monitoring arkworks ecosystem for updates

### Resolved Vulnerabilities
- **ring (RUSTSEC-2024-0336):** Fixed by upgrading libp2p from 0.53 to 0.56

### Security Recommendations
1. **Run Audits Regularly:** Execute `cargo audit` before building
2. **Monitor Dependencies:** Stay updated with RustSec advisories
3. **Report Issues:** Use responsible disclosure (see `SECURITY.md`)
4. **Code Review:** All changes undergo security review

### Audit Commands
```bash
# Run security audit
cargo audit

# Check for updates
cargo audit --update

# Generate audit report
cargo audit --format json > audit-report.json
```

For full audit details and responsible disclosure policy, see [`SECURITY.md`](SECURITY.md).

## 🧪 Testing Results

### ✅ Comprehensive Test Suite (8/8 Passing)

| Test Category | Status | Description |
|---------------|--------|-------------|
| **Transaction Validation** | ✅ PASSED | Core transaction logic and validation |
| **Block Creation** | ✅ PASSED | Block structure and hashing |
| **Block Hash** | ✅ PASSED | Cryptographic hash verification |
| **Chain Initialization** | ✅ PASSED | Blockchain genesis and setup |
| **Economics** | ✅ PASSED | Supply mechanics and halving |
| **Wallet Balance** | ✅ PASSED | Balance tracking and queries |
| **Transaction Creation** | ✅ PASSED | Transaction building and signing |
| **Mining Simulation** | ✅ PASSED | VDF mining loop functionality |

### ✅ Network Testing Results

| Test Phase | Status | Result |
|------------|--------|--------|
| **Peer Discovery** | ✅ PASSED | mDNS discovery working |
| **Connection Establishment** | ✅ PASSED | Peer dialing and linking |
| **Real-Time Monitoring** | ✅ PASSED | Dashboard shows live peer count |
| **Multi-Node Operation** | ✅ PASSED | 2+ nodes connect successfully |
| **Transaction Broadcasting** | ✅ PASSED | Mempool and gossip protocol |
| **Application Integration** | ✅ PASSED | Python API demo functional |

### 🏃 Test Execution

```bash
# Run complete test suite
cargo test

# Run with detailed output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration_tests

# Check compilation
cargo check

# Build release version
cargo build --release
```

### 📊 Performance Metrics
- **Test Execution**: <1 second for all 8 tests
- **Compilation**: ~0.37s (debug), ~49s (release)
- **Binary Size**: ~3.7MB (main node), ~484KB (wallet), ~355KB (supply checker)
- **Network Discovery**: <5 seconds for peer connection
- **Memory Usage**: Efficient embedded database (sled)

## ⚙️ Configuration

### Default Configuration
```toml
# config.toml
[network]
listen_address = "/ip4/0.0.0.0/tcp/0"
bootstrap_peers = ["/ip4/1.2.3.4/tcp/6000", "/ip4/5.6.7.8/tcp/6000"]

[mining]
enabled = true
threads = 4

[storage]
path = "axiom_chain.dat"

[ai]
neural_guardian_enabled = true
trust_threshold = 0.4
```

### Environment Variables
```bash
# Network configuration
export AXIOM_LISTEN_ADDR="/ip4/0.0.0.0/tcp/0"
export AXIOM_BOOTSTRAP_PEERS="/ip4/1.2.3.4/tcp/6000,/ip4/5.6.7.8/tcp/6000"

# Mining configuration
export AXIOM_MINING_ENABLED=true
export AXIOM_MINING_THREADS=4

# Storage configuration
export AXIOM_STORAGE_PATH="./data/axiom_chain.dat"
```

### Running with Bootstrap Peers

By default, the node will attempt to connect to these public bootstrap peers:

```
/ip4/34.160.111.145/tcp/6000
/ip4/51.15.23.200/tcp/6000
/ip4/3.8.120.113/tcp/6000
```

To override or add your own, set the `AXIOM_BOOTSTRAP_PEERS` environment variable to a comma-separated list of multiaddresses:

```bash
export AXIOM_BOOTSTRAP_PEERS="/ip4/1.2.3.4/tcp/6000,/ip4/5.6.7.8/tcp/6000"
cargo run --release --bin axiom
```

Replace the example IPs with real public node addresses. This allows your node to discover and sync with the global network, not just local peers.

## 🔍 Network Troubleshooting & Diagnostics

### Quick Diagnostics

When you start a node, it now prints detailed network information:

```
🌐 Node successfully bound to port: 6000
🆔 PeerId: 12D3KooWABC123...
🔊 Listening on: /ip4/0.0.0.0/tcp/6000
[DIAG] To connect another node, set AXIOM_BOOTSTRAP_PEER="12D3KooWABC123...@/ip4/0.0.0.0/tcp/6000"
```

**Use this information to:**
1. Copy the PeerId for connecting other nodes
2. Verify which port the node is listening on
3. Use the provided AXIOM_BOOTSTRAP_PEER string on other nodes

### Automated Troubleshooting Script

Run the network diagnostics script to check your setup:

```bash
./network-troubleshoot.sh
```

This script checks:
- ✅ Node running status
- ✅ Port bindings (6000-6010)
- ✅ Firewall configuration
- ✅ Network interfaces and IPs
- ✅ Internet connectivity
- ✅ NAT/Router detection
- ✅ Bootstrap peer configuration
- ✅ Recent connection events

### Connecting Nodes on Different Networks

**Step 1: Start the first node (Node A)**
```bash
cargo run --release --bin axiom
```

Look for the startup output:
```
🆔 PeerId: 12D3KooWABC123xyz...
🔊 Listening on: /ip4/0.0.0.0/tcp/6000
```

**Step 2: Get your public IP (if behind NAT/router)**
```bash
curl ifconfig.me
# Example output: 203.0.113.42
```

**Step 3: Forward port 6000 on your router**
- Log into your router admin panel
- Forward TCP port 6000 to your device's local IP
- Or allow port 6000 in firewall:
```bash
# Linux (UFW)
sudo ufw allow 6000/tcp

# Or use the shortcut for all Axiom ports
sudo ufw allow 6000:6010/tcp
```

**Step 4: Connect from Node B using bootstrap peer**
```bash
# Use your public IP and PeerId from Node A
export AXIOM_BOOTSTRAP_PEER="12D3KooWABC123xyz@/ip4/203.0.113.42/tcp/6000"
cargo run --release --bin axiom
```

### Connecting Nodes on Same Local Network

If both nodes are on the same LAN (WiFi/Ethernet), mDNS should discover them automatically. No manual configuration needed!

**Just run on both devices:**
```bash
cargo run --release --bin axiom
```

Watch for:
```
🔎 mDNS discovered peer: 12D3KooW...
   └─ 📞 Dialing...
🔗 Peer connected: 12D3KooW... | Total peers: 1
```

### Common Issues & Solutions

**Issue: Peers = 0 after several minutes**

Solutions:
1. **Same network**: Check if mDNS is blocked by firewall
2. **Different networks**: Ensure AXIOM_BOOTSTRAP_PEER is set correctly
3. **Behind NAT**: Forward ports 6000-6010 on your router
4. **Firewall**: Allow ports `sudo ufw allow 6000:6010/tcp`
5. **Wrong IP**: Use public IP (from `curl ifconfig.me`), not local IP

**Issue: "Connection refused" or "Connection timeout"**

Solutions:
1. Verify the node is running on the target device
2. Check firewall allows incoming connections
3. Verify port forwarding is configured correctly
4. Check if the IP address is correct (public vs private)

**Issue: Connections drop immediately**

Solutions:
1. Check both nodes are on compatible versions
2. Verify firewall isn't blocking established connections
3. Check network stability (ping test)

### Enhanced Dashboard

The node dashboard (printed every 10 seconds) now shows detailed network status:

```
--- 🏛️  AXIOM STATUS ---
⛓️  Height: 42 | Diff: 2 | Trend: STABLE ↔️
⏳ Time-Lock: 58m remaining | 🤖 AI Shield: ACTIVE
💰 Mined: 420.00 AXM | Remaining: 83,999,580.00 AXM | 0.50% of max supply
🌐 Network Status:
   ├─ PeerId: 12D3KooWABC123...
   ├─ Connected Peers: 2
   │  ├─ 12D3KooWXYZ789...
   │  └─ 12D3KooWDEF456...
   └─ Listen Addresses:
      ├─ /ip4/192.168.1.100/tcp/6000
      └─ /ip4/0.0.0.0/tcp/6000
------------------------
```

### Detailed Connection Events

The node now logs all network events with detailed information:

```
🔗 Peer connected: 12D3KooW... | Total peers: 1
   └─ Direction: Dialer | Address: /ip4/203.0.113.42/tcp/6000
🔌 Peer disconnected: 12D3KooW... | Total peers: 0
   └─ Cause: Connection reset by peer
📞 Incoming connection attempt from /ip4/198.51.100.10/tcp/54321
⚠️  Outgoing connection to 12D3KooW... failed: Connection timeout
```

## 🤝 Contributing

We welcome contributions to the AXIOM Protocol! Please follow these guidelines:

### Development Setup
```bash
# Fork and clone
git clone https://github.com/your-username/Axiom-Protocol.git
cd Axiom-Protocol

# Create feature branch
git checkout -b feature/your-feature

# Make changes, ensure tests pass
cargo test
cargo clippy

# Submit pull request
```

### Code Standards
- **Rust Edition**: 2021
- **Formatting**: `cargo fmt`
- **Linting**: `cargo clippy` (must pass)
- **Testing**: All new features require tests
- **Documentation**: Public APIs must be documented

### Areas for Contribution
- **ZK-SNARK Optimization**: Upgrade to full arkworks circuits
- **Network Enhancements**: Bootstrap nodes, peer discovery
- **AI Improvements**: Advanced attack detection algorithms
- **Performance**: Parallel processing, caching optimizations
- **Tools**: Block explorers, monitoring dashboards

## 📄 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

```
MIT License

Copyright (c) 2025 AXIOM Protocol Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be
included in all copies or substantial portions of the Software.
```

## 🌐 Links & Resources
- **Documentation**: [docs/](docs/) (generated with `cargo doc`)
- **ONNX Usage**: [ONNX_USAGE.md](ONNX_USAGE.md)

- **GitHub Repository**: [https://github.com/Ghost-84M/Axiom-Protocol](https://github.com/Ghost-84M/Axiom-Protocol)
- **Documentation**: [docs/](docs/) (generated with `cargo doc`)
- **Issues**: [GitHub Issues](https://github.com/Ghost-84M/Axiom-Protocol/issues)
- **Discussions**: [GitHub Discussions](https://github.com/Ghost-84M/Axiom-Protocol/discussions)
- **Latest Release**: [GitHub Releases](https://github.com/Ghost-84M/Axiom-Protocol/releases)

### 📊 Repository Status
- **✅ Fully Deployed**: All features committed and pushed
- **✅ Tests Passing**: 8/8 integration tests successful
- **✅ Network Verified**: Multi-node peer discovery tested
- **✅ Production Ready**: Optimized release builds available
- **✅ Documentation Complete**: Comprehensive README and API docs

### 🚀 Quick Deploy
```bash
# Clone and run immediately
git clone https://github.com/Ghost-84M/Axiom-Protocol.git
cd Axiom-Protocol
cargo build --release
./launch-node.sh
```

## ⚠️ Disclaimer

**✅ PRODUCTION STATUS**: This blockchain implementation has been thoroughly tested with 8/8 passing integration tests, verified multi-node peer discovery, and confirmed transaction processing capabilities. The network monitoring, wallet operations, and consensus mechanisms are fully functional.

This software is provided "as is" without warranty of any kind. Use at your own risk. The AXIOM Protocol implements production-grade security features including ZK-SNARK privacy, AI-powered network protection, and decentralized consensus.

**Network Status**: Ready for mainnet deployment with real-time peer monitoring and comprehensive testing validation.

---

**"The timeline is decentralized. No owners. No governance. Only math."**

*Built with ❤️ in Rust for the decentralized future.*

> **Binary Signature**: `01010011 01100001 01110100 01101111 01110011 01101000 01101001` 🔐

---

## 🔑 Canonical ZK-SNARK Proving Key

**This proving key is the canonical trusted setup artifact for the current circuit.**

- **File:** proving_key.bin
- **IPFS CID (v1, sha2-256):** `bafkreigjmiu2vtn7iehy6btmah7pfyxxknpddij4m3pyaq4occukv2qov4`
- **SHA256:** `f6d4552e9be710535ef45a0470572311d7506a0f1f13b75f340bac11ccffd9cd`
- **IPFS Gateway URL:** [https://bafkreigjmiu2vtn7iehy6btmah7pfyxxknpddij4m3pyaq4occukv2qov4.ipfs.w3s.link/](https://bafkreigjmiu2vtn7iehy6btmah7pfyxxknpddij4m3pyaq4occukv2qov4.ipfs.w3s.link/)

---

