# Axiom Protocol v2.2.1 Deployment Guide

**Last Updated**: February 6, 2026  
**Version**: 2.2.1  
**Status**: Ready for Production Deployment

---

## Table of Contents

1. [Pre-Deployment Checklist](#pre-deployment-checklist)
2. [System Requirements](#system-requirements)
3. [Installation Steps](#installation-steps)
4. [Configuration](#configuration)
5. [Verification & Testing](#verification--testing)
6. [Network Integration](#network-integration)
7. [Monitoring & Health Checks](#monitoring--health-checks)
8. [Rollback Procedure](#rollback-procedure)
9. [Troubleshooting](#troubleshooting)
10. [Support](#support)

---

## Pre-Deployment Checklist

Before deploying v2.2.1, ensure all of the following are complete:

### Phase 1: Code Review
- [ ] PR #10 reviewed by team leads
- [ ] All changes approved
- [ ] No blocking comments remain
- [ ] Code audit documentation reviewed

### Phase 2: Testing
- [ ] All 67 tests passing locally
- [ ] CI/CD pipeline green
- [ ] Staging environment tested
- [ ] Performance benchmarks acceptable

### Phase 3: Documentation
- [ ] Release notes reviewed (RELEASE_NOTES_v2.2.1.md)
- [ ] Deployment guide understood (this document)
- [ ] Team trained on changes
- [ ] Rollback procedure documented

### Phase 4: Infrastructure
- [ ] Monitoring systems operational
- [ ] Alerting configured
- [ ] Log aggregation ready
- [ ] Backup systems verified

### Phase 5: Stakeholder Approval
- [ ] Product stakeholder approval
- [ ] Operations team approval
- [ ] Security team approval (if required)
- [ ] Compliance review complete

**Once all boxes are checked, proceed to Installation.**

---

## System Requirements

### Minimum Requirements
- **OS**: Linux (Ubuntu 20.04 LTS or later)
- **Architecture**: x86_64
- **GLIBC**: 2.31 or later (v2.2.1 compiled with 2.39)
- **CPU**: 2+ cores recommended
- **RAM**: 2 GB minimum, 4 GB recommended
- **Disk**: 20 GB available for blockchain data

### Network Requirements
- **Bandwidth**: 10 Mbps minimum
- **Ports**: 8765 (API), 30333 (P2P), custom ports for monitoring
- **Firewall**: Inbound access to required ports
- **Connectivity**: Stable internet connection

### Recommended Production Setup
- **OS**: Ubuntu 22.04 LTS or 24.04 LTS
- **CPU**: 4+ cores with 2.5+ GHz frequency
- **RAM**: 8+ GB
- **Disk**: NVMe SSD, 100+ GB
- **Network**: Redundant connectivity (multiple ISPs if possible)

---

## Installation Steps

### Step 1: Download Binary

#### Option A: Download Pre-built Binary

```bash
# Create installation directory
mkdir -p ~/axiom-v2.2.1
cd ~/axiom-v2.2.1

# Download binary (replace with actual download URL)
wget https://github.com/Ghost-84M/Axiom-Protocol/releases/download/v2.2.1/axiom
chmod +x axiom

# Verify binary
./axiom --version
# Expected output: Axiom Protocol v2.2.1
```

#### Option B: Build from Source

```bash
# Clone repository
git clone https://github.com/Ghost-84M/Axiom-Protocol.git
cd Axiom-Protocol

# Checkout v2.2.1
git checkout v2.2.1

# Verify commit
git log -1 --oneline
# Expected: Tag v2.2.1

# Build release binary
cargo build --release --bin axiom

# Verify binary location
ls -lh target/release/axiom
# Expected: ~4.0 MB

# Copy to installation directory
cp target/release/axiom ~/axiom-v2.2.1/
```

### Step 2: Setup Configuration

```bash
# Create configuration directory
mkdir -p ~/.axiom/config

# Copy default configuration
cp config/bootstrap.toml ~/.axiom/config/

# Edit configuration if needed
nano ~/.axiom/config/bootstrap.toml
```

**Key Configuration Parameters**:
- `node_name`: Unique identifier for this node
- `listen_addr`: P2P listen address (default: 127.0.0.1:30333)
- `rpc_addr`: RPC endpoint (default: 127.0.0.1:8765)
- `bootstrap_nodes`: List of bootstrap nodes to connect to
- `storage_path`: Path to blockchain data

### Step 3: Create Directories

```bash
# Create necessary directories
mkdir -p ~/.axiom/{logs,data,snapshots}

# Set proper permissions
chmod 700 ~/.axiom
chmod 700 ~/.axiom/logs
chmod 700 ~/.axiom/data
chmod 700 ~/.axiom/snapshots
```

### Step 4: Verify Installation

```bash
# Check if binary runs
~/axiom-v2.2.1/axiom --help

# Check version
~/axiom-v2.2.1/axiom --version

# Verify configuration is readable
cat ~/.axiom/config/bootstrap.toml
```

---

## Configuration

### Basic Configuration

Edit `~/.axiom/config/bootstrap.toml`:

```toml
[node]
name = "my-axiom-node"
version = "2.2.1"

[network]
listen_addr = "0.0.0.0:30333"
external_addr = "YOUR.IP.ADDRESS:30333"  # Set to your public IP
max_peers = 50

[rpc]
listen_addr = "127.0.0.1:8765"
enable_metrics = true
allowed_origins = ["http://localhost:3000"]  # Adjust as needed

[storage]
path = "/home/user/.axiom/data"
max_size_gb = 100

[bootstrap]
nodes = [
    "node1.axiom-network.io:30333",
    "node2.axiom-network.io:30333"
]

[logging]
level = "info"
format = "json"
path = "/home/user/.axiom/logs"
```

### Advanced Configuration

For production deployments, consider:

```toml
[performance]
max_block_size = 5242880  # 5 MB
transaction_pool_size = 10000
finalization_delay = 12

[consensus]
algorithm = "LWMA"  # Longest Work (PoW)
block_time_target = 12  # seconds

[security]
enable_pruning = true
prune_age_blocks = 100000
validate_signatures = true
```

### Environment Variables

```bash
# Set in shell or systemd service
export AXIOM_HOME=~/.axiom
export AXIOM_LOG_LEVEL=info
export AXIOM_RUST_LOG=axiom=debug,libp2p=info
```

---

## Verification & Testing

### Pre-Deployment Testing (Local)

```bash
# Run full test suite
cd /path/to/axiom-protocol
cargo test --lib

# Expected output:
# test result: ok. 67 passed; 0 failed; 4 ignored

# Run specific module tests
cargo test --lib consensus::
cargo test --lib economics::
cargo test --lib network_config::
```

### Staging Environment Test

```bash
# Start axiom on staging
~/axiom-v2.2.1/axiom \
    --config ~/.axiom/config/staging.toml \
    --log-level debug

# In another terminal, verify connectivity
curl http://localhost:8765/status

# Check for errors
tail -f ~/.axiom/logs/axiom.log | grep -E "ERROR|WARN"

# Monitor for 5-10 minutes, then gracefully shutdown
# Ctrl+C to stop
```

### Health Verification

```bash
# Check node is synced
curl -s http://localhost:8765/status | jq '.blockchain.synced'

# Check peer count
curl -s http://localhost:8765/status | jq '.network.peer_count'

# Check block height
curl -s http://localhost:8765/status | jq '.blockchain.height'

# Verify no errors in logs
grep -i "error\|panic\|fatal" ~/.axiom/logs/axiom.log | wc -l
# Expected: 0
```

---

## Network Integration

### Mainnet Integration

#### Step 1: Bootstrap Connection

```bash
# Ensure configuration points to mainnet bootstrap nodes
nano ~/.axiom/config/bootstrap.toml

# Key settings:
# - Set correct bootstrap_nodes for mainnet
# - Set external_addr to your public IP
# - Set listen_addr to accessible ports
```

#### Step 2: Start Node

```bash
# Start axiom (using systemd or manual)
systemctl start axiom
# OR
./axiom --config ~/.axiom/config/bootstrap.toml &
```

#### Step 3: Monitor Sync Progress

```bash
# Watch blockchain sync
watch -n 5 'curl -s http://localhost:8765/status | jq ".blockchain | {height, syncing, synced}"'

# Expected output during sync:
# {
#   "height": 1234567,
#   "syncing": true,
#   "synced": false
# }

# After full sync:
# {
#   "height": 5432100,
#   "syncing": false,
#   "synced": true
# }
```

#### Step 4: Verify Consensus Participation

```bash
# Check if node is validating blocks
curl -s http://localhost:8765/status | jq '.consensus'

# Expected for PoW:
# {
#   "algorithm": "LWMA",
#   "mining": true,
#   "hashrate": 123456
# }
```

---

## Monitoring & Health Checks

### Log Monitoring

```bash
# Real-time log monitoring
tail -f ~/.axiom/logs/axiom.log

# Search for specific issues
grep "ERROR" ~/.axiom/logs/axiom.log | tail -20

# Count errors
grep -c "ERROR" ~/.axiom/logs/axiom.log

# Monitor performance metrics
grep "finalization_time" ~/.axiom/logs/axiom.log | tail -10
```

### Metrics & Dashboards

#### Prometheus Integration

```bash
# Metrics endpoint (if enabled)
curl http://localhost:9090/metrics

# Check key metrics:
# - axiom_blocks_height (current block height)
# - axiom_network_peers (connected peers)
# - axiom_mempool_transactions (pending transactions)
# - axiom_consensus_participation (block validation rate)
```

#### Grafana Dashboards

Pre-built dashboards available in `monitoring/grafana/`:
1. Node Overview
2. Performance Metrics
3. Network Health
4. Consensus Status

### Health Check Script

```bash
#!/bin/bash
# Health check for axiom node

echo "=== Axiom Node Health Check ==="

# Check if process is running
if pgrep -f axiom > /dev/null; then
    echo "✅ Process running"
else
    echo "❌ Process not running"
    exit 1
fi

# Check API connectivity
STATUS=$(curl -s http://localhost:8765/status 2>/dev/null)
if [ $? -eq 0 ]; then
    echo "✅ API responsive"
else
    echo "❌ API not responsive"
    exit 1
fi

# Check blockchain sync
SYNCED=$(echo "$STATUS" | jq -r '.blockchain.synced')
if [ "$SYNCED" = "true" ]; then
    echo "✅ Blockchain synced"
else
    echo "⚠️  Blockchain syncing..."
fi

# Check peer count
PEERS=$(echo "$STATUS" | jq -r '.network.peer_count')
if [ "$PEERS" -gt 0 ]; then
    echo "✅ Connected to $PEERS peers"
else
    echo "❌ No peers connected"
fi

# Check error count
ERRORS=$(grep -c "ERROR" ~/.axiom/logs/axiom.log 2>/dev/null || echo "0")
if [ "$ERRORS" -eq 0 ]; then
    echo "✅ No errors in logs"
else
    echo "⚠️  $ERRORS errors found (check logs)"
fi

echo "=== Health Check Complete ==="
```

---

## Rollback Procedure

### When to Rollback

Rollback to v2.2.0 if:
- Critical bugs found in production
- Performance degradation observed
- Data corruption detected
- Network incompatibility issues
- Security vulnerability discovered

### Rollback Steps

#### Step 1: Stop Current Node

```bash
# Stop gracefully
systemctl stop axiom
# OR
pkill -f "axiom.*--config"

# Wait for graceful shutdown (up to 30 seconds)
sleep 30

# Force kill if needed
pkill -9 -f "axiom.*--config" || true
```

#### Step 2: Backup Current Data

```bash
# Backup blockchain data
tar -czf ~/.axiom/backups/data-v2.2.1-$(date +%s).tar.gz ~/.axiom/data/

# Backup configuration
cp ~/.axiom/config/bootstrap.toml ~/.axiom/config/bootstrap.toml.v2.2.1.bak
```

#### Step 3: Download Previous Version

```bash
# Get v2.2.0 binary
wget https://github.com/Ghost-84M/Axiom-Protocol/releases/download/v2.2.0/axiom
chmod +x axiom

# Verify version
./axiom --version
# Expected: Axiom Protocol v2.2.0
```

#### Step 4: Restore Previous Version

```bash
# Replace binary
cp axiom ~/axiom-v2.2.0/
rm ~/axiom-v2.2.1/axiom

# Start with v2.2.0
~/axiom-v2.2.0/axiom --config ~/.axiom/config/bootstrap.toml &

# Verify it starts
sleep 5
curl http://localhost:8765/status
```

#### Step 5: Verify Rollback

```bash
# Check version
curl -s http://localhost:8765/status | jq '.version'
# Expected: "2.2.0"

# Monitor logs for proper operation
tail -f ~/.axiom/logs/axiom.log

# Watch for 10+ minutes to ensure stability
```

#### Step 6: Post-Rollback Analysis

```bash
# Collect logs for debugging
tar -czf ~/.axiom/debug/logs-rollback-$(date +%s).tar.gz ~/.axiom/logs/

# Notify team of rollback
# File incident report
# Schedule post-mortem
```

---

## Troubleshooting

### Common Issues

#### Issue 1: Binary Won't Start

**Symptoms**: `./axiom: command not found` or `Segmentation fault`

**Solutions**:
```bash
# Check binary compatibility
ldd ./axiom | grep "not found"

# Verify GLIBC version
ldd --version | head -1

# Check file integrity
file axiom
# Expected: ELF 64-bit LSB executable

# Try debug build
cargo build --bin axiom
./target/debug/axiom --config ~/.axiom/config/bootstrap.toml
```

#### Issue 2: Cannot Connect to Network

**Symptoms**: `0 peers` in status, no connections

**Solutions**:
```bash
# Check configuration
cat ~/.axiom/config/bootstrap.toml | grep bootstrap_nodes

# Test bootstrap node connectivity
nc -zv bootstrap.axiom-network.io 30333

# Check firewall rules
sudo ufw status | grep 30333

# Try with debug logging
RUST_LOG=debug ./axiom --config ~/.axiom/config/bootstrap.toml
```

#### Issue 3: Out of Memory

**Symptoms**: Node crashes, `killed` in logs

**Solutions**:
```bash
# Check available memory
free -h

# Monitor memory usage
watch -n 1 'ps aux | grep axiom'

# Reduce database size (if safe)
# Or increase system memory

# Enable pruning in config
# max_blocks_to_keep = 1000000
```

#### Issue 4: Blockchain Sync Slow

**Symptoms**: `syncing: false` but height not advancing

**Solutions**:
```bash
# Check peer count and quality
curl -s http://localhost:8765/status | jq '.network'

# Verify block download speed
BLOCKS_PER_MIN=$(curl -s http://localhost:8765/metrics | grep axiom_blocks_height)

# Consider full resync if severely out of sync
# Backup data folder
# Delete blockchain data
# Restart node to re-download
```

#### Issue 5: High CPU Usage

**Symptoms**: `top` shows axiom using 90%+ CPU consistently

**Solutions**:
```bash
# Check what's consuming CPU
perf top -p $(pgrep -f axiom)

# Reduce concurrency in config
worker_threads = 2

# Check for infinite loops in logs
tail -f ~/.axiom/logs/axiom.log | grep -i loop
```

### Debug Mode

For detailed troubleshooting:

```bash
# Enable debug logging
export RUST_LOG=axiom=debug,libp2p=debug
./axiom --config ~/.axiom/config/bootstrap.toml 2>&1 | tee ~/.axiom/logs/debug-$(date +%s).log

# Collect system information
uname -a
lsb_release -a
cargo --version
rustc --version

# Check binary details
nm axiom | grep main
strings axiom | grep version

# Create debug bundle
tar -czf debug-bundle-$(date +%s).tar.gz \
    ~/.axiom/logs/ \
    ~/.axiom/config/ \
    /var/log/syslog
```

---

## Support

### Getting Help

**For Deployment Issues**:
1. Check this guide's [Troubleshooting](#troubleshooting) section
2. Review logs: `~/.axiom/logs/axiom.log`
3. Run health check script above
4. Create issue on GitHub with:
   - Error logs
   - System information (`uname -a`)
   - Configuration (sanitized)
   - Steps to reproduce

**For Security Issues**:
- Email: security@axiom-protocol.io
- Do NOT create public GitHub issues for security vulnerabilities

**For General Questions**:
- GitHub Discussions
- Community Discord
- Documentation: [TECHNICAL_SPEC.md](TECHNICAL_SPEC.md)

---

## Post-Deployment

### Day 1 Checklist
- [ ] Node started successfully
- [ ] Syncing with network
- [ ] No errors in logs
- [ ] Peer count > 0
- [ ] API responses normal
- [ ] Metrics being collected

### Week 1 Checklist
- [ ] Node fully synced
- [ ] Stable block production
- [ ] No consensus errors
- [ ] Performance within expected range
- [ ] All alerts functioning
- [ ] Backup systems verified

### Ongoing Maintenance
- Monitor logs daily
- Check metrics weekly
- Backup data monthly
- Update security patches promptly
- Participate in network consensus

---

## Appendix

### Service File (systemd)

Create `/etc/systemd/system/axiom.service`:

```ini
[Unit]
Description=Axiom Protocol Node
After=network.target

[Service]
Type=simple
User=axiom
ExecStart=/home/axiom/axiom-v2.2.1/axiom --config /home/axiom/.axiom/config/bootstrap.toml
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable axiom
sudo systemctl start axiom
sudo systemctl status axiom
```

### Useful Commands

```bash
# Manage service
systemctl {start|stop|restart|status} axiom

# View logs
journalctl -u axiom -f  # Real-time
journalctl -u axiom -n 100  # Last 100 lines

# Check performance
systemctl show -p MemoryCurrent axiom

# Monitor uptime
uptime

# Create snapshot
curl -X POST http://localhost:8765/snapshot

# Stop and wait
systemctl stop --no-block axiom && sleep 30
```

---

**Deployment Guide v2.2.1**  
Last Updated: February 6, 2026  
Status: Production Ready
