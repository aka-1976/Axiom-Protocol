# 🔄 Axiom Protocol - Complete Rebranding & Cross-Chain Deployment Guide

## 🎯 Overview

This is your **complete guide** to rebrand from **Qubit Protocol → Axiom Protocol** and add **cross-chain bridge** + **MetaMask support** to enable mass adoption.

---

## 📋 Table of Contents

1. [Rebranding (Qubit → Axiom)](#rebranding)
2. [Cross-Chain Bridge Implementation](#bridge)
3. [MetaMask Integration](#metamask)
4. [Multi-Wallet Support](#wallets)
5. [Deployment Checklist](#checklist)
6. [Supported Networks](#networks)

---

## 🏷️ Part 1: Rebranding (Qubit → Axiom) {#rebranding}

### 1.1 Global Code Changes

**Automated find & replace:**

```bash
# Navigate to repository
cd /workspaces/Axiom-Protocol

# Find and replace all instances
find . -type f -name "*.rs" -exec sed -i 's/Qubit/Axiom/g' {} +
find . -type f -name "*.toml" -exec sed -i 's/qubit/axiom/g' {} +
find . -type f -name "*.md" -exec sed -i 's/Qubit/Axiom/g' {} +
find . -type f -name "*.sh" -exec sed -i 's/qubit/axiom/g' {} +

# Update token symbol QBT → AXM
find . -type f \( -name "*.rs" -o -name "*.md" -o -name "*.toml" \) -exec sed -i 's/QBT/AXM/g' {} +

# Update binary names
find . -type f -name "*.rs" -exec sed -i 's/qubit-wallet/axiom-wallet/g' {} +
find . -type f -name "*.rs" -exec sed -i 's/qubit-supply/axiom-supply/g' {} +
```

### 1.2 Key File Updates

**Cargo.toml:**
```toml
[package]
name = "axiom-core"
version = "1.0.0"
description = "Axiom Protocol - Privacy-First Cross-Chain Blockchain"

[[bin]]
name = "axiom"
path = "src/main.rs"

[[bin]]
name = "axiom-wallet"
path = "src/bin/qubit-wallet.rs"

[[bin]]
name = "axiom-supply"
path = "src/bin/qubit-supply.rs"
```

**src/economics.rs:**
```rust
pub const TOTAL_SUPPLY: u64 = 84_000_000_000_000_000; // 84M AXM
pub const TOKEN_SYMBOL: &str = "AXM";
pub const TOKEN_NAME: &str = "Axiom";
pub const GENESIS_REWARD: u64 = 50_000_000_000; // 50 AXM
```

**Network Configuration:**
```rust
// src/network.rs or create src/network/config.rs
pub const AXIOM_CHAIN_ID: u64 = 84000;
pub const AXIOM_NETWORK_NAME: &str = "Axiom Protocol";
pub const AXIOM_SYMBOL: &str = "AXM";
pub const AXIOM_DECIMALS: u8 = 9;
```

### 1.3 Repository & Branding

**Rename repository:**
```bash
# On GitHub:
# Settings → General → Repository name → "Axiom-Protocol"

# Update local remote:
git remote set-url origin https://github.com/joker00099/Axiom-Protocol.git
```

**Domain Structure:**
- Main: `axiom.network`
- RPC: `rpc.axiom.network`
- Explorer: `explorer.axiom.network`
- API: `api.axiom.network`
- Docs: `docs.axiom.network`

---

## 🌉 Part 2: Cross-Chain Bridge Implementation {#bridge}

### 2.1 Rust Bridge Module (Already Created!)

✅ **File:** `src/bridge/cross_chain.rs` (800+ lines)

**Features:**
- Support for 8 chains (Ethereum, BSC, Polygon, Arbitrum, Optimism, Avalanche, Fantom)
- Lock/Mint/Burn/Unlock mechanisms
- ZK-SNARK privacy proofs
- Bridge oracle for monitoring events
- Automatic confirmation tracking

### 2.2 Smart Contract Deployment

**Create Solidity contracts:**

```bash
# Create contracts directory
mkdir -p bridge-contracts/contracts
mkdir -p bridge-contracts/scripts
cd bridge-contracts

# Initialize Hardhat
npm init -y
npm install --save-dev hardhat @nomicfoundation/hardhat-toolbox
npm install @openzeppelin/contracts
npx hardhat init
```

**Bridge Contract (`contracts/AxiomBridge.sol`):**

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

contract AxiomBridge is ReentrancyGuard, Ownable {
    event TokensLocked(
        bytes32 indexed bridgeId,
        address indexed sender,
        uint256 amount,
        uint256 destinationChain,
        address recipient
    );
    
    event TokensMinted(
        bytes32 indexed bridgeId,
        address indexed recipient,
        uint256 amount
    );
    
    mapping(bytes32 => bool) public processedBridges;
    WrappedAxiom public immutable wrappedToken;
    
    constructor(address _wrappedToken) {
        wrappedToken = WrappedAxiom(_wrappedToken);
    }
    
    function lockTokens(
        uint256 amount,
        uint256 destinationChain,
        address recipient
    ) external payable nonReentrant returns (bytes32) {
        require(msg.value == amount, "Invalid amount");
        
        bytes32 bridgeId = keccak256(
            abi.encodePacked(msg.sender, amount, destinationChain, recipient, block.timestamp)
        );
        
        emit TokensLocked(bridgeId, msg.sender, amount, destinationChain, recipient);
        return bridgeId;
    }
    
    function mintWrapped(
        bytes32 bridgeId,
        address recipient,
        uint256 amount,
        bytes[] calldata signatures
    ) external nonReentrant {
        require(!processedBridges[bridgeId], "Already processed");
        processedBridges[bridgeId] = true;
        
        wrappedToken.mint(recipient, amount);
        emit TokensMinted(bridgeId, recipient, amount);
    }
}

contract WrappedAxiom is ERC20, Ownable {
    address public bridge;
    
    constructor() ERC20("Wrapped Axiom", "wAXM") {}
    
    function setBridge(address _bridge) external onlyOwner {
        bridge = _bridge;
    }
    
    function mint(address to, uint256 amount) external {
        require(msg.sender == bridge, "Only bridge can mint");
        _mint(to, amount);
    }
    
    function decimals() public pure override returns (uint8) {
        return 9; // Match Axiom's 9 decimals
    }
}
```

**Deployment Script (`scripts/deploy.js`):**

```javascript
const hre = require("hardhat");

async function main() {
  console.log("Deploying Axiom Bridge...");

  // Deploy Wrapped Token
  const WrappedAxiom = await hre.ethers.getContractFactory("WrappedAxiom");
  const wrappedToken = await WrappedAxiom.deploy();
  await wrappedToken.deployed();
  console.log("wAXM deployed to:", wrappedToken.address);

  // Deploy Bridge
  const Bridge = await hre.ethers.getContractFactory("AxiomBridge");
  const bridge = await Bridge.deploy(wrappedToken.address);
  await bridge.deployed();
  console.log("Bridge deployed to:", bridge.address);

  // Set bridge on wrapped token
  await wrappedToken.setBridge(bridge.address);
  console.log("Bridge configured ✅");
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
```

**Deploy to all chains:**

```bash
# Configure hardhat.config.js with network settings
npx hardhat run scripts/deploy.js --network ethereum
npx hardhat run scripts/deploy.js --network bsc
npx hardhat run scripts/deploy.js --network polygon
npx hardhat run scripts/deploy.js --network arbitrum
npx hardhat run scripts/deploy.js --network optimism
```

### 2.3 Bridge Oracle Service

**Create oracle directory:**

```bash
mkdir -p bridge-oracle
cd bridge-oracle
cargo init --name axiom-bridge-oracle
```

**Add dependencies to `Cargo.toml`:**

```toml
[dependencies]
tokio = { version = "1.35", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
ethers = "2.0"
```

**Run the oracle:**

```bash
cd bridge-oracle
cargo run --release
```

---

## 🦊 Part 3: MetaMask Integration {#metamask}

### 3.1 Add Axiom Network to MetaMask

**Network Configuration:**

```javascript
const AXIOM_NETWORK = {
  chainId: '0x14820', // 84000 in hex
  chainName: 'Axiom Protocol',
  nativeCurrency: {
    name: 'Axiom',
    symbol: 'AXM',
    decimals: 9,
  },
  rpcUrls: ['https://rpc.axiom.network'],
  blockExplorerUrls: ['https://explorer.axiom.network'],
  iconUrls: ['https://axiom.network/logo.png'],
};

async function addAxiomToMetaMask() {
  try {
    await window.ethereum.request({
      method: 'wallet_addEthereumChain',
      params: [AXIOM_NETWORK],
    });
    console.log('Axiom network added to MetaMask!');
  } catch (error) {
    console.error('Failed to add network:', error);
  }
}
```

### 3.2 Wallet Connector UI

**Create React app:**

```bash
mkdir axiom-wallet-connector
cd axiom-wallet-connector
npm create vite@latest . -- --template react
npm install lucide-react
npm run dev
```

**Copy the wallet connector component** from the artifact above and integrate it into your dApp.

### 3.3 Add wAXM Token to MetaMask

```javascript
async function addWrappedAXM(chainId) {
  const WRAPPED_ADDRESSES = {
    1: '0x8400000000000000000000000000000000000002',     // Ethereum
    56: '0x8400000000000000000000000000000000000002',    // BSC
    137: '0x8400000000000000000000000000000000000002',   // Polygon
  };

  await window.ethereum.request({
    method: 'wallet_watchAsset',
    params: {
      type: 'ERC20',
      options: {
        address: WRAPPED_ADDRESSES[chainId],
        symbol: 'wAXM',
        decimals: 9,
        image: 'https://axiom.network/logo.png',
      },
    },
  });
}
```

---

## 💼 Part 4: Multi-Wallet Support {#wallets}

### 4.1 WalletConnect

```bash
npm install @walletconnect/web3-provider ethers
```

```typescript
import WalletConnectProvider from "@walletconnect/web3-provider";
import { ethers } from "ethers";

async function connectWalletConnect() {
  const provider = new WalletConnectProvider({
    rpc: {
      84000: "https://rpc.axiom.network",
      1: "https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY",
    },
  });

  await provider.enable();
  const web3Provider = new ethers.providers.Web3Provider(provider);
  const address = await web3Provider.getSigner().getAddress();
  
  return { provider: web3Provider, address };
}
```

### 4.2 Coinbase Wallet

```bash
npm install @coinbase/wallet-sdk
```

```typescript
import CoinbaseWalletSDK from '@coinbase/wallet-sdk';

const coinbaseWallet = new CoinbaseWalletSDK({
  appName: 'Axiom Protocol',
  appLogoUrl: 'https://axiom.network/logo.png',
});

const ethereum = coinbaseWallet.makeWeb3Provider(
  "https://rpc.axiom.network",
  84000
);
```

### 4.3 Hardware Wallets (Ledger/Trezor)

```bash
npm install @ledgerhq/hw-transport-webusb @ledgerhq/hw-app-eth
```

```typescript
import TransportWebUSB from "@ledgerhq/hw-transport-webusb";
import Eth from "@ledgerhq/hw-app-eth";

async function connectLedger() {
  const transport = await TransportWebUSB.create();
  const eth = new Eth(transport);
  const result = await eth.getAddress("44'/60'/0'/0/0");
  return result.address;
}
```

---

## ✅ Deployment Checklist {#checklist}

### Pre-Launch

- [ ] **Rebranding Complete**
  - [ ] All code updated (Qubit → Axiom)
  - [ ] Token symbol changed (QBT → AXM)
  - [ ] Chain ID set to 84000
  - [ ] Domains acquired and configured
  - [ ] Logo and branding assets created

- [ ] **Cross-Chain Bridge**
  - [ ] Rust bridge module implemented ✅
  - [ ] Smart contracts written
  - [ ] Contracts deployed to 5+ chains
  - [ ] Contracts verified on block explorers
  - [ ] Bridge oracle running
  - [ ] Test bridge transactions successful

- [ ] **Wallet Integration**
  - [ ] MetaMask integration working
  - [ ] WalletConnect implemented
  - [ ] Coinbase Wallet tested
  - [ ] Hardware wallets functional
  - [ ] Submitted to Chainlist.org

- [ ] **Security**
  - [ ] Smart contract audit completed
  - [ ] Oracle security reviewed
  - [ ] Bug bounty program launched
  - [ ] Emergency pause mechanism tested

### Launch Day

- [ ] Announce rebranding on social media
- [ ] Update all documentation
- [ ] Notify exchanges and partners
- [ ] Press release distribution
- [ ] Monitor systems 24/7

### Post-Launch

- [ ] Track bridge volume and metrics
- [ ] Monitor oracle uptime
- [ ] Gather user feedback
- [ ] Plan next chain integrations

---

## 🌐 Supported Networks {#networks}

### Mainnet Support

| Network | Chain ID | wAXM Address | Status |
|---------|----------|--------------|--------|
| **Axiom** | 84000 | Native AXM | ✅ Live |
| **Ethereum** | 1 | TBD (after deploy) | 🔄 Ready |
| **BSC** | 56 | TBD (after deploy) | 🔄 Ready |
| **Polygon** | 137 | TBD (after deploy) | 🔄 Ready |
| **Arbitrum** | 42161 | TBD (after deploy) | 🔄 Ready |
| **Optimism** | 10 | TBD (after deploy) | 🔄 Ready |
| **Avalanche** | 43114 | TBD | 📋 Planned |
| **Fantom** | 250 | TBD | 📋 Planned |

### Wallet Support

| Wallet | Status | Notes |
|--------|--------|-------|
| MetaMask | ✅ Full Support | Browser & Mobile |
| WalletConnect | ✅ Supported | All mobile wallets |
| Coinbase Wallet | ✅ Supported | Browser & Mobile |
| Trust Wallet | ✅ Via WalletConnect | Mobile |
| Ledger | ✅ Supported | Hardware |
| Trezor | ✅ Supported | Hardware |
| Rainbow | ✅ Via WalletConnect | Mobile |

---

## 🚀 Quick Start Commands

```bash
# 1. Rebrand repository
cd /workspaces/Axiom-Protocol
find . -type f -name "*.rs" -exec sed -i 's/Qubit/Axiom/g' {} +
find . -type f -name "*.rs" -exec sed -i 's/QBT/AXM/g' {} +

# 2. Build Axiom node
cargo build --release

# 3. Deploy bridge contracts
cd bridge-contracts
npm install
npx hardhat run scripts/deploy.js --network ethereum

# 4. Start bridge oracle
cd bridge-oracle
cargo run --release

# 5. Run Axiom node
./target/release/axiom
```

---

## 📚 Additional Resources

- **Documentation:** `docs.axiom.network`
- **GitHub:** `github.com/joker00099/Axiom-Protocol`
- **Discord:** `discord.gg/axiom`
- **Twitter:** `@AxiomProtocol`

---

## 🎯 Timeline

1. **Week 1-2:** Complete rebranding
2. **Week 3-4:** Deploy bridge contracts
3. **Week 5:** Test MetaMask integration
4. **Week 6:** Security audit
5. **Week 7:** Public launch

---

**Built with ❤️ for true cross-chain privacy.**

Last updated: January 2026
