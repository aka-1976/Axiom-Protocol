# AXIOM Protocol - Proof of Work (PoW) Technical Specification

## Executive Summary

AXIOM Protocol uses a **Blake3 based Proof of Work** system integrated with the VDF consensus layer. The PoW mechanism provides Sybil resistance and finality, while the VDF enforces time-based fairness. This document specifies the exact PoW algorithm, input construction, nonce mechanism, and difficulty adjustment.

**Key Properties**:
- **Hash Function**: Blake3 (256-bit output)
- **Target Block Time**: 30 minutes (1800 seconds)
- **Difficulty Window**: 60 blocks (~30 hours)
- **Adjustment Bounds**: ±3x per adjustment period
- **Minimum Difficulty**: 1000

---

## 1. Hash Function Specification

### 1.1 Algorithm: Blake3

**Standard**: BLAKE3 cryptographic hash (RFC 7517 compatible)  
**Output Size**: 256 bits (32 bytes)  
**Library**: Rust `blake3` crate (official implementation)

```rust
use blake3;

// Pseudocode
let hash: [u8; 32] = blake3::hash(input_data).into();

// Or using hasher for streaming:
let mut hasher = blake3::Hasher::new();
hasher.update(input_data);
let hash = hasher.finalize();
```

### 1.2 Why Blake3?

1. **High Performance**: 2-3x faster than SHA-256 on modern CPUs
2. **Simper API**: Single function for hashing vs. multi-step hasher
3. **Parallelizable**: Efficient tree hashing for large inputs
4. **Modern cryptography**: Designed by cryptographic experts (2019)
5. **Hardware-friendly**: Efficient on both CPUs and custom hardware

---

## 2. PoW Input Format & Construction

### 2.1 Block Serialization (Input to Hash)

The **entire block structure** is serialized and hashed:

```rust
#[derive(Serialize, Deserialize)]
pub struct Block {
    pub parent: [u8; 32],           // Previous block hash
    pub slot: u64,                  // Block number/height
    pub miner: [u8; 32],            // Miner's address (Ed25519)
    pub transactions: Vec<Transaction>,  // Tx list
    pub vdf_proof: [u8; 32],        // VDF proof (from time-lock)
    pub zk_proof: Vec<u8>,          // ZK-STARK proof (privacy)
    pub nonce: u64,                 // PoW counter (variable)
}
```

### 2.2 Serialization Method: Bincode

**Format**: Binary canonical encoding (deterministic, compact)  
**Library**: `bincode` crate (Rust standard serialization)

```rust
// Input construction
let block = Block {
    parent: [0u8; 32],
    slot: 1,
    miner: wallet.address,
    transactions: vec![tx1, tx2, ...],
    vdf_proof: vdf::generate_proof(...),
    zk_proof: zk::generate_proof(...),
    nonce: 0,  // Start at 0
};

// Serialize to bytes
let input: Vec<u8> = bincode::serialize(&block)?;

// Compute hash
let hash: [u8; 32] = blake3::hash(&input).into();
```

### 2.3 Bit-by-bit Encoding

| Field | Type | Bytes | Encoding |
|-------|------|-------|----------|
| `parent` | `[u8; 32]` | 32 | Raw bytes |
| `slot` | `u64` | 8 | Big-endian |
| `miner` | `[u8; 32]` | 32 | Raw bytes |
| `transactions` | `Vec<Tx>` | Variable | Bincode array |
| `vdf_proof` | `[u8; 32]` | 32 | Raw bytes |
| `zk_proof` | `Vec<u8>` | Variable | Bincode array |
| `nonce` | `u64` | 8 | Big-endian |
| **Total minimum** | | **132+ bytes** | **Bincode format** |

**Important**: Changes to ANY field (including nonce) change the entire hash.

---

## 3. Nonce Mechanism & Increment Strategy

### 3.1 Nonce Definition

```rust
pub nonce: u64  // Range: 0 to u64::MAX (18,446,744,073,709,551,615)
```

