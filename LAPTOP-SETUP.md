# 💻 AXIOM Protocol - Laptop Setup Guide

## 📥 Clone and Build on Your Laptop

### Step 1: Clone the Repository
```bash
git clone https://github.com/Ghost-84M/Axiom-Protocol.git
cd Axiom-Protocol
```

### Step 2: Install Rust (if not installed)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Step 3: Install Dependencies

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev libgmp-dev
```

**macOS:**
```bash
brew install gmp
```

### Step 4: Build the Project
```bash
# Build in release mode (optimized, faster)
cargo build --release

# Or build in debug mode (faster compilation, slower runtime)
cargo build
```

Build time: ~2-5 minutes depending on your system.

### Step 5: Run the Node
```bash
# Run release version (recommended)
./target/release/axiom

# Or run debug version
./target/debug/axiom
```

---

## 🔄 How to Update AXIOM on Your Laptop

### Method 1: Pull Latest Changes
```bash
cd Axiom-Protocol

# Stop running node first (Ctrl+C if running in foreground)

# Pull latest code
git pull origin main

# Rebuild
cargo build --release

# Restart node
./target/release/axiom
```

### Method 2: Clean Build (if issues occur)
```bash
cd Axiom-Protocol

# Pull latest
git pull origin main

# Clean previous build
cargo clean

# Rebuild from scratch
cargo build --release

# Run
./target/release/axiom
```

### Method 3: Quick Update Script
Create a file `update-axiom.sh`:
```bash
#!/bin/bash
echo "🔄 Updating AXIOM Protocol..."

# Pull latest code
git pull origin main

# Rebuild
cargo build --release

echo "✅ Update complete! Run: ./target/release/axiom"
```

Make it executable and run:
```bash
chmod +x update-axiom.sh
./update-axiom.sh
```

---

## 🚀 Running Options

### 1. Foreground (see all output)
```bash
./target/release/axiom
```
Stop with: `Ctrl+C`

### 2. Background (runs in background)
```bash
nohup ./target/release/axiom > axiom.log 2>&1 &
```

View logs:
```bash
tail -f axiom.log
```

Stop:
```bash
pkill axiom
```

### 3. Using systemd (Linux) - Auto-restart on boot
```bash
# Copy service file
sudo cp contrib/axiom.service /etc/systemd/system/

# Edit paths in the service file if needed
sudo nano /etc/systemd/system/axiom.service

# Enable and start
sudo systemctl enable axiom
sudo systemctl start axiom

# Check status
sudo systemctl status axiom

# View logs
sudo journalctl -u axiom -f
```

---

## 🔍 Verify Node is Running

### Check Process
```bash
ps aux | grep axiom
```

### Check Network Connections
```bash
# On Linux
ss -tulpn | grep 6000

# On macOS
lsof -i :6000
```

### View Node Output
```bash
tail -f axiom.log
```

---

## 🌐 Connect Multiple Nodes

If you run one node on your laptop and want to connect it to another node:

**On Node 1:** Note the PeerId and address from the output:
```
🆔 PeerId: 12D3KooWKAMwtFmGyCocTKsiYvRD9EXF6jBTkRTMajCxWZ7SmUSc
🔊 Listening on: /ip4/192.168.1.100/tcp/6000
```

**On Node 2:** Set the bootstrap peer:
```bash
AXIOM_BOOTSTRAP_PEER="12D3KooWKAMwtFmGyCocTKsiYvRD9EXF6jBTkRTMajCxWZ7SmUSc@/ip4/192.168.1.100/tcp/6000" ./target/release/axiom
```

---

## 🔧 Troubleshooting

### Port Already in Use
If port 6000 is busy, the node will automatically try ports 6001-6010.

### Build Errors
```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

### Missing Dependencies
```bash
# Ubuntu/Debian
sudo apt install build-essential pkg-config libssl-dev libgmp-dev

# macOS
brew install gmp openssl pkg-config
```

### Check Latest Version
```bash
git log --oneline -5
```

Current version: **v2.0.0** (55175ef - Fix node startup)

---

## 📊 Node Status

The node will display status every 10 seconds:
- ⛓️ Chain height and difficulty
- ⏳ Time-lock countdown (60 minutes per block)
- 💰 Supply information
- 🌐 Connected peers
- 🤖 AI Neural Guardian stats

---

## 🔐 Security Notes

- **wallet.dat** - Keep this file safe! It contains your private key
- Backup: `cp wallet.dat wallet.dat.backup`
- The node runs on **MAINNET** with Chain ID **84000**
- Maximum supply: **124,000,000 AXM**
- Block reward: **50 AXM** (halves every 1,240,000 blocks)
- Block time: **30 minutes** (1800 seconds)

---

## 📚 Additional Commands

### Build specific binary
```bash
cargo build --release --bin axiom
cargo build --release --bin axiom-wallet
cargo build --release --bin axiom-supply
```

### Run tests
```bash
cargo test
```

### Check version
```bash
./target/release/axiom --version
# (Note: currently shows version from banner)
```

---

## 🆘 Need Help?

- Check logs: `tail -f axiom.log`
- View errors: `cat axiom.log | grep -i error`
- Repository: https://github.com/Ghost-84M/Axiom-Protocol
- Latest Release: https://github.com/Ghost-84M/Axiom-Protocol/releases

---

**Ready to run AXIOM Protocol v2.0.0 on your laptop! 🚀**
