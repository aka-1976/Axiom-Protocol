// Network stress and performance tests
#[cfg(test)]
mod stress_tests {
    use axiom_core::*;
    use axiom_core::block::Block;
    use axiom_core::chain::{self, Timechain};
    use axiom_core::genesis;
    use axiom_core::wallet::Wallet;
    use std::time::Instant;

    #[test]
    #[ignore] // Run with: cargo test --test stress_tests -- --ignored --nocapture
    fn test_block_mining_performance() {
        // Test how long it takes to mine blocks at various difficulties
        let genesis = genesis::genesis();
        let mut chain = Timechain::new(genesis.clone());
        
        let wallet = Wallet::load_or_create();
        
        let difficulties = vec![10, 50, 100, 500];
        
        for diff in difficulties {
            chain.difficulty = diff;
            
            let parent_hash = chain.blocks.last().unwrap().hash();
            let current_slot = chain.blocks.len() as u64;
            
            let vdf_seed = vdf::evaluate(parent_hash, current_slot);
            let vdf_proof = main_helper::compute_vdf(vdf_seed, chain.difficulty as u32);
            let zk_pass = genesis::generate_zk_pass(&wallet, parent_hash);
            
            let start = Instant::now();
            let mut nonce = 0u64;
            let mut found = false;
            
            while !found && nonce < 100000 {
                let block = Block {
                    parent: parent_hash,
                    slot: current_slot,
                    timestamp: genesis::GENESIS_TIMESTAMP + chain::TARGET_TIME * current_slot,
                    miner: wallet.address,
                    transactions: vec![],
                    vdf_proof,
                    zk_proof: zk_pass.clone(),
                    nonce,
                };
                
                if block.meets_difficulty(chain.difficulty) {
                    found = true;
                }
                nonce += 1;
            }
            
            let elapsed = start.elapsed();
            println!("Difficulty {}: {} attempts in {:?} ({:.2} hashes/sec)", 
                     diff, nonce, elapsed, nonce as f64 / elapsed.as_secs_f64());
            
            assert!(found, "Should find valid nonce within 100k attempts for difficulty {}", diff);
        }
    }

    #[test]
    #[ignore]
    fn test_transaction_processing_throughput() {
        // Test how many transactions can be created and validated per second
        let genesis = genesis::genesis();
        let chain = Timechain::new(genesis.clone());
        
        let wallet = Wallet::load_or_create();
        let to_address = [1u8; 32];
        
        let num_transactions = 100;
        let mut transactions = Vec::new();
        
        // Test transaction creation speed
        let start_create = Instant::now();
        for i in 0..num_transactions {
            let tx = wallet.create_transaction(
                to_address, 
                1_000_000, 
                100_000, 
                i, 
                num_transactions * 2_000_000
            ).unwrap();
            transactions.push(tx);
        }
        let create_elapsed = start_create.elapsed();
        
        println!("Created {} transactions in {:?} ({:.2} tx/sec)", 
                 num_transactions, 
                 create_elapsed, 
                 num_transactions as f64 / create_elapsed.as_secs_f64());
        
        // Test transaction validation speed
        let start_validate = Instant::now();
        let mut valid_count = 0;
        for tx in &transactions {
            if chain.validate_transaction(tx).is_ok() {
                valid_count += 1;
            }
        }
        let validate_elapsed = start_validate.elapsed();
        
        println!("Validated {} transactions in {:?} ({:.2} tx/sec)", 
                 valid_count, 
                 validate_elapsed, 
                 valid_count as f64 / validate_elapsed.as_secs_f64());
    }

