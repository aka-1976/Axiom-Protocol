use crate::block::Block;
use crate::transaction::{Transaction, Address};
use crate::state::State;
use crate::economics;
use std::collections::HashSet;

pub const TARGET_TIME: u64 = 1800; // 30 Minute Time-Lock (VDF)
pub const HALVING_INTERVAL: u64 = 2_100_000;
pub const INITIAL_REWARD: u64 = 50_000_000_000; // 500 AXM (8 decimals)
pub const MAX_SUPPLY: u64 = 124_000_000_000_000_000; // 124M AXM in smallest units
pub const DECIMALS: u32 = 8;

/// THE SOVEREIGN ANCHOR: Updated for V4.2.0 (Block struct now includes timestamp).
pub const GENESIS_ANCHOR: &str = "2b3ef0c4f235645a868eb66de324756e2dc91e7d2df99e54cc58bbed3a6e4070";

pub struct Timechain {
    pub blocks: Vec<Block>,
    pub state: State,
    pub difficulty: u64,
    seen_hashes: HashSet<[u8; 32]>, // Injection Protection
    pub total_issued: u64,
}

impl Timechain {
    pub fn new(genesis: Block) -> Self {
        // LOCKING MECHANISM:
        // Before creating the chain, verify the genesis block matches your anchor.
        let actual_hash = hex::encode(genesis.calculate_hash());
        if actual_hash != GENESIS_ANCHOR {
            panic!(
                "\nFATAL: Genesis Anchor Mismatch!\nExpected: {}\nFound:    {}\nProtocol integrity compromised. Shutdown.\n",
                GENESIS_ANCHOR, actual_hash
            );
        }

        let mut tc = Timechain {
            blocks: vec![genesis],
            state: State::new(),
            difficulty: 1000,
            seen_hashes: HashSet::new(),
            total_issued: 0,
        };
        tc.rebuild_state();
        tc
    }

    /// Restore a Timechain from previously-validated blocks loaded from
    /// persistent storage.
    ///
    /// Verifies the genesis anchor but skips per-block consensus
    /// validation (VDF, PoW, ZK) since these blocks were already accepted
    /// when first appended. State (balances, nonces, supply) is rebuilt
    /// deterministically from the block sequence.
    pub fn from_saved_blocks(saved_blocks: Vec<Block>) -> Result<Self, &'static str> {
        if saved_blocks.is_empty() {
            return Err("No blocks to restore");
        }

        // Verify genesis anchor
        let genesis_hash = hex::encode(saved_blocks[0].calculate_hash());
        if genesis_hash != GENESIS_ANCHOR {
            return Err("Genesis anchor mismatch on reload — chain file corrupted");
        }

