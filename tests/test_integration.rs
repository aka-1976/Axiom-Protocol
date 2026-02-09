use axiom_core::block::Block;
use axiom_core::state::State;
use axiom_core::transaction::Transaction;

#[test]
fn test_block_validation_and_state() {
    let mut state = State::new();
    let from = [1u8; 32];
    let to = [2u8; 32];
    state.credit(from, 100_000_000);
    let tx = Transaction::new(from, to, 10_000, 100, 0, vec![1u8; 128], vec![1u8; 64]);
    assert!(state.apply_tx(&tx).is_ok());
    assert_eq!(state.balance(&from), 100_000_000 - 10_000 - 100);
    assert_eq!(state.balance(&to), 10_000);
}


#[test]
fn test_mining_reward_halving() {
    // AXIOM halving interval is 1,240,000 blocks (not Bitcoin's 210k or 840k)
    assert_eq!(Block::mining_reward(0), 50_000_000);                // 50 AXM initial
    assert_eq!(Block::mining_reward(1_240_000), 25_000_000);        // 25 AXM after 1st halving
    assert_eq!(Block::mining_reward(2_480_000), 12_500_000);        // 12.5 AXM after 2nd halving
}