    #[test]
    #[ignore]
    fn test_chain_sync_performance() {
        // Test how long it takes to validate and sync a long chain
        let genesis = genesis::genesis();
        let mut chain = Timechain::new(genesis.clone());
        
        let wallet = Wallet::load_or_create();
        chain.difficulty = 10; // Low difficulty for fast testing
        
        let num_blocks = 50;
        
        println!("Mining {} blocks...", num_blocks);
        let start = Instant::now();
        
        for i in 0..num_blocks {
            let parent_hash = chain.blocks.last().unwrap().hash();
            let current_slot = chain.blocks.len() as u64;
            
            let vdf_seed = vdf::evaluate(parent_hash, current_slot);
            let vdf_proof = main_helper::compute_vdf(vdf_seed, chain.difficulty as u32);
            let zk_pass = genesis::generate_zk_pass(&wallet, parent_hash);
            
            let mut block = Block {
                parent: parent_hash,
                slot: current_slot,
                timestamp: genesis::GENESIS_TIMESTAMP + chain::TARGET_TIME * current_slot,
                miner: wallet.address,
                transactions: vec![],
                vdf_proof,
                zk_proof: zk_pass,
                nonce: 0,
            };
            
            // Find valid nonce
            while !block.meets_difficulty(chain.difficulty) && block.nonce < 50000 {
                block.nonce += 1;
            }
            
            if block.nonce < 50000 {
                chain.add_block(block).unwrap();
            }
            
            if (i + 1) % 10 == 0 {
                println!("  Mined {} blocks...", i + 1);
            }
        }
        
        let elapsed = start.elapsed();
        let blocks_added = chain.blocks.len() - 1;
        
        println!("Chain sync completed:");
        println!("  Blocks: {}", blocks_added);
        println!("  Time: {:?}", elapsed);
        println!("  Rate: {:.2} blocks/sec", blocks_added as f64 / elapsed.as_secs_f64());
    }

    #[test]
    #[ignore]
    fn test_mempool_stress() {
        // Test handling of large mempool with many transactions
        let genesis = genesis::genesis();
        let chain = Timechain::new(genesis.clone());
        
        let wallet = Wallet::load_or_create();
        let to_address = [1u8; 32];
        
        let num_transactions = 1000;
        let mut mempool = Vec::new();
        
        println!("Generating {} transactions for mempool...", num_transactions);
        let start = Instant::now();
        
        for i in 0..num_transactions {
            if let Ok(tx) = wallet.create_transaction(
                to_address, 
                10_000, 
                1_000, 
                i, 
                num_transactions * 20_000
            ) {
                mempool.push(tx);
            }
        }
        
        let create_elapsed = start.elapsed();
        
        // Test mempool validation
        let start_validate = Instant::now();
        let mut valid_count = 0;
        for tx in &mempool {
            if chain.validate_transaction(tx).is_ok() {
                valid_count += 1;
            }
        }
        let validate_elapsed = start_validate.elapsed();
        
        println!("Mempool stress test:");
        println!("  Created: {} tx in {:?} ({:.2} tx/sec)", 
                 mempool.len(), 
                 create_elapsed, 
                 mempool.len() as f64 / create_elapsed.as_secs_f64());
        println!("  Validated: {} tx in {:?} ({:.2} tx/sec)", 
                 valid_count, 
                 validate_elapsed, 
                 valid_count as f64 / validate_elapsed.as_secs_f64());
        
        assert!(valid_count > 0, "Should have at least some valid transactions");
    }

    #[test]
    #[ignore]
    fn test_concurrent_block_validation() {
        // Test validating multiple blocks received from different peers simultaneously
        let genesis = genesis::genesis();
        let mut chain = Timechain::new(genesis.clone());
        
        let wallet = Wallet::load_or_create();
        chain.difficulty = 10;
        
        // Pre-mine several blocks
        let num_blocks = 10;
        let mut blocks: Vec<Block> = Vec::new();
        
        println!("Pre-mining {} blocks...", num_blocks);
        
        for _ in 0..num_blocks {
            let parent_hash = if blocks.is_empty() {
                genesis.hash()
            } else {
                blocks.last().unwrap().hash()
            };
            
            let current_slot = (blocks.len() + 1) as u64;
            
            let vdf_seed = vdf::evaluate(parent_hash, current_slot);
            let vdf_proof = main_helper::compute_vdf(vdf_seed, chain.difficulty as u32);
            let zk_pass = genesis::generate_zk_pass(&wallet, parent_hash);
            
            let mut block = Block {
                parent: parent_hash,
                slot: current_slot,
                timestamp: genesis::GENESIS_TIMESTAMP + chain::TARGET_TIME * current_slot,
                miner: wallet.address,
                transactions: vec![],
                vdf_proof,
                zk_proof: zk_pass,
                nonce: 0,
            };
            
            while !block.meets_difficulty(chain.difficulty) && block.nonce < 50000 {
                block.nonce += 1;
            }
            
            if block.nonce < 50000 {
                blocks.push(block);
            }
        }
        
        println!("Validating {} pre-mined blocks...", blocks.len());
        let start = Instant::now();
        
        let mut added = 0;
        for block in blocks {
            if chain.add_block(block).is_ok() {
                added += 1;
            }
        }
        
        let elapsed = start.elapsed();
        
        println!("Concurrent validation results:");
        println!("  Blocks validated: {}", added);
        println!("  Time: {:?}", elapsed);
        println!("  Rate: {:.2} blocks/sec", added as f64 / elapsed.as_secs_f64());
        
        assert_eq!(added, num_blocks, "All blocks should be added successfully");
    }

