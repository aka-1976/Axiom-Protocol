# 🌉 Axiom Bridge Contracts

Smart contracts for the Axiom Protocol cross-chain bridge.

## 📋 Overview

These Solidity contracts enable cross-chain bridging of AXM tokens between Axiom Protocol and major EVM chains:

- **Ethereum** (Chain ID: 1)
- **BSC** (Chain ID: 56)
- **Polygon** (Chain ID: 137)
- **Arbitrum** (Chain ID: 42161)
- **Optimism** (Chain ID: 10)
- **Avalanche** (Chain ID: 43114)
- **Fantom** (Chain ID: 250)

## 🏗️ Architecture

### Contracts

1. **AxiomBridge.sol** - Main bridge contract
   - Lock/unlock native AXM on Axiom chain
   - Mint/burn wrapped AXM (wAXM) on other chains
   - Oracle-based validation
   - Security features (pausable, reentrancy guards)

2. **WrappedAxiom.sol** - ERC20 wrapped token
   - Standard ERC20 with 9 decimals (matching AXM)
   - Only mintable/burnable by bridge contract
   - 1:1 peg with native AXM

## 🚀 Quick Start

### Installation

```bash
cd bridge-contracts
npm install
```

### Configuration

1. Copy `.env.example` to `.env`:
```bash
cp .env.example .env
```

2. Fill in your configuration:
```bash
PRIVATE_KEY=your_deployment_private_key
ETHEREUM_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY
ETHERSCAN_API_KEY=your_etherscan_api_key
# ... (add other networks)
```

### Compile

```bash
npx hardhat compile
```

### Deploy

Deploy to a single network:
```bash
npx hardhat run scripts/deploy.js --network ethereum
npx hardhat run scripts/deploy.js --network bsc
npx hardhat run scripts/deploy.js --network polygon
```

Deploy to all networks:
```bash
npm run deploy:all
```

### Verify Contracts

After deployment, verify on block explorers:

```bash
npx hardhat verify --network ethereum <WRAPPED_TOKEN_ADDRESS>
npx hardhat verify --network ethereum <BRIDGE_ADDRESS> <WRAPPED_TOKEN_ADDRESS>
```

## 📊 Bridge Flow

### Axiom → Other Chain

1. User calls `lockTokens()` on Axiom bridge
2. Native AXM locked in contract
3. Oracle detects lock event
4. Oracle waits for confirmations
5. Oracle calls `mintWrapped()` on destination chain
6. User receives wAXM on destination chain

### Other Chain → Axiom

1. User calls `burnWrapped()` on source chain
2. wAXM tokens burned
3. Oracle detects burn event
4. Oracle calls `unlockTokens()` on Axiom bridge
5. User receives native AXM

## 🔒 Security Features

- **ReentrancyGuard** - Prevents reentrancy attacks
- **Pausable** - Emergency stop mechanism
- **Oracle validation** - Multiple oracle signatures required
- **Amount limits** - Min/max bridge amounts
- **Ownable** - Admin functions protected

## 📄 Contract Addresses

After deployment, addresses will be stored in `deployments/` directory:

```
deployments/
├── ethereum-deployment.json
├── bsc-deployment.json
├── polygon-deployment.json
├── arbitrum-deployment.json
└── optimism-deployment.json
```

## 🧪 Testing

Run tests:
```bash
npx hardhat test
```

Run with gas reporting:
```bash
REPORT_GAS=true npx hardhat test
```

## 📚 Contract Functions

### AxiomBridge

**User Functions:**
- `lockTokens(amount, destinationChain, recipient)` - Lock AXM to bridge
- `burnWrapped(amount, destinationChain, recipient)` - Burn wAXM to bridge back

**Oracle Functions:**
- `mintWrapped(bridgeId, recipient, amount, signatures)` - Mint wAXM on destination
- `unlockTokens(bridgeId, recipient, amount, signatures)` - Unlock AXM on Axiom

**Admin Functions:**
- `addOracle(address)` - Add trusted oracle
- `removeOracle(address)` - Remove oracle
- `setRequiredOracles(uint256)` - Set minimum oracle signatures
- `pause() / unpause()` - Emergency controls

### WrappedAxiom (wAXM)

Standard ERC20 with:
- 9 decimals (matching native AXM)
- Bridge-only minting/burning
- Name: "Wrapped Axiom"
- Symbol: "wAXM"

## 🛠️ Development

**Add new network:**

1. Add to `hardhat.config.js`:
```javascript
newchain: {
  url: "https://rpc.newchain.com",
  accounts: [process.env.PRIVATE_KEY],
  chainId: 12345
}
```

2. Add deploy script:
```bash
npm run deploy:newchain
```

## 📦 Dependencies

- **Hardhat** - Development environment
- **OpenZeppelin Contracts** - Secure contract standards
- **Ethers.js** - Ethereum library

## 🔗 Links

- [Axiom Protocol](https://axiom.network)
- [Documentation](https://docs.axiom.network)
- [GitHub](https://github.com/joker00099/Axiom-Protocol)

## 📄 License

MIT License - see LICENSE file for details