        let mut tc = Timechain {
            blocks: saved_blocks,
            state: State::new(),
            difficulty: 1000,
            seen_hashes: HashSet::new(),
            total_issued: 0,
        };
        // Populate seen_hashes for injection protection
        for block in &tc.blocks {
            tc.seen_hashes.insert(block.calculate_hash());
        }
        tc.rebuild_state();
        Ok(tc)
    }

    /// Rebuild state from all blocks
    pub fn rebuild_state(&mut self) {
        self.state = State::new();
        self.total_issued = 0;

        for block in &self.blocks {
            // Process mining reward
            let reward = economics::block_reward(block.slot, self.total_issued);
            if reward > 0 && block.miner != [0u8; 32] {
                self.state.credit(block.miner, reward);
                self.total_issued += reward;
            }

            // Process transactions
            for tx in &block.transactions {
                if self.state.apply_tx(tx).is_ok() {
                    // Transaction successful
                }
            }
        }
    }

    /// The Core Consensus Logic: VDF + PoW + Self-Healing
    pub fn add_block(&mut self, block: Block) -> Result<(), &'static str> {
        // 1. DUPLICATE & INJECTION PROTECTION
        let block_hash = block.calculate_hash();
        if self.seen_hashes.contains(&block_hash) {
            return Err("Block already exists (Injection Attack thwarted)");
        }

        // 2. VALIDATE BLOCK STRUCTURE
        let prev_block = self.blocks.last().unwrap();
        if block.parent != prev_block.hash() {
            return Err("Invalid parent hash");
        }

        if block.slot != self.blocks.len() as u64 {
            return Err("Invalid block slot");
        }

        // 3. VALIDATE TIMESTAMP
        // Block timestamp must be ≥ parent's timestamp (no time travel).
        // Allow up to 2 minutes in the future to tolerate clock skew.
        let prev_ts = prev_block.timestamp;
        if block.timestamp < prev_ts {
            return Err("Block timestamp before parent");
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if block.timestamp > now + 120 {
            return Err("Block timestamp too far in the future");
        }

        // 4. VALIDATE VDF PROOF
        let expected_vdf = crate::main_helper::compute_vdf(
            crate::vdf::evaluate(block.parent, block.slot),
            self.difficulty as u32
        );
        if block.vdf_proof != expected_vdf {
            return Err("Invalid VDF proof");
        }

        // 5. VALIDATE POW
        if !block.meets_difficulty(self.difficulty) {
            return Err("Block doesn't meet difficulty requirement");
        }

        // 6. VALIDATE TRANSACTIONS
        for tx in &block.transactions {
            let sender_balance = self.state.balance(&tx.from);
            tx.validate(sender_balance)?;
        }

        // 7. VALIDATE ZK PASS FOR MINER
        if !crate::genesis::verify_zk_pass(&block.miner, &block.parent, &block.zk_proof) {
            return Err("Invalid miner ZK pass");
        }

        // 8. APPLY BLOCK
        self.seen_hashes.insert(block_hash);
        // Compute elapsed time from block timestamps.  min(1) prevents
        // division by zero in adjust_difficulty when two blocks carry the
        // same second-resolution timestamp (e.g. during testing or when
        // miners have synchronized clocks).
        let elapsed = block.timestamp.saturating_sub(prev_ts).max(1);
        self.blocks.push(block.clone());

        // 9. UPDATE STATE
        let reward = economics::block_reward(block.slot, self.total_issued);
        if reward > 0 && block.miner != [0u8; 32] {
            self.state.credit(block.miner, reward);
            self.total_issued += reward;
        }

        for tx in &block.transactions {
            if self.state.apply_tx(tx).is_err() {
                return Err("Transaction application failed");
            }
        }

        // 10. ADJUST DIFFICULTY based on actual block time
        self.adjust_difficulty(elapsed);

        Ok(())
    }

    /// Adjust difficulty based on block time using proportional adjustment.
    ///
    /// Uses pure integer arithmetic:
    ///   `new_difficulty = difficulty × TARGET_TIME / elapsed`
    /// clamped to [3/4, 5/4] of the current difficulty to prevent sudden
    /// jumps from outlier block times.
    fn adjust_difficulty(&mut self, elapsed: u64) {
        // Avoid division by zero; treat instant blocks as minimum 1 second
        let elapsed = elapsed.max(1);

        // Integer proportional: difficulty * TARGET_TIME / elapsed
        let raw = self.difficulty.saturating_mul(TARGET_TIME) / elapsed;

        // Clamp to [75%, 125%] of current difficulty
        let lower = self.difficulty.saturating_mul(3) / 4; // 75%
        let upper = self.difficulty.saturating_mul(5) / 4; // 125%
        let clamped = raw.max(lower).min(upper);

        // Enforce minimum difficulty of 1
        self.difficulty = clamped.max(1);
    }

    /// Get current balance for address
    pub fn balance(&self, address: &Address) -> u64 {
        self.state.balance(address)
    }

    /// Get supply information
    pub fn supply_info(&self) -> (u64, u64, f64) {
        let mined = self.total_issued;
        let remaining = MAX_SUPPLY.saturating_sub(mined);
        let percent = (mined as f64 / MAX_SUPPLY as f64) * 100.0;
        (mined, remaining, percent)
    }

    /// Format amount to AXM with decimals
    pub fn format_axm(amount: u64) -> String {
        let whole = amount / 10u64.pow(DECIMALS);
        let fractional = amount % 10u64.pow(DECIMALS);
        format!("{}.{:08}", whole, fractional)
    }

    /// Validate a transaction against the current chain state.
    pub fn validate_transaction(&self, tx: &Transaction) -> Result<(), &'static str> {
        let sender_balance = self.state.balance(&tx.from);
        tx.validate(sender_balance)
    }
}
