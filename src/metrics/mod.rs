use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Instant;

/// Node metrics collector
#[derive(Clone, Debug, Default)]
pub struct NodeMetrics {
    pub uptime_secs: u64,
    pub peer_count: usize,
    pub blocks_received: u64,
    pub transactions_received: u64,
    pub is_syncing: bool,
}

/// Metrics registry
#[derive(Clone)]
pub struct MetricsCollector {
    metrics: Arc<RwLock<NodeMetrics>>,
    start_time: Instant,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(NodeMetrics::default())),
            start_time: Instant::now(),
        }
    }
    
    pub fn update_peer_count(&self, count: usize) {
        self.metrics.write().peer_count = count;
    }
    
    pub fn increment_blocks(&self, count: u64) {
        self.metrics.write().blocks_received += count;
    }
    
    pub fn increment_transactions(&self, count: u64) {
        self.metrics.write().transactions_received += count;
    }
    
    pub fn set_syncing(&self, syncing: bool) {
        self.metrics.write().is_syncing = syncing;
    }
    
    pub fn snapshot(&self) -> NodeMetrics {
        let mut metrics = self.metrics.read().clone();
        metrics.uptime_secs = self.start_time.elapsed().as_secs();
        metrics
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
