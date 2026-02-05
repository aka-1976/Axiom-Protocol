# Axiom Protocol Bootstrap Node Deployment Guide

## Overview

This guide explains how to set up and run an Axiom Protocol bootstrap node on a GCP instance, and how other nodes connect to it.

## Current Bootstrap Node Information

**Status:** ✅ Active  
**Public IP:** `34.10.172.207`  
**Port:** `6000`  
**PeerId:** `12D3KooWAzD3QjhHMamey1XuysPovzwXyAZy9VzpZmQN7GkrURWU`  
**Multiaddr:** `/ip4/34.10.172.207/tcp/6000/p2p/12D3KooWAzD3QjhHMamey1XuysPovzwXyAZy9VzpZmQN7GkrURWU`

---

## For Bootstrap Node Operator (Server)

### 1. Initial Setup

```bash
cd ~/Axiom-Protocol

# Build release binary
cargo build --release

# Run first time to get PeerId
./target/release/axiom
```

**Copy from output:**
- 🆔 PeerId
- 🔊 Listening address

Press `Ctrl+C` to stop.

### 2. Configure Firewall

```bash
# Allow SSH
sudo ufw allow 22/tcp

# Allow Axiom P2P
sudo ufw allow 6000/tcp

# Enable firewall
sudo ufw --force enable

# Verify
sudo ufw status
```

### 3. Keep Node Running (Systemd Service)

```bash
# Create service file
sudo tee /etc/systemd/system/axiom-bootstrap.service > /dev/null << 'EOF'
[Unit]
Description=Axiom Protocol Bootstrap Node
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$HOME/Axiom-Protocol
ExecStart=$HOME/Axiom-Protocol/target/release/axiom
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

# Enable and start
sudo systemctl daemon-reload
sudo systemctl enable axiom-bootstrap
sudo systemctl start axiom-bootstrap

# Check status
sudo systemctl status axiom-bootstrap

# View logs
sudo journalctl -u axiom-bootstrap -f
```

### 4. Monitor Bootstrap Node

```bash
# Check if running
systemctl is-active axiom-bootstrap

# View recent logs
sudo journalctl -u axiom-bootstrap -n 50 --no-pager

# Monitor in real-time
sudo journalctl -u axiom-bootstrap -f

# Check port is listening
sudo netstat -tlnp | grep 6000
```

---

## For Client Nodes (Connecting to Bootstrap)

### Option 1: Edit config/bootstrap.toml (Recommended)

```bash
# File: config/bootstrap.toml
# The repository already includes the main bootstrap node

# Just build and run:
cd ~/Axiom-Protocol
cargo build --release
./target/release/axiom
```

Expected output:
```
🌍 Bootstrap Configuration:
   📌 Using config/bootstrap.toml with server bootstrap node
🔗 Peer connected: 12D3KooWAzD3QjhHMamey1XuysPovzwXyAZy9VzpZmQN7GkrURWU
```

### Option 2: Use Environment Variable

```bash
export AXIOM_BOOTSTRAP_PEERS="/ip4/34.10.172.207/tcp/6000/p2p/12D3KooWAzD3QjhHMamey1XuysPovzwXyAZy9VzpZmQN7GkrURWU"
cargo run --release
```

### Option 3: Local mDNS Discovery (Same Network Only)

```bash
# Leave config/bootstrap.toml empty
# Nodes will discover each other via mDNS
cargo run --release
```

---

## Node Synchronization

### What Happens During Sync

1. **Connection Phase:**
   ```
   🔍 Checking bootstrap connectivity...
   🔗 Peer connected: 12D3KooWAzD3QjhHMamey1XuysPovzwXyAZy9VzpZmQN7GkrURWU
   ```

2. **Block Exchange Phase:**
   ```
   📥 Requesting chain from peer: 12D3KooWAzD3QjhHMamey1XuysPovzwXyAZy9VzpZmQN7GkrURWU
   🔁 Synced complete chain from peer. New height: 10
   ✅ Block accepted and added to chain
   ```

3. **Synchronized State:**
   ```
   ⛓️  Height: 10 | Diff: 1000 | Trend: STABLE ↔️
   🌐 Connected Peers: 1
   ```

### Monitoring Sync Progress

```bash
# Watch the status line for height changes
# Higher block height = more synced

tail -f ~/.axiom/logs.txt | grep "Height"
```

---

## Troubleshooting

### Issue: "Port 6000 already in use"

```bash
# Check what's using it
sudo lsof -i :6000

# Kill the process (if needed)
sudo kill -9 <PID>

# Or try different port in bootstrap.toml
```

### Issue: "Can't connect to bootstrap node"

```bash
# Verify firewall
sudo ufw status | grep 6000

# Test connectivity from your machine
telnet 34.10.172.207 6000

# Check node logs
sudo journalctl -u axiom-bootstrap --no-pager | tail -20
```

### Issue: "mDNS discovered peer but failed to dial"

This is normal - mDNS may discover unsupported multiaddr formats. The node will retry with TCP.

### Issue: Sync stuck at Height: 1

```bash
# The bootstrap node may not have mined blocks yet
# Wait for bootstrap node to catch up, or
# Mine blocks manually:

# Check if mining is enabled
grep "mining_enabled" config/bootstrap.toml

# Increase mining threads for faster block generation
```

---

## Adding More Bootstrap Nodes

To add redundancy, update `config/bootstrap.toml`:

```toml
bootnodes = [
    "/ip4/34.10.172.207/tcp/6000/p2p/12D3KooWAzD3QjhHMamey1XuysPovzwXyAZy9VzpZmQN7GkrURWU",
    "/ip4/X.X.X.X/tcp/6000/p2p/12D3KooWXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
    "/ip4/Y.Y.Y.Y/tcp/6000/p2p/12D3KooWYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY",
]
```

---

## Performance Tuning

### For Bootstrap Node

```bash
# Monitor resource usage
htop

# Recommended specs:
# - CPU: 2+ cores
# - RAM: 2+ GB
# - Disk: 10+ GB for blockchain data
# - Network: 10+ Mbps

# Increase file descriptor limits
sudo ulimit -n 65536
```

### For Client Nodes

```bash
# In config/bootstrap.toml, adjust mining threads
[mining]
enabled = true
threads = 4  # Increase for more mining power
```

---

## Security Considerations

⚠️ **Important:**
- Keep SSH access restricted
- Use strong passwords
- Monitor the systemd service for crashes
- Back up wallet.dat regularly
- Keep Axiom updated

---

## Status Dashboard

Monitor the network status at:
```
tail $(find ~ -name "health_check.json" 2>/dev/null | head -1)
```

---

## Support

For issues or questions:
1. Check bootstrap node logs: `sudo journalctl -u axiom-bootstrap -f`
2. Verify network connectivity: `ping 34.10.172.207`
3. Review this guide's troubleshooting section
4. Open an issue on GitHub: https://github.com/Ghost-84M/Axiom-Protocol

---

**Last Updated:** February 5, 2026  
**Axiom Protocol Version:** 2.0.0  
**Status:** ✅ Mainnet Bootstrap Node Active
