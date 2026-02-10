#[cfg(test)]
mod tests {
    use axiom_core::*;
    use axiom_core::block::Block;
    use axiom_core::chain::{self, Timechain};
    use axiom_core::genesis;
    use axiom_core::wallet::Wallet;
    use axiom_core::economics::block_reward;
    use axiom_core::vdf;
    use axiom_core::main_helper;

    #[test]
    fn test_transaction_creation() {
        let wallet = Wallet::load_or_create();
        let to_address = [1u8; 32];
        let amount = 100000000; // 1 AXM
        let fee = 1000000; // 0.01 AXM
        let nonce = 0;
        let balance = 200000000; // 2 AXM

        let tx = wallet.create_transaction(to_address, amount, fee, nonce, balance).unwrap();

        assert_eq!(tx.from, wallet.address);
        assert_eq!(tx.to, to_address);
        assert_eq!(tx.amount, amount);
        assert_eq!(tx.fee, fee);
        assert_eq!(tx.nonce, nonce);
        assert!(!tx.zk_proof.is_empty());
        assert!(!tx.signature.is_empty());
    }

    #[test]
    fn test_transaction_validation() {
        let wallet = Wallet::load_or_create();
        let to_address = [1u8; 32];
        let amount = 100000000; // 1 AXM
        let fee = 1000000; // 0.01 AXM
        let nonce = 0;
        let balance = 200000000; // 2 AXM

        let tx = wallet.create_transaction(to_address, amount, fee, nonce, balance).unwrap();

        println!("Transaction created: {:?}", tx);
        println!("ZK proof length: {}", tx.zk_proof.len());
        println!("Signature length: {}", tx.signature.len());

        // Test valid transaction
        let result = tx.validate(balance);
        println!("Validation result: {:?}", result);
        if let Err(e) = &result {
            println!("[TEST DEBUG] Validation error: {}", e);
        }
        assert!(result.is_ok());

        // Test insufficient balance
        assert!(tx.validate(amount + fee - 1).is_err());
    }

    #[test]
    fn test_block_creation() {
        let parent = [0u8; 32];
        let slot = 1;
        let miner = [1u8; 32];
        let transactions = vec![];
        let vdf_proof = [2u8; 32];
        let zk_proof = vec![3u8; 128];
        let nonce = 42;

        let block = Block::new(parent, slot, miner, transactions, vdf_proof, zk_proof, nonce);

        assert_eq!(block.parent, parent);
        assert_eq!(block.slot, slot);
        assert_eq!(block.miner, miner);
        assert_eq!(block.nonce, nonce);
    }

    #[test]
    fn test_block_hash() {
        let block = genesis::genesis();
        let hash = block.hash();
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_chain_initialization() {
        let genesis = genesis::genesis();
        let chain = Timechain::new(genesis);
        assert_eq!(chain.blocks.len(), 1);
        assert_eq!(chain.difficulty, 1000);
    }

    #[test]
    fn test_economics() {
        // Test initial reward (50 AXM = 5,000,000,000 in smallest units)
        let reward = block_reward(0, 0);
        assert_eq!(reward, 5_000_000_000); // 50 AXM

        // Test halving at 1,240,000 blocks (25 AXM = 2,500,000,000 in smallest units)
        let reward_after_halving = block_reward(1_240_000, 0);
        assert_eq!(reward_after_halving, 2_500_000_000); // 25 AXM
    }

    #[test]
    fn test_wallet_balance() {
        let wallet = Wallet::load_or_create();
        let genesis = genesis::genesis();
        let chain = Timechain::new(genesis);

        let balance = wallet.get_balance(&chain);
        assert_eq!(balance, 0); // No rewards or transactions yet
    }

    #[test]
    fn test_mining_simulation() {
        let genesis = genesis::genesis();
        let mut chain = Timechain::new(genesis.clone());

        // Create a wallet for mining
        let wallet = Wallet::load_or_create();

        // Simulate mining a block
        let parent_hash = chain.blocks.last().unwrap().hash();
        let current_slot = chain.blocks.len() as u64;

        // Use low difficulty for testing
        chain.difficulty = 10;

        let vdf_seed = vdf::evaluate(parent_hash, current_slot);
        let vdf_proof = main_helper::compute_vdf(vdf_seed, chain.difficulty as u32);
        let zk_pass = genesis::generate_zk_pass(&wallet, parent_hash);

        // Try to find a valid nonce
        let mut nonce = 0u64;
        let mut found = false;

        while !found && nonce < 10000 {
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
                println!("Found valid nonce: {} for difficulty {}", nonce, chain.difficulty);
                if chain.add_block(block.clone()).is_ok() {
                    println!("Block added successfully!");
                    found = true;
                } else {
                    println!("Block validation failed");
                }
            }
            nonce += 1;
        }

        assert!(found, "Should find a valid nonce within 10000 attempts");
        assert_eq!(chain.blocks.len(), 2, "Chain should have 2 blocks after mining");
    }

