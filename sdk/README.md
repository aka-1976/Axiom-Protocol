# AXIOM Protocol SDKs

Official SDKs for interacting with the Axiom blockchain in multiple programming languages.

## Available SDKs

### Python SDK
Located in `python/`

**Installation:**
```bash
pip install axiom-sdk
```

**Quick Example:**
```python
from axiom_sdk import AxiomClient, Wallet, axm_to_sats

client = AxiomClient("http://localhost:8332")
wallet = Wallet()
tx_hash = client.send(wallet, recipient, axm_to_sats(1.5), use_zk=True)
```

**Documentation:** [python/README.md](python/README.md)

---

### JavaScript/Node.js SDK
Located in `javascript/`

**Installation:**
```bash
npm install axiom-sdk
```

**Quick Example:**
```javascript
const { AxiomClient, Wallet, axmToSats } = require('axiom-sdk');

const client = new AxiomClient('http://localhost:8332');
const wallet = new Wallet();
const txHash = await client.send(wallet, recipient, axmToSats(1.5), 1000, true);
```

**Documentation:** [javascript/README.md](javascript/README.md)

---

### Rust SDK
Located in `rust/`

**Installation:**
```toml
[dependencies]
axiom-sdk = "1.0"
```

**Quick Example:**
```rust
use axiom_sdk::{AxiomClient, Wallet, axm_to_sats};

let client = AxiomClient::new("http://localhost:8332");
let wallet = Wallet::new();
let tx_hash = client.send(&wallet, recipient, axm_to_sats(1.5), 1000, true)?;
```

**Documentation:** [rust/README.md](rust/README.md)

---

## Common Features

All SDKs provide:

1. **Wallet Management**
   - Generate new wallets
   - Load existing wallets from private keys
   - Sign transactions and messages
   - Verify signatures

2. **Transaction Operations**
   - Create transactions
   - Sign transactions
   - Broadcast transactions to the network
   - Query transaction status

3. **Blockchain Queries**
   - Get account balances
   - Retrieve blocks by hash or index
   - Get transaction details
   - Query chain info (height, supply, difficulty)

4. **Advanced Features**
   - ZK-STARK proof generation for private transactions
   - VDF proof verification
   - Neural Guardian threat detection queries

## RPC Endpoints

All SDKs connect to a Axiom node via JSON-RPC. Default endpoint: `http://localhost:8332/rpc`

### Available RPC Methods

- `get_balance` - Get account balance
- `get_nonce` - Get account nonce
- `broadcast_transaction` - Submit transaction
- `get_transaction` - Get transaction by hash
- `get_block` - Get block by hash or index
- `get_latest_block` - Get latest block
- `get_chain_info` - Get blockchain statistics
- `generate_zk_proof` - Generate ZK-STARK proof
- `verify_vdf` - Verify VDF proof
- `neural_guardian_query` - Query threat detection

## Examples

### Transfer AXM

**Python:**
```python
from axiom_sdk import AxiomClient, Wallet, axm_to_sats

client = AxiomClient("http://localhost:8332")
wallet = Wallet("your_private_key_hex")

# Send 10 AXM to recipient
tx_hash = client.send(
    wallet=wallet,
    recipient="recipient_address_64_char_hex",
    amount=axm_to_sats(10.0),
    fee=1000,
    use_zk=False
)
print(f"Transaction sent: {tx_hash}")
```

**JavaScript:**
```javascript
const { AxiomClient, Wallet, axmToSats } = require('axiom-sdk');

const client = new AxiomClient('http://localhost:8332');
const wallet = new Wallet('your_private_key_hex');

// Send 10 AXM to recipient
const txHash = await client.send(
    wallet,
    'recipient_address_64_char_hex',
    axmToSats(10.0),
    1000,
    false
);
console.log(`Transaction sent: ${txHash}`);
```

**Rust:**
```rust
use axiom_sdk::{AxiomClient, Wallet, axm_to_sats};

let client = AxiomClient::new("http://localhost:8332");
let wallet = Wallet::from_private_key("your_private_key_hex".to_string());

// Send 10 AXM to recipient
let tx_hash = client.send(
    &wallet,
    "recipient_address_64_char_hex",
    axm_to_sats(10.0),
    1000,
    false
)?;
println!("Transaction sent: {}", tx_hash);
```

### Private Transaction with ZK-STARK

**Python:**
```python
# Private transaction using ZK-STARK proof
tx_hash = client.send(
    wallet=wallet,
    recipient=recipient_address,
    amount=axm_to_sats(5.0),
    use_zk=True  # Enable privacy
)
```

**JavaScript:**
```javascript
// Private transaction using ZK-STARK proof
const txHash = await client.send(
    wallet,
    recipientAddress,
    axmToSats(5.0),
    1000,
    true  // Enable privacy
);
```

**Rust:**
```rust
// Private transaction using ZK-STARK proof
let tx_hash = client.send(
    &wallet,
    recipient_address,
    axm_to_sats(5.0),
    1000,
    true  // Enable privacy
)?;
```

### Query Blockchain

**Python:**
```python
# Get chain info
info = client.get_chain_info()
print(f"Height: {info['height']}")
print(f"Supply: {info['total_supply']} satoshis")

# Get latest block
block = client.get_latest_block()
print(f"Block #{block.index} with {len(block.transactions)} txs")

# Get balance
balance = client.get_balance(wallet.address)
print(f"Balance: {balance / 100_000_000} AXM")
```

## Development

### Building from Source

**Python:**
```bash
cd sdk/python
pip install -e .
pytest tests/
```

**JavaScript:**
```bash
cd sdk/javascript
npm install
npm test
```

**Rust:**
```bash
cd sdk/rust
cargo build --release
cargo test
```

## Security Considerations

1. **Private Key Management**: Never hardcode private keys. Use environment variables or secure key management systems.

2. **HTTPS**: Always use HTTPS for production RPC endpoints to prevent man-in-the-middle attacks.

3. **ZK-STARKs**: Private transactions using ZK-STARKs hide amounts and balances but not sender/recipient addresses.

4. **Fee Estimation**: Set appropriate fees based on network conditions. Too low = slow confirmation.

## Support

- Documentation: [https://docs.axiom.network](https://docs.axiom.network)
- GitHub Issues: [https://github.com/Ghost-84M/Axiom-Protocol/issues](https://github.com/Ghost-84M/Axiom-Protocol/issues)
- Discord: [https://discord.gg/axiom](https://discord.gg/axiom)

## License

MIT License - see [LICENSE](../LICENSE) for details
