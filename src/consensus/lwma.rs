// src/consensus/lwma.rs - The Shield: LWMA Difficulty Guard
// Linear Weighted Moving Average prevents Flash Mining and 10M-miner scenarios

use num_bigint::BigUint;
use num_traits::{One, Zero, ToPrimitive};

/// Target block time: 1 hour (3600 seconds)
pub const TARGET_BLOCK_TIME: u64 = 3600;

/// LWMA window: 60 blocks (~60 hours of history)
pub const LWMA_WINDOW: usize = 60;

/// Minimum difficulty
pub const MIN_DIFFICULTY: u64 = 1000;

/// Maximum difficulty adjustment per block (300% = 3x)
pub const MAX_ADJUSTMENT_FACTOR: f64 = 3.0;

/// Minimum difficulty adjustment per block (33% = 1/3)
pub const MIN_ADJUSTMENT_FACTOR: f64 = 0.33;

/// Simple block header for difficulty calculation
#[derive(Debug, Clone)]
pub struct BlockHeader {
    pub height: u64,
    pub timestamp: u64,
    pub difficulty: BigUint,
}

/// Calculate next difficulty using LWMA
pub fn calculate_lwma_difficulty(block_headers: &[BlockHeader]) -> BigUint {
    if block_headers.len() < LWMA_WINDOW + 1 {
        return BigUint::from(MIN_DIFFICULTY);
    }
    
    let start_idx = block_headers.len().saturating_sub(LWMA_WINDOW + 1);
    let window = &block_headers[start_idx..];
    
    let mut weighted_times: u64 = 0;
    let mut sum_difficulties = BigUint::zero();
    
    for i in 1..=LWMA_WINDOW {
        let time_delta = window[i]
            .timestamp
            .saturating_sub(window[i - 1].timestamp)
            .max(1);
        
        let weight = i as u64;
        weighted_times = weighted_times.saturating_add(time_delta.saturating_mul(weight));
        sum_difficulties += &window[i].difficulty;
    }
    
    let n = LWMA_WINDOW as u64;
    let expected_times = TARGET_BLOCK_TIME
        .saturating_mul(n)
        .saturating_mul(n + 1)
        / 2;
    
    let avg_difficulty = sum_difficulties / LWMA_WINDOW;
    
    let new_difficulty = if weighted_times == 0 || expected_times == 0 {
        avg_difficulty
    } else {
        let adjustment = weighted_times as f64 / expected_times as f64;
        let clamped_adjustment = adjustment
            .max(MIN_ADJUSTMENT_FACTOR)
            .min(MAX_ADJUSTMENT_FACTOR);
        
        let adjusted = avg_difficulty.to_f64().unwrap_or(MIN_DIFFICULTY as f64) * clamped_adjustment;
        BigUint::from(adjusted as u64)
    };
    
    new_difficulty.max(BigUint::from(MIN_DIFFICULTY))
}

/// Convert difficulty to target
pub fn difficulty_to_target(difficulty: &BigUint) -> BigUint {
    if difficulty.is_zero() {
        return max_target();
    }
    max_target() / difficulty
}

/// Maximum target (2^256 - 1)
pub fn max_target() -> BigUint {
    (BigUint::one() << 256) - BigUint::one()
}

/// Check if hash meets difficulty
pub fn meets_difficulty(block_hash: &[u8; 32], difficulty: &BigUint) -> bool {
    let hash_as_num = BigUint::from_bytes_be(block_hash);
    let target = difficulty_to_target(difficulty);
    hash_as_num <= target
}

/// Estimate hashrate from difficulty
pub fn estimate_hashrate(difficulty: &BigUint) -> f64 {
    let diff_f64 = difficulty.to_f64().unwrap_or(MIN_DIFFICULTY as f64);
    let hashes_per_block = diff_f64 * (1u64 << 32) as f64;
    hashes_per_block / TARGET_BLOCK_TIME as f64
}

/// Format hashrate for display
pub fn format_hashrate(hashrate: f64) -> String {
    if hashrate >= 1e15 {
        format!("{:.2} PH/s", hashrate / 1e15)
    } else if hashrate >= 1e12 {
        format!("{:.2} TH/s", hashrate / 1e12)
    } else if hashrate >= 1e9 {
        format!("{:.2} GH/s", hashrate / 1e9)
    } else if hashrate >= 1e6 {
        format!("{:.2} MH/s", hashrate / 1e6)
    } else if hashrate >= 1e3 {
        format!("{:.2} KH/s", hashrate / 1e3)
    } else {
        format!("{:.2} H/s", hashrate)
    }
}

/// Detect flash mining attack
pub fn detect_flash_mining(headers: &[BlockHeader]) -> bool {
    if headers.len() < LWMA_WINDOW {
        return false;
    }
    
    let recent = &headers[headers.len().saturating_sub(10)..];
    if recent.len() < 2 {
        return false;
    }
    
    let total_time = recent
        .last()
        .unwrap()
        .timestamp
        .saturating_sub(recent.first().unwrap().timestamp);
    
    let avg_time = total_time as f64 / (recent.len() - 1) as f64;
    avg_time < (TARGET_BLOCK_TIME as f64 / 10.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_headers(count: usize, block_time: u64, difficulty: u64) -> Vec<BlockHeader> {
        let mut headers = Vec::new();
        let mut timestamp = 1_700_000_000u64;
        
        for i in 0..count {
            headers.push(BlockHeader {
                height: i as u64,
                timestamp,
                difficulty: BigUint::from(difficulty),
            });
            timestamp += block_time;
        }
        headers
    }
    
    #[test]
    fn test_lwma_stable_hashrate() {
        let headers = create_test_headers(100, TARGET_BLOCK_TIME, 100_000);
        let new_diff = calculate_lwma_difficulty(&headers);
        let diff_u64 = new_diff.to_u64().unwrap_or(0);
        assert!(diff_u64 >= 90_000 && diff_u64 <= 110_000);
    }
    
    #[test]
    fn test_lwma_hashrate_increase() {
        // Simulate hashrate doubling (blocks come 2x faster)
        // Create enough history with slower blocks first
        let mut headers = create_test_headers(70, TARGET_BLOCK_TIME, 100_000);
        
        // Then add faster blocks (should trigger difficulty increase)
        let last_timestamp = headers.last().unwrap().timestamp;
        for i in 0..30 {
            headers.push(BlockHeader {
                height: (70 + i) as u64,
                timestamp: last_timestamp + (i as u64 * (TARGET_BLOCK_TIME / 2)),
                difficulty: BigUint::from(100_000u64),
            });
        }
        
        let new_diff = calculate_lwma_difficulty(&headers);
        // With blocks coming 2x faster, difficulty should increase
        // (might not double immediately due to weighted average)
        assert!(new_diff > BigUint::from(100_000u64));
    }
    
    #[test]
    fn test_minimum_difficulty() {
        let headers = create_test_headers(100, TARGET_BLOCK_TIME * 100, 1000);
        let new_diff = calculate_lwma_difficulty(&headers);
        assert!(new_diff >= BigUint::from(MIN_DIFFICULTY));
    }
    
    #[test]
    fn test_flash_mining_detection() {
        let normal = create_test_headers(70, TARGET_BLOCK_TIME, 100_000);
        assert!(!detect_flash_mining(&normal));
        
        let flash = create_test_headers(70, 30, 100_000);
        assert!(detect_flash_mining(&flash));
    }
}
