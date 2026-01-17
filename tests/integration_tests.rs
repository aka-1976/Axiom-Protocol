#[cfg(test)]
mod tests {
    use qubit_core::*;
    use qubit_core::block::Block;
    use qubit_core::chain::Timechain;
    use qubit_core::genesis;

    #[test]
    fn test_transaction_creation() {
        let wallet = wallet::Wallet::load_or_create();
        let to_address = [1u8; 32];
        let amount = 100000000; // 1 QBT
        let fee = 1000000; // 0.01 QBT
        let nonce = 0;
        let balance = 200000000; // 2 QBT

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
        let wallet = wallet::Wallet::load_or_create();
        let to_address = [1u8; 32];
        let amount = 100000000; // 1 QBT
        let fee = 1000000; // 0.01 QBT
        let nonce = 0;
        let balance = 200000000; // 2 QBT

        let tx = wallet.create_transaction(to_address, amount, fee, nonce, balance).unwrap();

        println!("Transaction created: {:?}", tx);
        println!("ZK proof length: {}", tx.zk_proof.len());
        println!("Signature length: {}", tx.signature.len());

        // Test valid transaction
        let result = tx.validate(balance);
        println!("Validation result: {:?}", result);
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
        // Test initial reward
        let reward = economics::block_reward(0, 0);
        assert_eq!(reward, 50000000000); // 500 QBT

        // Test halving
        let reward_after_halving = economics::block_reward(2100000, 0);
        assert_eq!(reward_after_halving, 25000000000); // 250 QBT
    }

    #[test]
    fn test_wallet_balance() {
        let wallet = wallet::Wallet::load_or_create();
        let genesis = genesis::genesis();
        let chain = Timechain::new(genesis);

        let balance = wallet.get_balance(&chain);
        assert_eq!(balance, 0); // No rewards or transactions yet
    }
}