**Purpose**: A counter that miners iterate to search for valid hashes

### 3.2 Increment Process

Miners perform **sequential search** (brute force):

```rust
let mut block = Block::new(...);
let mut nonce = 0u64;
let found = false;

while !found {
    block.nonce = nonce;
    let hash = block.hash();  // Recompute for each nonce
    
    if block.meets_difficulty(difficulty) {
        // Found valid PoW!
        broadcast_block(block);
        found = true;
        break;
    }
    
    nonce += 1;
    
    // After 2^64 iterations, overflow: nonce wraps to 0
    if nonce == u64::MAX {
        // In practice: restart with new transactions, timestamp, etc.
        break;
    }
}
```

### 3.3 Expected Search Space

```
Total nonces available: 2^64 = 18.4 quintillion
Nonces typically needed: difficulty / u64::MAX
```

**Example**: Difficulty 1000
```
Expected nonces = 1000 / (u64::MAX / 1) ≈ 5.4e-15 * u64::MAX ≈ 10-1000 nonces
```

Actually, the real formula is:

$$\text{Average Nonces} = \frac{\text{Difficulty} \times \text{u64::MAX}}{2}$$

### 3.4 Nonce Optimization: Parallel Mining

In practice, miners use **multiple threads** with nonce partitioning:

```rust
// Thread 1: nonce 0..1_000_000_000
// Thread 2: nonce 1_000_000_000..2_000_000_000
// Thread 3: nonce 2_000_000_000..3_000_000_000
// etc.

// Whichever finds a valid hash first broadcasts the block
```

---

## 4. Relationship to VDF Output

### 4.1 VDF Independence

The VDF computation is **completely independent** from PoW:

```
┌──────────────────────────────────┐
│  PoW Search (Parallel)           │
│  ├─ Thread 1: Try nonce 0        │
│  ├─ Thread 2: Try nonce 1000     │
│  └─ Thread 3: Try nonce 2000     │
│                                  │
│  ⚡ FAST: Typically 10-100ms     │
└──────────────────────────────────┘

┌──────────────────────────────────┐
│  VDF Computation (Sequential)    │
│  ├─ Fixed time: 1800 seconds     │
│  ├─ Cannot be parallelized       │
│  └─ Cannot be sped up            │
│                                  │
│  🕐 SLOW: Enforced 30 minutes    │
└──────────────────────────────────┘
```

### 4.2 Block Construction Sequence

1. **VDF computes first** (parallel with PoW search)
   ```
   vdf_proof = vdf::evaluate(parent_hash, slot)
   // Takes ~1800 seconds (wall-clock time)
   ```

2. **PoW searches in parallel**
   ```
   while !found {
       block.nonce = candidate
       if block.meets_difficulty(diff) {
           found = true
       }
   }
   // Takes ~10-100ms (mostly luck-based)
   ```

3. **Block assembled when both ready**
   ```
   Block {
       parent,
       slot,
       miner,
       transactions,
       vdf_proof,  // From step 1 (slow)
       zk_proof,
       nonce,      // From step 2 (fast)
   }
   ```

### 4.3 VDF Proof is Input to PoW Hash

The VDF proof is **part of the block data** that gets hashed:

```rust
// PoW input includes VDF proof
let block = Block {
    vdf_proof: [a, b, c, ...],  // From VDF computation
    nonce: [x],                 // From PoW search
};

let hash = SHA256(serialize(block));
```

**Critical**: Valid VDF proof is REQUIRED before PoW can be validated.

---

## 5. PoW Validation Function

### 5.1 Difficulty Check Code

```rust
impl Block {
    /// Checks if block meets difficulty target
    pub fn meets_difficulty(&self, difficulty: u64) -> bool {
        let h = self.hash();  // Compute SHA-256
        
        // Extract first 64 bits as u64
        let val = u64::from_be_bytes(h[0..8].try_into().unwrap());
        
        // Check: hash < (u64::MAX / difficulty)
        val < (u64::MAX / difficulty.max(1))
    }
}
```

