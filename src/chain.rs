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

/// THE SOVEREIGN ANCHOR: Hardcoded from your 2026-01-11 solo mine.
pub const GENESIS_ANCHOR: &str = "7876d9aac11b1197474167b7485626bf535e551a21865c6264f07f614281298c";

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
    pub fn add_block(&mut self, block: Block, elapsed: u64) -> Result<(), &'static str> {
        // 1. DUPLICATE & INJECTION PROTECTION
        let block_hash = block.calculate_hash();
        if self.seen_hashes.contains(&block_hash) {
            return Err("Block already exists (Injection Attack thwarted)");
        }

        // 2. VALIDATE BLOCK STRUCTURE
        if block.parent != self.blocks.last().unwrap().hash() {
            return Err("Invalid parent hash");
        }

        if block.slot != self.blocks.len() as u64 {
            return Err("Invalid block slot");
        }

        // 3. VALIDATE VDF PROOF
        let expected_vdf = crate::main_helper::compute_vdf(
            crate::vdf::evaluate(block.parent, block.slot),
            self.difficulty as u32
        );
        if block.vdf_proof != expected_vdf {
            return Err("Invalid VDF proof");
        }

        // 4. VALIDATE POW
        if !block.meets_difficulty(self.difficulty) {
            return Err("Block doesn't meet difficulty requirement");
        }

        // 5. VALIDATE TRANSACTIONS
        for tx in &block.transactions {
            let sender_balance = self.state.balance(&tx.from);
            tx.validate(sender_balance)?;
        }

        // 6. VALIDATE ZK PASS FOR MINER
        if !crate::genesis::verify_zk_pass(&block.miner, &block.parent, &block.zk_proof) {
            return Err("Invalid miner ZK pass");
        }

        // 7. APPLY BLOCK
        self.seen_hashes.insert(block_hash);
        self.blocks.push(block.clone());

        // 8. UPDATE STATE
        let reward = economics::block_reward(block.slot, self.total_issued);
        if reward > 0 && block.miner != [0u8; 32] {
            self.state.credit(block.miner, reward);
            self.total_issued += reward;
        }

        for tx in &block.transactions {
            if self.state.apply_tx(tx).is_err() {
                // This shouldn't happen since we validated above
                return Err("Transaction application failed");
            }
        }

        // 9. ADJUST DIFFICULTY
        self.adjust_difficulty(elapsed);

        Ok(())
    }

    /// Adjust difficulty based on block time
    fn adjust_difficulty(&mut self, elapsed: u64) {
        // Simple difficulty adjustment
        if elapsed < TARGET_TIME {
            self.difficulty = self.difficulty.saturating_add(1);
        } else if elapsed > TARGET_TIME {
            self.difficulty = self.difficulty.saturating_sub(1).max(1);
        }
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

    /// Validate and add transaction to mempool (placeholder for now)
    pub fn validate_transaction(&self, tx: &Transaction) -> Result<(), &'static str> {
        let sender_balance = self.state.balance(&tx.from);
        tx.validate(sender_balance)
    }
}
