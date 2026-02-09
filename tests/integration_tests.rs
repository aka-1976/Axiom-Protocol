#[cfg(test)]
mod tests {
    use axiom_core::*;
    use axiom_core::block::Block;
    use axiom_core::chain::Timechain;
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
                miner: wallet.address,
                transactions: vec![],
                vdf_proof,
                zk_proof: zk_pass.clone(),
                nonce,
            };

            if block.meets_difficulty(chain.difficulty) {
                println!("Found valid nonce: {} for difficulty {}", nonce, chain.difficulty);
                if chain.add_block(block.clone(), 3600).is_ok() {
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
}