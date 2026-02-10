use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Instant;
use std::collections::VecDeque;

/// Node metrics collector — tracks operational telemetry for production
/// monitoring dashboards and alerting.
#[derive(Clone, Debug)]
pub struct NodeMetrics {
    pub uptime_secs: u64,
    pub peer_count: usize,
    pub blocks_received: u64,
    pub transactions_received: u64,
    pub is_syncing: bool,
    /// Average block processing latency in microseconds (last 100 blocks)
    pub avg_block_latency_us: u64,
    /// Average transaction validation latency in microseconds (last 1000 txs)
    pub avg_tx_latency_us: u64,
    /// Transactions per second (rolling 60-second window)
    pub tx_throughput: f64,
    /// Current chain height
    pub chain_height: u64,
    /// Total mining reward earned by this node (in smallest units)
    pub total_rewards_earned: u64,
    /// Memory usage estimate in bytes (RSS from /proc if available)
    pub memory_usage_bytes: u64,
    /// Number of mempool transactions pending
    pub mempool_size: usize,
    /// Blocks mined by this node
    pub blocks_mined: u64,
}

impl Default for NodeMetrics {
    fn default() -> Self {
        Self {
            uptime_secs: 0,
            peer_count: 0,
            blocks_received: 0,
            transactions_received: 0,
            is_syncing: false,
            avg_block_latency_us: 0,
            avg_tx_latency_us: 0,
            tx_throughput: 0.0,
            chain_height: 0,
            total_rewards_earned: 0,
            memory_usage_bytes: 0,
            mempool_size: 0,
            blocks_mined: 0,
        }
    }
}

/// Metrics registry — thread-safe metrics collection for production monitoring
#[derive(Clone)]
pub struct MetricsCollector {
    metrics: Arc<RwLock<NodeMetrics>>,
    start_time: Instant,
    /// Rolling window of block processing latencies (microseconds)
    block_latencies: Arc<RwLock<VecDeque<u64>>>,
    /// Rolling window of transaction validation latencies (microseconds)
    tx_latencies: Arc<RwLock<VecDeque<u64>>>,
    /// Timestamps of recent transactions for throughput calculation
    tx_timestamps: Arc<RwLock<VecDeque<Instant>>>,
}

const MAX_BLOCK_LATENCY_SAMPLES: usize = 100;
const MAX_TX_LATENCY_SAMPLES: usize = 1000;
const THROUGHPUT_WINDOW_SECS: u64 = 60;

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(NodeMetrics::default())),
            start_time: Instant::now(),
            block_latencies: Arc::new(RwLock::new(VecDeque::with_capacity(MAX_BLOCK_LATENCY_SAMPLES))),
            tx_latencies: Arc::new(RwLock::new(VecDeque::with_capacity(MAX_TX_LATENCY_SAMPLES))),
            tx_timestamps: Arc::new(RwLock::new(VecDeque::new())),
        }
    }
    
    pub fn update_peer_count(&self, count: usize) {
        self.metrics.write().peer_count = count;
    }
    
    pub fn increment_blocks(&self, count: u64) {
        self.metrics.write().blocks_received += count;
    }
    
    pub fn increment_transactions(&self, count: u64) {
        let mut m = self.metrics.write();
        m.transactions_received += count;
        drop(m);

        // Record transaction timestamps for throughput calculation
        let now = Instant::now();
        let mut ts = self.tx_timestamps.write();
        for _ in 0..count {
            ts.push_back(now);
        }
        // Prune old timestamps beyond the throughput window
        let cutoff = now - std::time::Duration::from_secs(THROUGHPUT_WINDOW_SECS);
        while ts.front().map_or(false, |&t| t < cutoff) {
            ts.pop_front();
        }
    }
    
    pub fn set_syncing(&self, syncing: bool) {
        self.metrics.write().is_syncing = syncing;
    }

    /// Record a block processing latency measurement
    pub fn record_block_latency(&self, latency_us: u64) {
        let mut lats = self.block_latencies.write();
        if lats.len() >= MAX_BLOCK_LATENCY_SAMPLES {
            lats.pop_front();
        }
        lats.push_back(latency_us);
    }

    /// Record a transaction validation latency measurement
    pub fn record_tx_latency(&self, latency_us: u64) {
        let mut lats = self.tx_latencies.write();
        if lats.len() >= MAX_TX_LATENCY_SAMPLES {
            lats.pop_front();
        }
        lats.push_back(latency_us);
    }

    /// Update chain height
    pub fn set_chain_height(&self, height: u64) {
        self.metrics.write().chain_height = height;
    }

    /// Record a mined block's reward
    pub fn record_mined_block(&self, reward: u64) {
        let mut m = self.metrics.write();
        m.blocks_mined += 1;
        m.total_rewards_earned += reward;
    }

    /// Update mempool size
    pub fn set_mempool_size(&self, size: usize) {
        self.metrics.write().mempool_size = size;
    }
    
    /// Take a consistent snapshot of all metrics
    pub fn snapshot(&self) -> NodeMetrics {
        let mut metrics = self.metrics.read().clone();
        metrics.uptime_secs = self.start_time.elapsed().as_secs();

        // Calculate average block latency
        let block_lats = self.block_latencies.read();
        if !block_lats.is_empty() {
            metrics.avg_block_latency_us = block_lats.iter().sum::<u64>() / block_lats.len() as u64;
        }

        // Calculate average transaction latency
        let tx_lats = self.tx_latencies.read();
        if !tx_lats.is_empty() {
            metrics.avg_tx_latency_us = tx_lats.iter().sum::<u64>() / tx_lats.len() as u64;
        }

        // Calculate transaction throughput (TPS over rolling window)
        let now = Instant::now();
        let cutoff = now - std::time::Duration::from_secs(THROUGHPUT_WINDOW_SECS);
        let ts = self.tx_timestamps.read();
        let recent_count = ts.iter().filter(|&&t| t >= cutoff).count();
        metrics.tx_throughput = recent_count as f64 / THROUGHPUT_WINDOW_SECS as f64;

        // Read memory usage from /proc/self/status if available
        metrics.memory_usage_bytes = read_rss_bytes();

        metrics
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Read the RSS (Resident Set Size) from /proc/self/status on Linux.
/// Returns 0 on non-Linux or if the file cannot be read.
fn read_rss_bytes() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("VmRSS:"))
                .and_then(|l| {
                    l.split_whitespace()
                        .nth(1)
                        .and_then(|v| v.parse::<u64>().ok())
                })
                .map(|kb| kb * 1024) // Convert KB to bytes
        })
        .unwrap_or(0)
}
