// Mobile mining module for Axiom Protocol
// Enables privacy-preserving mobile mining with 1 AXM rewards

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Mobile miner instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileMiner {
    /// Miner ID (unique per device)
    pub miner_id: [u8; 32],
    /// Current mining intensity (0-100)
    pub intensity: u8,
    /// Mining difficulty target
    pub difficulty: u64,
    /// Total blocks mined
    pub blocks_mined: u64,
    /// Total rewards earned (in satoshis)
    pub rewards_earned: u64,
    /// Last block timestamp
    pub last_block_time: u64,
    /// Battery level (0-100) if mobile
    pub battery_level: Option<u8>,
    /// Is currently mining
    pub is_mining: bool,
}

impl MobileMiner {
    /// Create new mobile miner
    pub fn new(miner_id: [u8; 32], intensity: u8) -> Self {
        Self {
            miner_id,
            intensity: intensity.min(100),
            difficulty: 1_000_000,
            blocks_mined: 0,
            rewards_earned: 0,
            last_block_time: 0,
            battery_level: None,
            is_mining: false,
        }
    }

    /// Start mining
    pub fn start(&mut self) {
        self.is_mining = true;
    }

    /// Stop mining
    pub fn stop(&mut self) {
        self.is_mining = false;
    }

    /// Update battery level
    pub fn set_battery_level(&mut self, level: u8) {
        self.battery_level = Some(level.min(100));
        
        // Auto-throttle if low battery
        if level < 20 {
            self.intensity = (self.intensity / 2).max(10);
        }
    }

    /// Adjust intensity based on device performance
    pub fn adjust_intensity(&mut self, new_intensity: u8) {
        self.intensity = new_intensity.min(100);
    }

    /// Get estimated power consumption (watts)
    pub fn estimate_power(&self) -> f32 {
        let base_power = 2.0; // 2W base (idle)
        let cpu_power = (self.intensity as f32 / 100.0) * 3.0; // Up to 3W at 100%
        base_power + cpu_power
    }

    /// Get estimated hash rate (hashes/second)
    pub fn estimate_hashrate(&self) -> u64 {
        // Scales with intensity and difficulty
        let base_rate = 100_000; // 100k H/s at 100% intensity on reference device
        (base_rate as u64 * self.intensity as u64) / 100
    }

    /// Record mined block
    pub fn record_block(&mut self, _block_height: u64) {
        self.blocks_mined += 1;
        self.rewards_earned += 1_00000000; // 1 AXM = 100,000,000 satoshis
        self.last_block_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Get mining statistics
    pub fn get_stats(&self) -> MinerStats {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let uptime = if self.last_block_time > 0 {
            now - self.last_block_time
        } else {
            0
        };

        MinerStats {
            blocks_mined: self.blocks_mined,
            rewards_earned: self.rewards_earned,
            current_intensity: self.intensity,
            estimated_power_watts: self.estimate_power(),
            estimated_hashrate: self.estimate_hashrate(),
            battery_level: self.battery_level,
            uptime_seconds: uptime,
            is_mining: self.is_mining,
        }
    }
}

/// Mobile block structure (lightweight)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileBlock {
    /// Block height
    pub height: u64,
    /// Previous block hash
    pub prev_hash: [u8; 32],
    /// Miner ID
    pub miner_id: [u8; 32],
    /// Block hash
    pub hash: [u8; 32],
    /// Timestamp
    pub timestamp: u64,
    /// Number of transactions
    pub tx_count: u16,
    /// Difficulty
    pub difficulty: u64,
}

impl MobileBlock {
    /// Create new mobile block
    pub fn new(
        height: u64,
        prev_hash: [u8; 32],
        miner_id: [u8; 32],
        tx_count: u16,
        difficulty: u64,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // In production, calculate actual hash
        let hash = Self::calculate_hash(height, prev_hash, miner_id, timestamp);

        Self {
            height,
            prev_hash,
            miner_id,
            hash,
            timestamp,
            tx_count,
            difficulty,
        }
    }