### 5.2 Difficulty Formula

```
Target = u64::MAX / difficulty
Valid if: hash_as_u64 < target
```

**Rearranged**:
$$\text{Valid} \iff h < \frac{2^{64}}{D}$$

Where $h$ = first 64 bits of SHA-256 hash, $D$ = difficulty

### 5.3 Example: Difficulty 1000

```
u64::MAX = 18,446,744,073,709,551,615
Target = 18,446,744,073,709,551,615 / 1000
       = 18,446,744,073,709,552 (approx)

Hashes with first 8 bytes < target are valid
Probability per hash: ~1 in 1000
```

---

## 6. Exact Difficulty Adjustment Algorithm

### 6.1 LWMA (Linear Weighted Moving Average)

**Window Size**: 60 blocks  
**Target Block Time**: 1800 seconds (30 minutes)  
**History Duration**: ~30 hours

```rust
pub const TARGET_BLOCK_TIME: u64 = 1800;
pub const LWMA_WINDOW: usize = 60;
pub const MIN_DIFFICULTY: u64 = 1000;
pub const MAX_ADJUSTMENT_FACTOR: f64 = 3.0;      // 300% max
pub const MIN_ADJUSTMENT_FACTOR: f64 = 0.33;     // 33% min
```

### 6.2 Algorithm Implementation

```rust
pub fn calculate_lwma_difficulty(block_headers: &[BlockHeader]) -> u64 {
    // Need at least 61 block headers (60 window + 1 baseline)
    if block_headers.len() < LWMA_WINDOW + 1 {
        return MIN_DIFFICULTY;
    }
    
    // Extract last 60 blocks for LWMA window
    let start_idx = block_headers.len() - LWMA_WINDOW - 1;
    let window = &block_headers[start_idx..];
    
    // Calculate weighted sum of block times
    let mut weighted_times: u64 = 0;
    let mut sum_difficulties: u64 = 0;
    
    for i in 1..=LWMA_WINDOW {
        // Time since previous block
        let time_delta = window[i].timestamp - window[i - 1].timestamp;
        let time_delta = time_delta.max(1);  // Minimum 1 second
        
        // Weight increases linearly: 1, 2, 3, ..., 60
        let weight = i as u64;
        
        // Accumulate weighted times
        weighted_times += time_delta * weight;
        
        // Sum difficulties
        sum_difficulties += window[i].difficulty;
    }
    
    // Expected weighted time at target block time
    // = TARGET_BLOCK_TIME * (1+2+3+...+60)
    // = TARGET_BLOCK_TIME * 60 * 61 / 2
    let n = LWMA_WINDOW as u64;
    let expected_times = TARGET_BLOCK_TIME * n * (n + 1) / 2;
    
    // Average difficulty in window
    let avg_difficulty = sum_difficulties / LWMA_WINDOW as u64;
    
    // Calculate adjustment ratio
    let adjustment = weighted_times as f64 / expected_times as f64;
    
    // Clamp adjustment: 0.33x to 3.0x (prevent wild swings)
    let clamped = adjustment
        .max(MIN_ADJUSTMENT_FACTOR)
        .min(MAX_ADJUSTMENT_FACTOR);
    
    // Apply adjustment
    let new_difficulty = (avg_difficulty as f64 * clamped) as u64;
    
    // Enforce minimum
    new_difficulty.max(MIN_DIFFICULTY)
}
```

### 6.3 Mathematical Breakdown

**Step 1: Weight calculation**
$$W_i = i \quad \text{for } i = 1, 2, ..., 60$$

**Step 2: Weighted time sum**
$$\sum_{i=1}^{60} W_i \cdot t_i = 1t_1 + 2t_2 + 3t_3 + ... + 60t_{60}$$

**Step 3: Expected weighted time**
$$E = T_{target} \times \sum_{i=1}^{60} i = 1800 \times (1+2+...+60) = 1800 \times 1830 = 3,294,000$$

