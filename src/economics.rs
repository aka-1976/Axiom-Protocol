/// ECONOMICS — CONSENSUS MONETARY POLICY
/// Fixed total supply (forever)
pub const MAX_SUPPLY: u64 = 84_000_000_000_000_000;

/// Initial block reward
pub const INITIAL_REWARD: u64 = 50_000_000_000;

/// Blocks per halving (~4 years @ 15 min)
pub const HALVING_INTERVAL: u64 = 2_100_000;

/// Calculate block reward based on height
pub fn block_reward(block_height: u64, already_issued: u64) -> u64 {
    let halvings = block_height / HALVING_INTERVAL;

    let reward = INITIAL_REWARD >> halvings;

    if reward == 0 {
        return 0;
    }

    if already_issued + reward > MAX_SUPPLY {
        MAX_SUPPLY - already_issued
    } else {
        reward
    }
}
