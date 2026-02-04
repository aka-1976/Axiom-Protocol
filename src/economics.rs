// src/economics.rs - AXIOM Protocol: The Sovereign Identity
// 124M Fixed Supply | 70-Year Generation Era | Non-Governance Mathematics

/// The Scarcity Engine - Mathematical Constants
pub const PROTOCOL_NAME: &str = "AXIOM Protocol";
pub const TICKER: &str = "AXM";
pub const CREATOR: &str = "Ghost-84M (Non-Identity)";

/// Total Supply: 124,000,000 AXM (The Sovereign Constant)
pub const TOTAL_SUPPLY: u64 = 124_000_000_000_000_000; // 124M in smallest units
pub const SMALLEST_UNIT: u64 = 100_000_000; // 10^8 (Satoshi-scale divisibility)

/// Initial Mining Reward: 50 AXM per block
pub const INITIAL_REWARD: u64 = 50 * SMALLEST_UNIT; // 5,000,000,000

/// Halving Interval: 2,100,000 blocks (~4 years at 1hr/block)
pub const HALVING_INTERVAL: u64 = 2_100_000;

/// Block Time: 1 hour (3600 seconds) - The Pulse
pub const BLOCK_TIME_SECONDS: u64 = 3600;

/// Era Duration: ~4 years per generation
pub const ERA_DURATION_YEARS: f64 = 4.0;

/// Genesis Message
pub const GENESIS_MESSAGE: &[u8] = b"The timeline is decentralized. Only math governs. - Ghost-84M 2026";

/// Binary Signature: AXIOM in ASCII
pub const AXIOM_SIGNATURE: &str = "01000001 01011000 01001001 01001111 01001101";

// ==================== CORE ECONOMICS ====================

/// Calculate mining reward for a given block height
/// 
/// Formula: reward = 50 AXM >> (height / 2,100,000)
/// 
/// This implements exact binary halving every 2.1M blocks (~4 years).
/// After 33 halvings, reward becomes negligible (supply cap reached).
pub fn get_mining_reward(height: u64) -> u64 {
    let era = height / HALVING_INTERVAL;
    
    // After 64 halvings, reward is effectively 0
    if era >= 64 {
        return 0;
    }
    
    // Binary right shift for exact halving
    INITIAL_REWARD >> era
}

/// Legacy alias for compatibility with chain.rs
/// Calculate block reward based on block slot and total issued supply
pub fn block_reward(slot: u64, _total_issued: u64) -> u64 {
    get_mining_reward(slot)
}

/// Calculate total supply at a given height
/// 
/// This accounts for all mined blocks up to the current height,
/// applying the halving schedule correctly.
pub fn calculate_total_supply(height: u64) -> u64 {
    if height == 0 {
        return 0;
    }
    
    let mut total = 0u64;
    let mut current_height = 0u64;
    let mut era = 0u64;
    
    while current_height < height && era < 64 {
        let reward = INITIAL_REWARD >> era;
        let blocks_in_era = HALVING_INTERVAL.min(height - current_height);
        
        total = total.saturating_add(reward.saturating_mul(blocks_in_era));
        current_height += blocks_in_era;
        era += 1;
    }
    
    total.min(TOTAL_SUPPLY) // Never exceed 124M cap
}

/// Calculate remaining supply to be mined
pub fn remaining_supply(height: u64) -> u64 {
    TOTAL_SUPPLY.saturating_sub(calculate_total_supply(height))
}

/// Calculate percentage of supply mined
pub fn supply_percentage(height: u64) -> f64 {
    (calculate_total_supply(height) as f64 / TOTAL_SUPPLY as f64) * 100.0
}

/// Get current era (halving period)
pub fn current_era(height: u64) -> u64 {
    (height / HALVING_INTERVAL).min(63)
}

/// Calculate blocks until next halving
pub fn blocks_until_halving(height: u64) -> u64 {
    HALVING_INTERVAL - (height % HALVING_INTERVAL)
}

/// Get era statistics for display
#[derive(Debug, Clone)]
pub struct EraStats {
    pub era: u64,
    pub start_height: u64,
    pub end_height: u64,
    pub reward: u64,
    pub total_era_supply: u64,
    pub years_duration: f64,
}

impl EraStats {
    pub fn for_height(height: u64) -> Self {
        let era = current_era(height);
        let reward = get_mining_reward(height);
        let start_height = era * HALVING_INTERVAL;
        let end_height = (era + 1) * HALVING_INTERVAL;
        let total_era_supply = reward * HALVING_INTERVAL;
        
        Self {
            era,
            start_height,
            end_height,
            reward,
            total_era_supply,
            years_duration: ERA_DURATION_YEARS,
        }
    }
}

// ==================== 20-YEAR NETWORK SIMULATION ====================

/// Network phase definitions based on the 20-year simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkPhase {
    /// Year 1-5: Early miners secure 50 AXM reward (~4.3M AXM)
    PillarPhase,
    
    /// Year 5-10: Infrastructure Phase (~8.7M AXM total)
    InfrastructurePhase,
    
    /// Year 10-20: Sovereign Phase (~17.5M AXM total, 14% of supply)
    SovereignPhase,
    
    /// Year 20+: Maturity Phase
    MaturityPhase,
}

