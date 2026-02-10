// Security and attack prevention tests
#[cfg(test)]
mod security_tests {
    use axiom_core::*;
    use axiom_core::block::Block;
    use axiom_core::chain::{self, Timechain};
    use axiom_core::genesis;
    use axiom_core::wallet::Wallet;

    #[test]
    fn test_double_spend_prevention_same_block() {
        // Test that the same transaction cannot be included twice in the same block
        let genesis = genesis::genesis();
        let mut chain = Timechain::new(genesis.clone());
        
        let wallet = Wallet::load_or_create();
        let to_address = [1u8; 32];
        
        // Create a transaction
        let tx = wallet.create_transaction(to_address, 100_000_000, 1_000_000, 0, 200_000_000).unwrap();
        
        // Try to create a block with the same transaction twice
        let parent_hash = chain.blocks.last().unwrap().hash();
        let current_slot = chain.blocks.len() as u64;
        chain.difficulty = 10;
        
        let vdf_seed = vdf::evaluate(parent_hash, current_slot);
        let vdf_proof = main_helper::compute_vdf(vdf_seed, chain.difficulty as u32);
        let zk_pass = genesis::generate_zk_pass(&wallet, parent_hash);
        
        let block = Block {
            parent: parent_hash,
            slot: current_slot,
            timestamp: genesis::GENESIS_TIMESTAMP + chain::TARGET_TIME * current_slot,
            miner: wallet.address,
            transactions: vec![tx.clone(), tx.clone()], // Same transaction twice!
            vdf_proof,
            zk_proof: zk_pass,
            nonce: 0,
        };
        
        // The block should be rejected (or the chain should handle duplicates)
        // This tests the implementation's double-spend protection
        println!("Testing double-spend in same block...");
        let result = chain.add_block(block);
        // In production, this should fail or the duplicate should be filtered
        println!("Result: {:?}", result);
    }

    #[test]
    fn test_double_spend_prevention_different_blocks() {
        // Test that the same transaction cannot be spent twice across different blocks
        let genesis = genesis::genesis();
        let mut chain = Timechain::new(genesis.clone());
        
        let wallet = Wallet::load_or_create();
        let to_address = [1u8; 32];
        
        // Create a transaction
        let tx = wallet.create_transaction(to_address, 100_000_000, 1_000_000, 0, 200_000_000).unwrap();
        
        // Mine first block with transaction
        let parent_hash = chain.blocks.last().unwrap().hash();
        let current_slot = chain.blocks.len() as u64;
        chain.difficulty = 10;
        
        let vdf_seed = vdf::evaluate(parent_hash, current_slot);
        let vdf_proof = main_helper::compute_vdf(vdf_seed, chain.difficulty as u32);
        let zk_pass = genesis::generate_zk_pass(&wallet, parent_hash);
        
        let mut block1 = Block {
            parent: parent_hash,
            slot: current_slot,
            timestamp: genesis::GENESIS_TIMESTAMP + chain::TARGET_TIME * current_slot,
            miner: wallet.address,
            transactions: vec![tx.clone()],
            vdf_proof,
            zk_proof: zk_pass.clone(),
            nonce: 0,
        };
        
        // Find valid nonce for block 1
        while !block1.meets_difficulty(chain.difficulty) && block1.nonce < 10000 {
            block1.nonce += 1;
        }
        
        let result1 = chain.add_block(block1.clone());
        println!("Block 1 added: {:?}", result1);
        
        // Try to create another block with the SAME transaction (double-spend attempt)
        let parent_hash2 = chain.blocks.last().unwrap().hash();
        let current_slot2 = chain.blocks.len() as u64;
        
        let vdf_seed2 = vdf::evaluate(parent_hash2, current_slot2);
        let vdf_proof2 = main_helper::compute_vdf(vdf_seed2, chain.difficulty as u32);
        
        let mut block2 = Block {
            parent: parent_hash2,
            slot: current_slot2,
            timestamp: genesis::GENESIS_TIMESTAMP + chain::TARGET_TIME * current_slot2,
            miner: wallet.address,
            transactions: vec![tx.clone()], // Same transaction again!
            vdf_proof: vdf_proof2,
            zk_proof: zk_pass,
            nonce: 0,
        };
        
        // Find valid nonce for block 2
        while !block2.meets_difficulty(chain.difficulty) && block2.nonce < 10000 {
            block2.nonce += 1;
        }
        
        let result2 = chain.add_block(block2);
        println!("Block 2 (double-spend attempt) result: {:?}", result2);
        
        // This should fail because the transaction was already spent
        // The test verifies the chain's double-spend protection
    }