    fn calculate_hash(height: u64, prev: [u8; 32], miner: [u8; 32], time: u64) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(height.to_le_bytes());
        hasher.update(prev);
        hasher.update(miner);
        hasher.update(time.to_le_bytes());
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result[..32]);
        hash
    }

    /// Verify block validity
    pub fn is_valid(&self) -> bool {
        // Basic validation
        self.tx_count > 0 && self.tx_count < 1000
    }
}

/// Mining statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerStats {
    pub blocks_mined: u64,
    pub rewards_earned: u64,
    pub current_intensity: u8,
    pub estimated_power_watts: f32,
    pub estimated_hashrate: u64,
    pub battery_level: Option<u8>,
    pub uptime_seconds: u64,
    pub is_mining: bool,
}

impl MinerStats {
    /// Get efficiency (blocks per watt)
    pub fn efficiency(&self) -> f32 {
        if self.estimated_power_watts > 0.0 {
            self.blocks_mined as f32 / self.estimated_power_watts
        } else {
            0.0
        }
    }

    /// Estimate monthly earnings (at current rate)
    pub fn estimate_monthly_earnings(&self) -> u64 {
        if self.uptime_seconds > 0 {
            let blocks_per_second = self.blocks_mined as f64 / self.uptime_seconds as f64;
            let blocks_per_month = blocks_per_second * 30.0 * 24.0 * 3600.0;
            (blocks_per_month as u64) * 1_00000000 // 1 AXM per block
        } else {
            0
        }
    }

    /// Print formatted stats
    pub fn print(&self) {
        println!("╔═══════════════════════════════════╗");
        println!("║    MOBILE MINER STATISTICS        ║");
        println!("╚═══════════════════════════════════╝");
        println!("Blocks Mined:        {}", self.blocks_mined);
        println!("Rewards:             {:.8} AXM", self.rewards_earned as f64 / 100_000_000.0);
        println!("Intensity:           {}%", self.current_intensity);
        println!("Power Draw:          {:.2}W", self.estimated_power_watts);
        println!("Hashrate:            {:.2} kH/s", self.estimated_hashrate as f64 / 1000.0);
        
        if let Some(battery) = self.battery_level {
            println!("Battery:             {}%", battery);
        }
        
        println!("Uptime:              {}s", self.uptime_seconds);
        println!("Mining:              {}", if self.is_mining { "Yes" } else { "No" });
        println!("Efficiency:          {:.4} blocks/watt", self.efficiency());
        println!("Est. Monthly:        {:.8} AXM", self.estimate_monthly_earnings() as f64 / 100_000_000.0);
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mobile_miner_creation() {
        let miner = MobileMiner::new([1u8; 32], 50);
        assert_eq!(miner.intensity, 50);
        assert_eq!(miner.blocks_mined, 0);
    }

    #[test]
    fn test_miner_intensity_cap() {
        let miner = MobileMiner::new([1u8; 32], 150); // Over 100
        assert_eq!(miner.intensity, 100); // Should be capped at 100
    }

    #[test]
    fn test_power_estimation() {
        let miner = MobileMiner::new([1u8; 32], 100);
        let power = miner.estimate_power();
        assert!(power >= 2.0 && power <= 5.0); // Should be between 2W and 5W
    }

    #[test]
    fn test_block_recording() {
        let mut miner = MobileMiner::new([1u8; 32], 50);
        miner.record_block(100);
        assert_eq!(miner.blocks_mined, 1);
        assert_eq!(miner.rewards_earned, 1_00000000); // 1 AXM
    }

    #[test]
    fn test_mobile_block_creation() {
        let block = MobileBlock::new(1, [0u8; 32], [1u8; 32], 50, 1_000_000);
        assert_eq!(block.height, 1);
        assert!(block.is_valid());
    }
}