    #[test]
    fn test_signature_verification_rejects_invalid() {
        // A transaction with a non-empty but invalid signature must be rejected
        use axiom_core::transaction::Transaction;
        let tx = Transaction::new(
            [1u8; 32],
            [2u8; 32],
            100,
            10,
            0,
            vec![0u8; 128], // mining-format ZK proof
            vec![0xFFu8; 64], // invalid 64-byte signature
        );
        let result = tx.validate(1000);
        assert!(result.is_err(), "Invalid signature must be rejected");
    }

    #[test]
    fn test_signature_verification_accepts_valid() {
        // Verify that Wallet::verify_transaction_signature accepts a
        // properly signed transaction (independent of ZK proof verification).
        let wallet = Wallet::load_or_create();
        let to_address = [42u8; 32];
        let balance = 500_000_000u64;
        let amount = 100_000_000u64;
        let fee = 1_000_000u64;
        let nonce = 0u64;

        let tx = wallet.create_transaction(to_address, amount, fee, nonce, balance).unwrap();
        // Signature must be 64 bytes (Ed25519)
        assert_eq!(tx.signature.len(), 64, "Signature must be 64 bytes");
        // Direct signature verification must succeed
        let sig_valid = Wallet::verify_transaction_signature(&tx).unwrap();
        assert!(sig_valid, "Ed25519 signature verification must pass for a wallet-signed transaction");
    }

    #[test]
    fn test_mining_proof_verification() {
        // verify_zk_pass must accept valid proofs and reject invalid ones
        let wallet = Wallet::load_or_create();
        let parent_hash = [0u8; 32];
        let proof = genesis::generate_zk_pass(&wallet, parent_hash);

        // Valid proof should pass
        assert!(genesis::verify_zk_pass(&wallet.address, &parent_hash, &proof));

        // Empty proof should fail
        assert!(!genesis::verify_zk_pass(&wallet.address, &parent_hash, &[]));

        // Wrong length should fail
        assert!(!genesis::verify_zk_pass(&wallet.address, &parent_hash, &[0u8; 64]));

        // Zero miner address should fail
        assert!(!genesis::verify_zk_pass(&[0u8; 32], &parent_hash, &proof));

        // All-zero proof should fail (no secret commitment)
        let bad_proof = vec![0u8; 128];
        assert!(!genesis::verify_zk_pass(&wallet.address, &parent_hash, &bad_proof));

        // Proof with non-zero secret commitment but invalid public commitment should fail
        let mut bad_proof = vec![0u8; 128];
        bad_proof[0] = 1; // fake secret commitment
        assert!(!genesis::verify_zk_pass(&wallet.address, &parent_hash, &bad_proof),
            "Proof with incorrect public commitment must be rejected");

        // Proof verified against wrong parent hash should fail
        let wrong_parent = [0xFFu8; 32];
        assert!(!genesis::verify_zk_pass(&wallet.address, &wrong_parent, &proof),
            "Proof verified against wrong parent hash must be rejected");

        // Proof verified against wrong miner address should fail
        let wrong_miner = [0xABu8; 32];
        assert!(!genesis::verify_zk_pass(&wrong_miner, &parent_hash, &proof),
            "Proof verified against wrong miner address must be rejected");
    }