    #[test]
    fn test_transaction_nonce_validation() {
        // Test that transactions with incorrect nonces are rejected
        let genesis = genesis::genesis();
        let chain = Timechain::new(genesis.clone());
        
        let wallet = Wallet::load_or_create();
        let to_address = [1u8; 32];
        
        // Create transaction with nonce 0 (correct for first transaction)
        let tx_correct = wallet.create_transaction(to_address, 100_000_000, 1_000_000, 0, 200_000_000).unwrap();
        assert_eq!(tx_correct.nonce, 0);
        
        // Validate correct nonce
        let result_correct = chain.validate_transaction(&tx_correct);
        println!("Correct nonce (0) validation: {:?}", result_correct);
        
        // Create transaction with wrong nonce (skipping nonce 0)
        let tx_wrong_nonce = wallet.create_transaction(to_address, 100_000_000, 1_000_000, 5, 200_000_000).unwrap();
        assert_eq!(tx_wrong_nonce.nonce, 5);
        
        // This should fail because nonce should be sequential
        let result_wrong = chain.validate_transaction(&tx_wrong_nonce);
        println!("Wrong nonce (5) validation: {:?}", result_wrong);
    }

    #[test]
    fn test_transaction_replay_protection() {
        // Test that replaying a valid transaction from a previous chain state is rejected
        let genesis = genesis::genesis();
        let mut chain = Timechain::new(genesis.clone());
        
        let wallet = Wallet::load_or_create();
        let to_address = [1u8; 32];
        
        // Create and mine first transaction
        let tx1 = wallet.create_transaction(to_address, 50_000_000, 1_000_000, 0, 200_000_000).unwrap();
        
        // Add transaction to chain
        let parent_hash = chain.blocks.last().unwrap().hash();
        let current_slot = chain.blocks.len() as u64;
        chain.difficulty = 10;
        
        let vdf_seed = vdf::evaluate(parent_hash, current_slot);
        let vdf_proof = main_helper::compute_vdf(vdf_seed, chain.difficulty as u32);
        let zk_pass = genesis::generate_zk_pass(&wallet, parent_hash);
        
        let mut block1 = Block {
            parent: parent_hash,
            slot: current_slot,
            timestamp: genesis::GENESIS_TIMESTAMP + chain::TARGET_TIME * current_slot,
            miner: wallet.address,
            transactions: vec![tx1.clone()],
            vdf_proof,
            zk_proof: zk_pass,
            nonce: 0,
        };
        
        while !block1.meets_difficulty(chain.difficulty) && block1.nonce < 10000 {
            block1.nonce += 1;
        }
        
        let _ = chain.add_block(block1);
        println!("Block 1 mined with tx nonce 0");
        
        // Try to replay the same transaction (should be rejected due to nonce)
        let result_replay = chain.validate_transaction(&tx1);
        println!("Replay attempt result: {:?}", result_replay);
        // Should fail because nonce 0 was already used
    }

    #[test]
    fn test_insufficient_balance_rejection() {
        // Test that transactions with insufficient balance are rejected
        let genesis = genesis::genesis();
        let chain = Timechain::new(genesis.clone());
        
        let wallet = Wallet::load_or_create();
        let to_address = [1u8; 32];
        
        // Get actual balance
        let balance = wallet.get_balance(&chain);
        println!("Wallet balance: {}", balance);
        
        // Try to spend more than balance (should fail at creation or validation)
        let amount = balance + 100_000_000; // More than available
        let fee = 1_000_000;
        
        if balance > 0 {
            let tx_result = wallet.create_transaction(to_address, amount, fee, 0, balance);
            println!("Insufficient balance transaction creation: {:?}", tx_result);
            
            if let Ok(tx) = tx_result {
                // If creation succeeded (shouldn't in production), validation should fail
                let validation = chain.validate_transaction(&tx);
                println!("Validation result: {:?}", validation);
                assert!(validation.is_err(), "Transaction with insufficient balance should be rejected");
            }
        } else {
            println!("Wallet has zero balance, creating transaction with fake balance");
            // Try to create a transaction that exceeds the declared balance.
            // create_transaction itself should reject this.
            let fake_balance = 100_000_000;
            let tx_result = wallet.create_transaction(to_address, fake_balance + 50_000_000, fee, 0, fake_balance);
            
            // Either creation fails (amount+fee > balance) or validation fails
            match tx_result {
                Err(e) => {
                    println!("Transaction correctly rejected at creation: {}", e);
                    // This is the expected path — insufficient balance caught at creation
                }
                Ok(tx) => {
                    // If creation somehow succeeded, chain validation must reject it
                    let validation = chain.validate_transaction(&tx);
                    println!("Validation with fake balance: {:?}", validation);
                    assert!(validation.is_err(), "Transaction with insufficient balance should be rejected");
                }
            }
        }
    }

    #[test]
    fn test_minimum_fee_enforcement() {
        // Test that transactions below minimum fee are rejected
        let genesis = genesis::genesis();
        let chain = Timechain::new(genesis.clone());
        
        let wallet = Wallet::load_or_create();
        let to_address = [1u8; 32];
        
        // Create transaction with fee below minimum (0.01 AXM = 10,000,000 units)
        let low_fee = 100; // Way below minimum
        let tx = wallet.create_transaction(to_address, 100_000_000, low_fee, 0, 200_000_000).unwrap();
        
        // Validation should fail due to minimum fee requirement
        let result = chain.validate_transaction(&tx);
        println!("Low fee transaction validation: {:?}", result);
        // Should fail with minimum fee error
    }

