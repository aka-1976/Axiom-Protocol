# Join AXIOM Mainnet

## Requirements
- 2 CPU cores, 4GB RAM
- 50GB disk space
- Stable internet

## Quick Start
```bash
# 1. Clone repo
git clone https://github.com/Ghost-84M/Axiom-Protocol.git
cd Axiom-Protocol

# 2. Build
cargo build --release

# 3. Start mining
export AXIOM_BOOTSTRAP_PEERS="/ip4/34.10.172.207/tcp/443/p2p/12D3KooWLdfHzV2q393qEM9AJ1wgyUAaMkehEvPmx4PhAV9rx4VD"
./target/release/axiom --no-mdns
```

## What Happens
- Your node syncs blockchain from mainnet
- You compete with other miners for blocks
- Every 30 minutes, ONE miner wins 50 AXM
- Your wallet.dat stores your AXM

## Backup Your Wallet!
```bash
cp wallet.dat ~/wallet-backup-$(date +%Y%m%d).dat
```

## Verify Everyone is on Same Chain
Monitor connected peers:
```bash
# On bootstrap node
watch -n 5 './target/release/axiom status'
# Should show:
# Connected Peers: 100+
# Height: (same for everyone)
```
Check specific node:
```bash
./target/release/axiom status
# Output:
# Height: (current)
# Connected Peers: (number)
# Synced: ✅
```

## Critical: Genesis Hash Must Match
Everyone MUST have the same genesis hash:
```
7876d9aac11b1197474167b7485626bf535e551a21865c6264f07f614281298c
```
If anyone has a different genesis hash, they're on a different blockchain!

---

For more details, see the main README and deployment guide.