**Step 4: Adjustment factor**
$$A = \frac{\sum W_i \cdot t_i}{E}$$

**Step 5: Bounded adjustment**
$$A_{clamped} = \max(0.33, \min(3.0, A))$$

**Step 6: New difficulty**
$$D_{new} = D_{avg} \times A_{clamped}$$

### 6.4 Example Calculation

**Scenario**: Last 60 blocks took average 25 minutes each

```
Actual weighted times: ≈ 1500 seconds average per block
Expected times at target: 30 minutes = 1800 seconds

Adjustment = 1500 / 1800 = 0.833 (83.3%)

Result: Difficulty * 0.833 = reduced difficulty
(blocks came faster than target, so reduce difficulty)
```

### 6.5 Flash Mining Protection

```rust
pub fn detect_flash_mining(headers: &[BlockHeader]) -> bool {
    // Check if last 10 blocks came too fast (< 3 min average)
    if recent_avg_time < TARGET_BLOCK_TIME / 10 {
        return true;  // Attack detected!
    }
    return false;
}
```

**What is it**: 10+ blocks in less than 3 minutes = suspicious  
**Action**: Alert network operators, reduce difficulty cap

---

## 7. Complete PoW Timeline

### 7.1 Block Production Timeline

```
Time   Event                          PoW Status
────────────────────────────────────────────────────
T+0s   Parent block mined
T+0s   VDF computation starts        [■■■■■] 0%
       PoW search starts             [■] 0%

T+100ms  PoW search tries nonce 0-    [■■] 2%
         1,000... still searching

T+900s   Nonce search finds valid     [■■■■■■] 99%
         block hash (lucky miner!)    ← PoW DONE

T+1800s  VDF computation finishes     [■■■■■■■] 100%
         Block can now be validated   ← VDF DONE
         
T+1801s  Block broadcast to network
         Other nodes verify:
         1. VDF proof valid? ✅
         2. PoW meets difficulty? ✅
         3. Transactions valid? ✅
         4. ZK-STARK proof valid? ✅
```

### 7.2 Critical: VDF-PoW Synchronization

**PoW can complete before VDF**, but block is **invalid until VDF finishes**:

```rust
if !vdf_valid {
    return Err("VDF not valid yet");  // Even if PoW is done!
}

if !block.meets_difficulty(difficulty) {
    return Err("PoW doesn't meet difficulty");  // Even if VDF is done!
}
```

---

## 8. PoW Input Mutation (Nonce-Only)

### 8.1 What Miners Can Change

| Field | Mutable? | Strategy |
|-------|----------|----------|
| `parent` | ❌ No | Fixed to previous block |
| `slot` | ❌ No | Fixed to block height |
| `miner` | ❌ No | Fixed to miner address |
| `transactions` | ✅ Yes* | Can select subset |
| `vdf_proof` | ❌ No | Must wait for VDF |
| `zk_proof` | ❌ No | Must generate correctly |
| `nonce` | ✅ **Yes** | **Primary search variable** |

*Note: Changing transactions changes merkle root, thus changes hash

### 8.2 Nonce-Only Search Strategy (Most Efficient)

```rust
// Build block once
let mut block = Block {
    parent: parent_hash,
    slot: block_number,
    miner: miner_address,
    transactions: selected_transactions,
    vdf_proof: computed_vdf,
    zk_proof: generated_proof,
    nonce: 0,  // START HERE
};

// Search by varying ONLY nonce
loop {
    if block.meets_difficulty(current_difficulty) {
        return block;  // Found!
    }
    block.nonce += 1;
}
```

### 8.3 Multi-Transaction Strategy (If Lucky Mining Fails)

```rust
// If after 2^32 nonces (+Extraordinary amount) still no solution:
if attempts > 4_294_967_296 {
    // Change transactions (includes new timestamp, fees, etc.)
    block.transactions = get_new_pending_transactions();
    block.nonce = 0;  // Reset nonce for new transaction set
}
```