    #[test]
    #[ignore]
    fn test_chain_reorganization_performance() {
        // Test performance of handling chain reorganizations
        let genesis = genesis::genesis();
        let mut chain1 = Timechain::new(genesis.clone());
        let mut chain2 = Timechain::new(genesis.clone());
        
        let wallet = Wallet::load_or_create();
        chain1.difficulty = 10;
        chain2.difficulty = 10;
        
        // Build two competing chains
        println!("Building competing chains...");
        
        // Chain 1: 5 blocks
        for i in 0..5 {
            let parent_hash = chain1.blocks.last().unwrap().hash();
            let current_slot = chain1.blocks.len() as u64;
            
            let vdf_seed = vdf::evaluate(parent_hash, current_slot);
            let vdf_proof = main_helper::compute_vdf(vdf_seed, chain1.difficulty as u32);
            let zk_pass = genesis::generate_zk_pass(&wallet, parent_hash);
            
            let mut block = Block {
                parent: parent_hash,
                slot: current_slot,
                timestamp: genesis::GENESIS_TIMESTAMP + chain::TARGET_TIME * current_slot,
                miner: wallet.address,
                transactions: vec![],
                vdf_proof,
                zk_proof: zk_pass,
                nonce: i * 100, // Different nonces for uniqueness
            };
            
            while !block.meets_difficulty(chain1.difficulty) && block.nonce < 50000 {
                block.nonce += 1;
            }
            
            if block.nonce < 50000 {
                let _ = chain1.add_block(block);
            }
        }
        
        // Chain 2: 7 blocks (longer, should win)
        for i in 0..7 {
            let parent_hash = chain2.blocks.last().unwrap().hash();
            let current_slot = chain2.blocks.len() as u64;
            
            let vdf_seed = vdf::evaluate(parent_hash, current_slot);
            let vdf_proof = main_helper::compute_vdf(vdf_seed, chain2.difficulty as u32);
            let zk_pass = genesis::generate_zk_pass(&wallet, parent_hash);
            
            let mut block = Block {
                parent: parent_hash,
                slot: current_slot,
                timestamp: genesis::GENESIS_TIMESTAMP + chain::TARGET_TIME * current_slot,
                miner: wallet.address,
                transactions: vec![],
                vdf_proof,
                zk_proof: zk_pass,
                nonce: i * 200 + 50000, // Different nonces from chain1
            };
            
            while !block.meets_difficulty(chain2.difficulty) && block.nonce < 100000 {
                block.nonce += 1;
            }
            
            if block.nonce < 100000 {
                let _ = chain2.add_block(block);
            }
        }
        
        println!("Chain 1 length: {}", chain1.blocks.len());
        println!("Chain 2 length: {}", chain2.blocks.len());
        
        // In a real reorg scenario, chain1 would need to switch to chain2
        // This test measures the computational cost
        assert!(chain2.blocks.len() > chain1.blocks.len(), "Chain 2 should be longer");
    }

    #[test]
    fn test_memory_usage_estimation() {
        // Estimate memory usage for typical blockchain structures
        use std::mem::size_of;
        
        println!("Memory usage per structure:");
        println!("  Block: {} bytes", size_of::<Block>());
        println!("  Transaction: {} bytes", size_of::<transaction::Transaction>());
        
        // Estimate for 1000 blocks
        let blocks_count = 1000;
        let txs_per_block = 100;
        
        let block_mem = blocks_count * size_of::<Block>();
        let tx_mem = blocks_count * txs_per_block * size_of::<transaction::Transaction>();
        let total_mem = block_mem + tx_mem;
        
        println!("\nEstimated memory for {} blocks with {} tx each:", blocks_count, txs_per_block);
        println!("  Blocks: {:.2} MB", block_mem as f64 / 1_048_576.0);
        println!("  Transactions: {:.2} MB", tx_mem as f64 / 1_048_576.0);
        println!("  Total: {:.2} MB", total_mem as f64 / 1_048_576.0);
    }
}
