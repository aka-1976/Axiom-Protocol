use qubit_core::block::Block;
use qubit_core::state::State;
use qubit_core::transaction::Transaction;

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
    assert_eq!(Block::mining_reward(0), 50_000_000);
    assert_eq!(Block::mining_reward(210_000), 25_000_000);
    assert_eq!(Block::mining_reward(420_000), 12_500_000);
}