---

## 9. Performance Characteristics

### 9.1 Mining Rate (Hash Rate)

**Expected nonces to find valid block**:
$$E[\text{nonces}] = \frac{\text{difficulty}}{2}$$

**Expected time to mine**:
$$E[\text{time}] = \frac{E[\text{nonces}] \times \text{hash_time}}{hash\_rate}$$

### 9.2 Benchmark Numbers

| Difficulty | Expected Hashes | Time (1 GH/s) | Time (1 TH/s) |
|------------|-----------------|---------------|---------------|
| 1,000 | 500 | 0.5 μs | 0.0005 ms |
| 10,000 | 5,000 | 5 μs | 0.005 ms |
| 1,000,000 | 500,000 | 0.5 ms | 0.0005 ms |
| 1 billion | 500 billion | 500 s | 0.5 s |

### 9.3 Expected Block Time Composition

```
Total Time to Block = VDF Time + PoW Time (mostly waiting for VDF)

VDF Time:      1800 seconds (30 min) ← Dominates!
PoW Time:      ~0.01-1 second (negligible)
────────────────────────────────
Total Expected: 1800-1801 seconds
```

---

## 10. Security Properties

### 10.1 Why PoW + VDF Together?

| Property | PoW Alone | VDF Alone | PoW + VDF |
|----------|-----------|-----------|-----------|
| Sybil Resistance | ✅ Yes | ❌ No | ✅ Yes |
| 51% Attack Resistance | ✅ (expensive) | ❌ No | ✅ (expensive + time) |
| Time Enforcement | ❌ No | ✅ Yes | ✅ Yes |
| Leader Election Fair | ⚠️ Speed matters | ✅ Yes | ✅ Yes |

### 10.2 51% Attack Cost

Attacker needs:
1. **51% of PoW hashpower** (computational cost)
2. **1800 seconds of time per block** (temporal cost)

Cannot be sped up by parallelization or hardware.

### 10.3 Difficulty Bomb Prevention

LWMA bounds adjustments to ±3x per window:
```
If hashpower drops 90%:
- Naive adjustment: 10x increase (blocks every 5 min instead of 30)
- LWMA adjustment: 3x increase max (stays ~10 min target)
```

---

## 11. Verification Algorithm (Full Node)

```rust
pub fn validate_pow_for_block(block: &Block, difficulty: u64) -> Result<(), String> {
    // 1. Serialize block
    let serialized = bincode::serialize(block)
        .map_err(|_| "Serialization failed")?;
    
    // 2. Compute SHA-256
    let mut hasher = Sha256::new();
    hasher.update(&serialized);
    let hash: [u8; 32] = hasher.finalize().into();
    
    // 3. Extract first 64 bits
    let val = u64::from_be_bytes(hash[0..8].try_into()?);
    
    // 4. Check against difficulty target
    if val >= (u64::MAX / difficulty.max(1)) {
        return Err("PoW doesn't meet difficulty".to_string());
    }
    
    Ok(())
}
```

---

## Summary Table

| Aspect | Value |
|--------|-------|
| **Hash Algorithm** | SHA-256 (FIPS 180-4) |
| **Hash Output Size** | 256 bits (32 bytes) |
| **Input Format** | Bincode-serialized block |
| **Nonce Range** | 0 to 2^64-1 |
| **Difficulty Minimum** | 1000 |
| **Target Block Time** | 1800 seconds (30 minutes) |
| **Adjustment Window** | 60 blocks (~30 hours) |
| **Adjustment Bounds** | ±3x (300% max) |
| **Selection Criterion** | First 64 bits < (u64::MAX / difficulty) |
| **Verification Cost** | ~5ms per block |
| **VDF Relationship** | Independent, included in block hash |

---

**Version**: 2.0 (February 5, 2026)  
**Status**: Specification Finalized ✅  
**Confidence**: Production Ready