    /// End-to-end integration: wallet → ZK proof → mining → chain validation →
    /// transaction → STARK proof → persistence → reload → state rebuild.
    #[test]
    fn test_end_to_end_pipeline() {
        // 1. WALLET CREATION
        let wallet = Wallet::load_or_create();
        assert_eq!(wallet.address.len(), 32);
        assert_ne!(wallet.address, [0u8; 32], "Address must not be zero");

        // 2. GENESIS & CHAIN INIT
        let genesis_block = genesis::genesis();
        let genesis_hash = genesis_block.calculate_hash();
        assert_eq!(hex::encode(genesis_hash), axiom_core::chain::GENESIS_ANCHOR,
            "Genesis anchor must match hardcoded value");

        let mut chain = Timechain::new(genesis_block.clone());
        assert_eq!(chain.blocks.len(), 1);

        // 3. VDF COMPUTATION & PROOF
        let parent_hash = chain.blocks.last().unwrap().hash();
        let current_slot = chain.blocks.len() as u64;
        let vdf_seed = vdf::evaluate(parent_hash, current_slot);
        assert_ne!(vdf_seed, parent_hash, "VDF seed must differ from parent hash");

        // Use low difficulty for testing
        chain.difficulty = 10;
        let vdf_proof = main_helper::compute_vdf(vdf_seed, chain.difficulty as u32);
        // VDF must be deterministic
        let vdf_proof2 = main_helper::compute_vdf(vdf_seed, chain.difficulty as u32);
        assert_eq!(vdf_proof, vdf_proof2, "VDF must be deterministic");

        // 4. ZK MINING PROOF GENERATION & VERIFICATION
        let zk_pass = genesis::generate_zk_pass(&wallet, parent_hash);
        assert_eq!(zk_pass.len(), 128, "Mining proof must be 128 bytes");
        assert!(genesis::verify_zk_pass(&wallet.address, &parent_hash, &zk_pass),
            "Mining proof must pass verification against correct address");
        assert!(!genesis::verify_zk_pass(&[0xABu8; 32], &parent_hash, &zk_pass),
            "Mining proof must fail against wrong address");

        // 5. MINING (PoW nonce search)
        let mut found_nonce = None;
        for nonce in 0..50_000u64 {
            let candidate = Block {
                parent: parent_hash,
                slot: current_slot,
                timestamp: genesis::GENESIS_TIMESTAMP + chain::TARGET_TIME * current_slot,
                miner: wallet.address,
                transactions: vec![],
                vdf_proof,
                zk_proof: zk_pass.clone(),
                nonce,
            };
            if candidate.meets_difficulty(chain.difficulty) {
                found_nonce = Some(nonce);
                break;
            }
        }
        let nonce = found_nonce.expect("Must find valid nonce within 50000 attempts");

        // 6. BLOCK VALIDATION & CHAIN ADDITION
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
        let result = chain.add_block(block.clone());
        assert!(result.is_ok(), "Block must be accepted by chain: {:?}", result.err());
        assert_eq!(chain.blocks.len(), 2);

        // 7. MINING REWARD CREDITED
        let miner_balance = chain.balance(&wallet.address);
        let expected_reward = block_reward(1, 0);
        assert_eq!(miner_balance, expected_reward,
            "Miner must receive block reward after mining");

        // 8. PERSISTENCE — save and reload chain
        axiom_core::storage::save_chain(&chain.blocks);
        let loaded = axiom_core::storage::load_chain();
        assert!(loaded.is_some(), "Chain must be loadable from disk");
        let loaded_blocks = loaded.unwrap();
        assert_eq!(loaded_blocks.len(), 2, "Loaded chain must have 2 blocks");

        // 9. STATE REBUILD — rebuild chain from loaded blocks
        let rebuilt_chain = Timechain::from_saved_blocks(loaded_blocks)
            .expect("Chain must rebuild from saved blocks");
        assert_eq!(rebuilt_chain.blocks.len(), 2);
        let rebuilt_balance = rebuilt_chain.balance(&wallet.address);
        assert_eq!(rebuilt_balance, expected_reward,
            "Balance must be preserved after state rebuild");

        // 10. TRANSACTION WITH ZK-STARK PROOF
        let recipient = [42u8; 32];
        let amount = 100_000_000u64; // 1 AXM
        let fee = 1_000_000u64;      // 0.01 AXM
        let tx = wallet.create_transaction(recipient, amount, fee, 0, miner_balance)
            .expect("Transaction creation must succeed");

        // Verify all transaction components
        assert_eq!(tx.from, wallet.address);
        assert_eq!(tx.to, recipient);
        assert!(!tx.zk_proof.is_empty(), "ZK proof must be present");
        assert_eq!(tx.signature.len(), 64, "Ed25519 signature must be 64 bytes");

        // Ed25519 signature verification
        let sig_ok = Wallet::verify_transaction_signature(&tx).unwrap();
        assert!(sig_ok, "Signature must verify");

        // Full transaction validation (ZK proof + signature + balance)
        let tx_result = tx.validate(miner_balance);
        assert!(tx_result.is_ok(), "Transaction validation must pass: {:?}", tx_result.err());

        // Insufficient balance must be rejected
        assert!(tx.validate(amount).is_err(),
            "Transaction with insufficient balance must be rejected");

        // Clean up test file
        let _ = std::fs::remove_file("axiom_chain.dat");
    }
}