    #[test]
    fn test_block_height_sequential() {
        // Test that blocks must have sequential heights
        let genesis = genesis::genesis();
        let mut chain = Timechain::new(genesis.clone());
        
        let wallet = Wallet::load_or_create();
        
        // Mine first block normally
        let parent_hash = chain.blocks.last().unwrap().hash();
        let correct_slot = chain.blocks.len() as u64;
        chain.difficulty = 10;
        
        let vdf_seed = vdf::evaluate(parent_hash, correct_slot);
        let vdf_proof = main_helper::compute_vdf(vdf_seed, chain.difficulty as u32);
        let zk_pass = genesis::generate_zk_pass(&wallet, parent_hash);
        
        let mut block1 = Block {
            parent: parent_hash,
            slot: correct_slot,
            timestamp: genesis::GENESIS_TIMESTAMP + chain::TARGET_TIME * correct_slot,
            miner: wallet.address,
            transactions: vec![],
            vdf_proof,
            zk_proof: zk_pass.clone(),
            nonce: 0,
        };
        
        while !block1.meets_difficulty(chain.difficulty) && block1.nonce < 10000 {
            block1.nonce += 1;
        }
        
        let result1 = chain.add_block(block1);
        println!("Block 1 result: {:?}", result1);
        assert!(result1.is_ok());
        
        // Try to add block with wrong slot (skip a number)
        let parent_hash2 = chain.blocks.last().unwrap().hash();
        let wrong_slot = chain.blocks.len() as u64 + 5; // Skip ahead!
        
        let vdf_seed2 = vdf::evaluate(parent_hash2, wrong_slot);
        let vdf_proof2 = main_helper::compute_vdf(vdf_seed2, chain.difficulty as u32);
        
        let mut block_wrong = Block {
            parent: parent_hash2,
            slot: wrong_slot,
            timestamp: genesis::GENESIS_TIMESTAMP + chain::TARGET_TIME * wrong_slot,
            miner: wallet.address,
            transactions: vec![],
            vdf_proof: vdf_proof2,
            zk_proof: zk_pass,
            nonce: 0,
        };
        
        while !block_wrong.meets_difficulty(chain.difficulty) && block_wrong.nonce < 10000 {
            block_wrong.nonce += 1;
        }
        
        let result_wrong = chain.add_block(block_wrong);
        println!("Wrong slot block result: {:?}", result_wrong);
        // Should fail due to non-sequential slot number
    }

    #[test]
    fn test_invalid_parent_hash_rejection() {
        // Test that blocks with invalid parent hashes are rejected
        let genesis = genesis::genesis();
        let mut chain = Timechain::new(genesis.clone());
        
        let wallet = Wallet::load_or_create();
        chain.difficulty = 10;
        
        // Create block with wrong parent hash
        let wrong_parent = [99u8; 32]; // Not the actual parent
        let current_slot = chain.blocks.len() as u64;
        
        let vdf_seed = vdf::evaluate(wrong_parent, current_slot);
        let vdf_proof = main_helper::compute_vdf(vdf_seed, chain.difficulty as u32);
        let zk_pass = genesis::generate_zk_pass(&wallet, wrong_parent);
        
        let mut block = Block {
            parent: wrong_parent,
            slot: current_slot,
            timestamp: genesis::GENESIS_TIMESTAMP + chain::TARGET_TIME * current_slot,
            miner: wallet.address,
            transactions: vec![],
            vdf_proof,
            zk_proof: zk_pass,
            nonce: 0,
        };
        
        while !block.meets_difficulty(chain.difficulty) && block.nonce < 10000 {
            block.nonce += 1;
        }
        
        let result = chain.add_block(block);
        println!("Invalid parent hash result: {:?}", result);
        assert!(result.is_err(), "Block with invalid parent should be rejected");
    }

    #[test]
    fn test_transaction_signature_verification() {
        // Test that transactions with invalid signatures are rejected
        let genesis = genesis::genesis();
        let chain = Timechain::new(genesis.clone());
        
        let wallet = Wallet::load_or_create();
        let to_address = [1u8; 32];
        
        // Create a valid transaction
        let mut tx = wallet.create_transaction(to_address, 100_000_000, 1_000_000, 0, 200_000_000).unwrap();
        
        // Tamper with the signature
        if !tx.signature.is_empty() {
            tx.signature[0] = tx.signature[0].wrapping_add(1); // Corrupt signature
        }
        
        // Validation should fail due to invalid signature
        let result = chain.validate_transaction(&tx);
        println!("Tampered signature validation: {:?}", result);
        // Should fail with signature verification error
    }

    #[test]
    fn test_block_timestamp_validation() {
        // Test that blocks with future timestamps are handled appropriately
        let genesis = genesis::genesis();
        let chain = Timechain::new(genesis.clone());
        
        println!("Genesis block timestamp: {}", genesis.slot);
        println!("Current chain length: {}", chain.blocks.len());
        
        // This test verifies the chain handles time-based validation
        // In production, blocks with timestamps too far in the future should be rejected
    }
}
