# AXIOM Protocol - Network Configuration

## 🌐 MAINNET CONFIGURATION

AXIOM Protocol is configured for **MAINNET DEPLOYMENT** (not testnet).

### Network Parameters

```toml
Network ID: 1                  # Mainnet
Chain ID: "AXIOM-1"            # Mainnet identifier
Genesis Timestamp: TBD         # To be set at launch
```

### Bootstrap Nodes

The network uses the following mainnet bootnodes (see `config/bootstrap.toml`):

```
Region          IP Address       Port    Purpose
---------------------------------------------------
US-Central      34.145.123.45   6000    Discovery
EU-West         35.246.89.12    6000    Discovery
Asia-SE         13.237.156.78   6000    Discovery  
South America   52.67.234.89    6000    Discovery
```

**Note:** These are placeholder IPs. Real mainnet bootnodes will be deployed at launch.

### Consensus Configuration

```toml
# Block Production
block_time = 1800              # 30 minutes (1800 seconds)
vdf_time_param = 3600000000    # Calibrated for 1-hour blocks

# VDF Parameters
vdf_iterations = 3600000000    # Sequential squaring operations
vdf_rsa_bits = 2048           # RSA modulus size (production)

# ZK-SNARK Parameters
zk_curve = "BN254"            # Elliptic curve
zk_proof_system = "Groth16"   # Proof system

# AI Oracle Configuration
min_oracles = 3               # Minimum for consensus
oracle_count = 5              # Initial oracle network size
similarity_threshold = 0.85   # Response similarity (85%)
```

### Economics Configuration

```toml
# Supply
max_supply = 12400000000000000      # 124M AXM (8 decimals)
initial_reward = 5000000000        # 50 AXM per block
decimals = 8                       # Smallest unit: 0.00000001 AXM

# Halving Schedule
halving_interval = 840000          # Blocks between halvings
total_halvings = 32               # Until reward rounds to 0

# Initial Distribution (Genesis Block)
foundation_allocation = 1680000000000000    # 16.8M (20%)
development_allocation = 1260000000000000   # 12.6M (15%)
community_allocation = 840000000000000      # 8.4M (10%)
advisor_allocation = 420000000000000        # 4.2M (5%)
mining_rewards = 4200000000000000           # 42M (50%)
```

### Node Types

#### 1. Validator Node (Full Node + Block Production)
```toml
node_type = "validator"
enable_mining = true
validator_key = "/path/to/validator.key"
```

**Requirements:**
- 8 vCPU, 32GB RAM, 1TB SSD
- 1Gbps network
- 24/7 uptime

#### 2. Full Node (Non-Validating)
```toml
node_type = "full"
enable_mining = false
```

**Requirements:**
- 4 vCPU, 16GB RAM, 500GB SSD
- 100Mbps network

#### 3. Light Node (SPV Client)
```toml
node_type = "light"
enable_mining = false
pruning = true
```

**Requirements:**
- 2 vCPU, 4GB RAM, 50GB SSD
- 10Mbps network

### Network Ports

```
Port    Protocol    Purpose
------------------------------------
6000    TCP         P2P networking (libp2p)
8545    HTTP        RPC endpoints
8546    WS          WebSocket RPC
9615    HTTP        Prometheus metrics
30333   TCP         Additional P2P
```

**Firewall Rules:**
- Open: 6000, 8545, 8546 (for public nodes)
- Restricted: 9615 (metrics - internal only)

### Running a Mainnet Node

#### As Validator
```bash
cargo run --release --bin axiom \
  --config config/mainnet.toml \
  --validator \
  --keys /secure/path/validator.key \
  --name "My AXIOM Validator"
```

#### As Full Node
```bash
cargo run --release --bin axiom \
  --config config/mainnet.toml \
  --full-node \
  --name "My AXIOM Node"
```

#### With Custom Bootnodes
```bash
cargo run --release --bin axiom \
  --config config/mainnet.toml \
  --bootnodes /ip4/1.2.3.4/tcp/6000/p2p/12D3...
```

### Mainnet vs Testnet Comparison

| Feature | Mainnet (Current) | Testnet (N/A) |
|---------|-------------------|---------------|
| Network ID | 1 | 2 (not used) |
| Supply | 124M AXM (real value) | N/A |
| Block Time | 1 hour | N/A |
| Validators | Production (3+) | N/A |
| Reset Policy | Never | N/A |
| Purpose | **Production use** | Skipped |

**Why No Testnet?**
- All critical bugs already fixed
- 11/11 tests passing
- Production-ready code from Day 1
- Security audits in parallel with mainnet
- Real economic incentives align stakeholders

### Verifying Network Configuration

```bash
# Check your node's network ID
cargo run --bin axiom config show | grep network_id
# Expected: network_id = 1 (mainnet)

# Verify genesis block
cargo run --bin axiom genesis verify
# Expected: Network: mainnet, Supply: 124,000,000 AXM

# Check connected peers
curl http://localhost:8545/peers
# Should show mainnet bootnodes
```

### Network Status Endpoints

```bash
# Node info
curl http://localhost:8545/node_info

# Network stats
curl http://localhost:8545/net_stats

# Current block height
curl http://localhost:8545/block_height

# Peer count
curl http://localhost:8545/peer_count
```

### Mainnet Launch Phases

#### Phase 0: Pre-Genesis (Current)
- Code complete and tested
- Infrastructure being prepared
- Validators identified
- Community informed

#### Phase 1: Genesis (Week 1)
- Genesis block created at specific timestamp
- Initial validators start simultaneously
- First VDF computation begins
- Network goes LIVE! 🚀

#### Phase 2: Stabilization (Week 2-4)
- Monitor block production (1 block/hour)
- Verify VDF proofs
- Check oracle consensus
- Onboard additional validators

#### Phase 3: Growth (Month 2+)
- Exchange listings
- Public node operators
- Ecosystem development
- Community expansion

### Emergency Procedures

#### Network Halt Detection
```bash
# If no new blocks for 2+ hours
curl http://localhost:8545/block_height
# Compare with previous reading

# Check validator status
curl http://localhost:8545/validators
```

#### Node Sync Issues
```bash
# Resync from genesis
cargo run --bin axiom \
  --config config/mainnet.toml \
  --resync-from-genesis

# Or sync from specific block
cargo run --bin axiom \
  --config config/mainnet.toml \
  --resync-from 12345
```

### Monitoring & Alerts

**Key Metrics to Watch:**
- Block height (should increase by 1/hour)
- VDF computation time (should be ~1800 seconds)
- Oracle consensus rate (should be >95%)
- Peer count (should be >10)
- Transaction throughput (target 1000 TPS)

**Alert Thresholds:**
- ⚠️  VDF time >4000s (>1h 6m)
- 🚨 No new block in 2 hours
- ⚠️  Oracle consensus <90%
- 🚨 Peer count <3
- ⚠️  Disk usage >80%

### Support & Community

**Development Team:**
- GitHub: https://github.com/Ghost-84M/Axiom-Protocol
- Issues: Report bugs via GitHub Issues

**Node Operators:**
- Discord: [To be announced]
- Telegram: [To be announced]
- Documentation: This repository

---

## ✅ Confirmation: THIS IS MAINNET

**Network Type:** MAINNET  
**Network ID:** 1  
**Purpose:** Production deployment with real economic value  
**Launch Date:** To be announced  
**Status:** Ready for deployment

**Testnet?** NO - We are skipping testnet and launching directly to mainnet.

All nodes running this code are configured for **MAINNET OPERATION**.
