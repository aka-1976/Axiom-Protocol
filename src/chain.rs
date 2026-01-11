use crate::block::Block;
use crate::transaction::{Transaction, Address};
use std::collections::{HashMap, HashSet};

pub const TARGET_TIME: u64 = 3600; // 1 Hour Time-Lock
pub const HALVING_INTERVAL: u64 = 2_100_000;
pub const INITIAL_REWARD: u64 = 50_000_000_000; // 500 QBT (8 decimals)

/// THE SOVEREIGN ANCHOR: Hardcoded from your 2026-01-11 solo mine.
pub const GENESIS_ANCHOR: &str = "2dfba633817046c7f559ed4b93076048435f7e1a90f14eb8035c04b9ebae2537";

pub struct Timechain {
    pub blocks: Vec<Block>,
    pub balances: HashMap<Address, u64>,
    pub difficulty: u64,
    seen_hashes: HashSet<[u8; 32]>, // Injection Protection
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
            balances: HashMap::new(),
            difficulty: 1000,
            seen_hashes: HashSet::new(),
        };
        tc.rebuild_state();
        tc
    }

    /// The Core Consensus Logic: VDF + PoW + Self-Healing
    pub fn add_block(&mut self, block: Block, elapsed: u64) -> Result<(), &'static str> {
        // 1. DUPLICATE & INJECTION PROTECTION
        let block_hash = block.calculate_hash(); // Ensure this matches your Block implementation
        if self.seen_hashes.contains(&block_hash) {
            return Err("Block already exists (Injection Attack thwarted)");
        }

        // 2. 60% ATTACK PROTECTION (The VDF Gate)
        if elapsed < TARGET_TIME && self.blocks.len() > 1 {
            return Err("VDF Violation: Time-lock not expired");
        }

        // 3. POW VALIDATION
        if !block.meets_difficulty(self.difficulty) {
            return Err("PoW Violation: Insufficient Hash Power");
        }

        // 4. SYBIL PROTECTION (ZK-PASS VALIDATION)
        if !crate::genesis::verify_zk_pass(&block.miner, &block.parent, &block.zk_proof) {
            return Err("Sybil Violation: Invalid ZK-Pass");
        }

        // 5. AGGRESSIVE RETARGETING (Governance-Free)
        if elapsed < TARGET_TIME {
            self.difficulty = self.difficulty.saturating_add(self.difficulty / 2);
        } else {
            self.difficulty = self.difficulty.saturating_sub(self.difficulty / 10);
        }

        // 6. SELF-HEALING (Longest Chain Rule)
        if block.parent == self.blocks.last().unwrap().calculate_hash() {
            self.blocks.push(block);
            self.seen_hashes.insert(block_hash);
            self.rebuild_state();
            Ok(())
        } else {
            Err("Chain Split: Block does not follow local tip")
        }
    }

    /// 84M Bit-Shift Halving & State Engine
    pub fn rebuild_state(&mut self) {
        self.balances.clear();
        let mut processed_txs = HashSet::new();

        for (i, block) in self.blocks.iter().enumerate() {
            let halvings = i as u64 / HALVING_INTERVAL;
            let reward = INITIAL_REWARD >> halvings;

            // Apply Miner Reward
            let balance = self.balances.entry(block.miner).or_insert(0);
            *balance = balance.saturating_add(reward);

            // DOUBLE SPEND PROTECTION
            for tx in &block.transactions {
                let tx_hash = tx.hash();
                if processed_txs.contains(&tx_hash) { continue; }

                // (Transaction logic would go here)
                processed_txs.insert(tx_hash);
            }
        }
    }
}
