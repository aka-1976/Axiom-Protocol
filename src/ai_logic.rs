use crate::block::Block;
use std::collections::HashMap;

pub struct AIGuardian {
    peer_trust_scores: HashMap<String, f64>,
}

impl AIGuardian {
    pub fn new() -> Self {
        Self { peer_trust_scores: HashMap::new() }
    }

    /// AI Performance Check: Analyze incoming block for "selfish mining" patterns
    pub fn analyze_block_quality(&mut self, peer_id: &str, block: &Block, local_height: u64) -> bool {
        let score = self.peer_trust_scores.entry(peer_id.to_string()).or_insert(1.0);

        // Pattern: If a peer sends a block that is significantly behind or way ahead
        let height_diff = (block.slot as i64 - local_height as i64).abs();
        
        if height_diff > 5 {
            *score -= 0.1; // Penalize "Anomaly"
        } else {
            *score += 0.05; // Reward "Consistency"
        }

        // Only accept if Trust Score > 0.5
        *score > 0.5
    }

    pub fn get_trust_report(&self) -> String {
        format!("AI Active: Monitoring {} Peers", self.peer_trust_scores.len())
    }
}