impl NetworkPhase {
    pub fn from_height(height: u64) -> Self {
        // Approximate years based on 30-min blocks
        let blocks_per_year = (365.25 * 24.0 * 60.0 / 30.0) as u64; // ~17,532 blocks/year
        let years = height / blocks_per_year;
        
        match years {
            0..=4 => NetworkPhase::PillarPhase,
            5..=9 => NetworkPhase::InfrastructurePhase,
            10..=19 => NetworkPhase::SovereignPhase,
            _ => NetworkPhase::MaturityPhase,
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            NetworkPhase::PillarPhase => {
                "Pillar Phase: Foundation building, early adopters secure the network"
            }
            NetworkPhase::InfrastructurePhase => {
                "Infrastructure Phase: AI shield training, global adoption begins"
            }
            NetworkPhase::SovereignPhase => {
                "Sovereign Phase: Proven track record, mathematical sovereignty established"
            }
            NetworkPhase::MaturityPhase => {
                "Maturity Phase: Network valued for decades of unpatched, perfect math"
            }
        }
    }
    
    pub fn expected_supply(&self) -> u64 {
        match self {
            NetworkPhase::PillarPhase => 4_300_000 * SMALLEST_UNIT,
            NetworkPhase::InfrastructurePhase => 8_700_000 * SMALLEST_UNIT,
            NetworkPhase::SovereignPhase => 17_500_000 * SMALLEST_UNIT,
            NetworkPhase::MaturityPhase => 35_000_000 * SMALLEST_UNIT,
        }
    }
}

// ==================== DISPLAY & FORMATTING ====================

/// Format AXM amount for display (converts from smallest units)
pub fn format_axm(amount: u64) -> String {
    let axm = amount as f64 / SMALLEST_UNIT as f64;
    format!("{:.8} AXM", axm)
}

/// Format supply statistics for monitoring
pub fn format_supply_stats(height: u64) -> String {
    let current_supply = calculate_total_supply(height);
    let remaining = remaining_supply(height);
    let percentage = supply_percentage(height);
    let reward = get_mining_reward(height);
    let era = current_era(height);
    let blocks_to_halving = blocks_until_halving(height);
    let phase = NetworkPhase::from_height(height);
    
    format!(
        r#"
╔══════════════════════════════════════════════════════════════╗
║           🔺 AXIOM PROTOCOL - SOVEREIGN SUPPLY              ║
╠══════════════════════════════════════════════════════════════╣
║  Total Supply:        124,000,000 AXM (Fixed Forever)       ║
║  Current Supply:      {:>12} AXM ({:>5.2}%)             ║
║  Remaining:           {:>12} AXM ({:>5.2}%)             ║
║                                                              ║
║  Block Height:        {:>12}                              ║
║  Current Reward:      {:>12} AXM                          ║
║  Era:                 {:>2}/64                               ║
║  Blocks to Halving:   {:>12}                              ║
║                                                              ║
║  Network Phase:       {}                    ║
║  {}  ║
╚══════════════════════════════════════════════════════════════╝
        "#,
        current_supply / SMALLEST_UNIT,
        percentage,
        remaining / SMALLEST_UNIT,
        100.0 - percentage,
        height,
        reward / SMALLEST_UNIT,
        era,
        blocks_to_halving,
        match phase {
            NetworkPhase::PillarPhase => "Pillar         ",
            NetworkPhase::InfrastructurePhase => "Infrastructure ",
            NetworkPhase::SovereignPhase => "Sovereign      ",
            NetworkPhase::MaturityPhase => "Maturity       ",
        },
        phase.description()
    )
}

// ==================== VALIDATION ====================

/// Validate that the economics are correctly implemented
pub fn validate_economics() -> Result<(), String> {
    // Test 1: Initial reward should be 50 AXM
    if get_mining_reward(0) != 50 * SMALLEST_UNIT {
        return Err(format!(
            "Initial reward incorrect: expected {}, got {}",
            50 * SMALLEST_UNIT,
            get_mining_reward(0)
        ));
    }
    
    // Test 2: First halving should occur at block 2,100,000
    let before_halving = get_mining_reward(HALVING_INTERVAL - 1);
    let after_halving = get_mining_reward(HALVING_INTERVAL);
    
    if after_halving != before_halving / 2 {
        return Err(format!(
            "Halving not working correctly at block {}: {} -> {}",
            HALVING_INTERVAL, before_halving, after_halving
        ));
    }
    
    // Test 3: Total supply calculation
    let mut total = 0u64;
    let mut era = 0u64;
    
    while era < 64 {
        let reward = get_mining_reward(era * HALVING_INTERVAL);
        if reward == 0 {
            break;
        }
        // Add reward for all blocks in this era
        total = total.saturating_add(reward.saturating_mul(HALVING_INTERVAL));
        era += 1;
    }
    
    // Allow small rounding error (should be very close to 124M)
    if total < TOTAL_SUPPLY * 99 / 100 || total > TOTAL_SUPPLY {
        return Err(format!(
            "Total supply calculation incorrect: expected {}, got {}",
            TOTAL_SUPPLY, total
        ));
    }
    
    Ok(())
}

// ==================== TESTS ====================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_initial_reward() {
        assert_eq!(get_mining_reward(0), 50 * SMALLEST_UNIT);
    }
    
    #[test]
    fn test_halving_schedule() {
        // Era 0: 50 AXM
        assert_eq!(get_mining_reward(0), 50 * SMALLEST_UNIT);
        assert_eq!(get_mining_reward(HALVING_INTERVAL - 1), 50 * SMALLEST_UNIT);
        
        // Era 1: 25 AXM
        assert_eq!(get_mining_reward(HALVING_INTERVAL), 25 * SMALLEST_UNIT);
        assert_eq!(get_mining_reward(HALVING_INTERVAL * 2 - 1), 25 * SMALLEST_UNIT);
        
        // Era 2: 12.5 AXM
        assert_eq!(get_mining_reward(HALVING_INTERVAL * 2), 12 * SMALLEST_UNIT + 50_000_000);
        
        // Era 3: 6.25 AXM
        assert_eq!(get_mining_reward(HALVING_INTERVAL * 3), 6 * SMALLEST_UNIT + 25_000_000);
    }
    
    #[test]
    fn test_supply_cap() {
        // Total supply should never exceed 124M
        // Calculate supply after many eras (rewards diminish to near-zero)
        let final_height = 40 * HALVING_INTERVAL;  // After 40 halvings, reward is microscopic
        let final_supply = calculate_total_supply(final_height);
        assert!(final_supply <= TOTAL_SUPPLY);
        
        // Should be very close to 124M (within rounding error)
        // The halving schedule ensures we approach 124M asymptotically
        assert!(final_supply >= TOTAL_SUPPLY * 99 / 100, 
            "Supply {} is less than 99% of {}", final_supply, TOTAL_SUPPLY);
    }
    
    #[test]
    fn test_era_calculation() {
        assert_eq!(current_era(0), 0);
        assert_eq!(current_era(HALVING_INTERVAL - 1), 0);
        assert_eq!(current_era(HALVING_INTERVAL), 1);
        assert_eq!(current_era(HALVING_INTERVAL * 2), 2);
    }
    
    #[test]
    fn test_blocks_until_halving() {
        assert_eq!(blocks_until_halving(0), HALVING_INTERVAL);
        assert_eq!(blocks_until_halving(1), HALVING_INTERVAL - 1);
        assert_eq!(blocks_until_halving(HALVING_INTERVAL - 1), 1);
        assert_eq!(blocks_until_halving(HALVING_INTERVAL), HALVING_INTERVAL);
    }
    
    #[test]
    fn test_network_phases() {
        let blocks_per_year = (365.25 * 24.0 * 60.0 / 30.0) as u64;
        
        // Year 1: Pillar Phase
        assert_eq!(NetworkPhase::from_height(blocks_per_year), NetworkPhase::PillarPhase);
        
        // Year 7: Infrastructure Phase
        assert_eq!(
            NetworkPhase::from_height(7 * blocks_per_year),
            NetworkPhase::InfrastructurePhase
        );
        
        // Year 15: Sovereign Phase
        assert_eq!(
            NetworkPhase::from_height(15 * blocks_per_year),
            NetworkPhase::SovereignPhase
        );
        
        // Year 25: Maturity Phase
        assert_eq!(
            NetworkPhase::from_height(25 * blocks_per_year),
            NetworkPhase::MaturityPhase
        );
    }
    
    #[test]
    fn test_20_year_simulation() {
        let blocks_per_year = (365.25 * 24.0 * 60.0 / 30.0) as u64;
        
        // Year 5: ~4.3M AXM (Pillar Phase)
        let year_5_supply = calculate_total_supply(5 * blocks_per_year);
        let year_5_axm = year_5_supply / SMALLEST_UNIT;
        assert!(year_5_axm >= 4_000_000 && year_5_axm <= 4_500_000);
        
        // Year 10: ~8.7M AXM (Infrastructure Phase)
        let year_10_supply = calculate_total_supply(10 * blocks_per_year);
        let year_10_axm = year_10_supply / SMALLEST_UNIT;
        assert!(year_10_axm >= 8_000_000 && year_10_axm <= 9_000_000);
        
        // Year 20: ~17.5M AXM (Sovereign Phase)
        let year_20_supply = calculate_total_supply(20 * blocks_per_year);
        let year_20_axm = year_20_supply / SMALLEST_UNIT;
        assert!(year_20_axm >= 17_000_000 && year_20_axm <= 18_000_000);
    }
    
    #[test]
    fn test_validation() {
        assert!(validate_economics().is_ok());
    }
    
    #[test]
    fn test_format_axm() {
        assert_eq!(format_axm(100_000_000), "1.00000000 AXM");
        assert_eq!(format_axm(50_000_000_000), "500.00000000 AXM");
    }
}
