use dashmap::DashMap;
use libp2p::{Multiaddr, PeerId};
use lru::LruCache;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::interval;
use tracing::{debug, error, info, trace, warn};

use super::config::NetworkConfig;
use super::discv5_service::DiscoveredPeer;

/// Peer connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerState {
    /// Peer discovered but not yet connected
    Discovered,
    
    /// Connection attempt in progress
    Dialing,
    
    /// Successfully connected
    Connected,
    
    /// Connection failed
    Failed,
    
    /// Peer disconnected
    Disconnected,
    
    /// Peer banned (misbehavior detected)
    Banned,
}

/// Peer reputation score (-100 to +100)
#[derive(Debug, Clone, Copy)]
pub struct Reputation {
    score: i32,
}

impl Reputation {
    const MIN_SCORE: i32 = -100;
    const MAX_SCORE: i32 = 100;
    const BAN_THRESHOLD: i32 = -50;
    const INITIAL_SCORE: i32 = 0;
    
    pub fn new() -> Self {
        Self {
            score: Self::INITIAL_SCORE,
        }
    }
    
    /// Add reputation points (for good behavior)
    pub fn add(&mut self, points: i32) {
        self.score = (self.score + points).min(Self::MAX_SCORE);
    }
    
    /// Subtract reputation points (for bad behavior)
    pub fn subtract(&mut self, points: i32) {
        self.score = (self.score - points).max(Self::MIN_SCORE);
    }
    
    /// Check if peer should be banned
    pub fn should_ban(&self) -> bool {
        self.score <= Self::BAN_THRESHOLD
    }
    
    /// Get current score
    pub fn score(&self) -> i32 {
        self.score
    }
    
    /// Reset to initial score
    pub fn reset(&mut self) {
        self.score = Self::INITIAL_SCORE;
    }
}

impl Default for Reputation {
    fn default() -> Self {
        Self::new()
    }
}

/// Peer information with lifecycle tracking
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: PeerId,
    pub multiaddrs: Vec<Multiaddr>,
    pub state: PeerState,
    pub reputation: Reputation,
    pub discovered_at: Instant,
    pub last_connected: Option<Instant>,
    pub last_disconnected: Option<Instant>,
    pub connection_attempts: u32,
    pub successful_connections: u32,
    pub failed_connections: u32,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub protocol_version: Option<String>,
    pub agent_version: Option<String>,
}

impl PeerInfo {
    pub fn new(peer_id: PeerId, multiaddrs: Vec<Multiaddr>) -> Self {
        Self {
            peer_id,
            multiaddrs,
            state: PeerState::Discovered,
            reputation: Reputation::new(),
            discovered_at: Instant::now(),
            last_connected: None,
            last_disconnected: None,
            connection_attempts: 0,
            successful_connections: 0,
            failed_connections: 0,
            bytes_sent: 0,
            bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
            protocol_version: None,
            agent_version: None,
        }
    }
    
    /// Mark peer as dialing
    pub fn mark_dialing(&mut self) {
        self.state = PeerState::Dialing;
        self.connection_attempts += 1;
    }
    
    /// Mark peer as connected
    pub fn mark_connected(&mut self) {
        self.state = PeerState::Connected;
        self.last_connected = Some(Instant::now());
        self.successful_connections += 1;
        self.reputation.add(5); // Reward successful connection
    }
    
    /// Mark peer as disconnected
    pub fn mark_disconnected(&mut self) {
        self.state = PeerState::Disconnected;
        self.last_disconnected = Some(Instant::now());
    }
    
    /// Mark peer as failed
    pub fn mark_failed(&mut self) {
        self.state = PeerState::Failed;
        self.failed_connections += 1;
        self.reputation.subtract(2); // Penalize failed connection
    }
    
    /// Mark peer as banned
    pub fn mark_banned(&mut self) {
        self.state = PeerState::Banned;
    }
    
    /// Check if peer should be retried
    pub fn should_retry(&self, max_attempts: u32) -> bool {
        self.state == PeerState::Failed 
            && self.connection_attempts < max_attempts
            && !self.reputation.should_ban()
    }
    
    /// Get time since last connection attempt
    pub fn time_since_last_attempt(&self) -> Option<Duration> {
        self.last_connected.or(self.last_disconnected).map(|t| t.elapsed())
    }
}

/// Peer connection event
#[derive(Debug, Clone)]
pub enum PeerEvent {
    /// Request to dial a peer
    Dial { peer_id: PeerId, address: Multiaddr },
    
    /// Request to disconnect from peer
    Disconnect { peer_id: PeerId },
    
    /// Request to ban peer
    Ban { peer_id: PeerId, reason: String },
    
    /// Peer connected successfully
    Connected { peer_id: PeerId, address: Multiaddr },
    
    /// Peer disconnected
    Disconnected { peer_id: PeerId },
    
    /// Connection attempt failed
    DialFailed { peer_id: PeerId, error: String },
}

/// Peer manager metrics
#[derive(Debug, Clone, Default)]
pub struct PeerMetrics {
    pub total_peers_discovered: u64,
    pub total_connections_attempted: u64,
    pub total_connections_succeeded: u64,
    pub total_connections_failed: u64,
    pub total_disconnections: u64,
    pub total_bans: u64,
    pub current_connected: usize,
    pub current_dialing: usize,
    pub current_banned: usize,
}

/// Production-grade peer connection manager
#[derive(Clone)]
pub struct PeerManager {
    /// All known peers indexed by PeerId
    peers: Arc<DashMap<PeerId, PeerInfo>>,
    
    /// Currently connected peers (fast lookup)
    connected_peers: Arc<RwLock<HashSet<PeerId>>>,
    
    /// Banned peers with unban time
    banned_peers: Arc<DashMap<PeerId, Instant>>,
    
    /// LRU cache for recently seen peers (prevents memory bloat)
    peer_cache: Arc<RwLock<LruCache<PeerId, ()>>>,
    
    /// Metrics
    metrics: Arc<RwLock<PeerMetrics>>,
    
    /// Configuration
    config: NetworkConfig,
    
    /// Event channel for peer actions
    event_tx: mpsc::UnboundedSender<PeerEvent>,
    event_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<PeerEvent>>>>,
    
    /// Shutdown signal
    shutdown_tx: Option<mpsc::UnboundedSender<()>>,
}

impl PeerManager {
    /// Create new peer manager
    pub fn new(config: NetworkConfig) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        
        // LRU cache size = 2x max_peers (keeps recent disconnected peers)
        let cache_size = NonZeroUsize::new(config.max_peers * 2).unwrap();
        
        Self {
            peers: Arc::new(DashMap::new()),
            connected_peers: Arc::new(RwLock::new(HashSet::new())),
            banned_peers: Arc::new(DashMap::new()),
            peer_cache: Arc::new(RwLock::new(LruCache::new(cache_size))),
            metrics: Arc::new(RwLock::new(PeerMetrics::default())),
            config,
            event_tx,
            event_rx: Arc::new(RwLock::new(Some(event_rx))),
            shutdown_tx: None,
        }
    }
    
    /// Add discovered peer from Discv5
    pub fn add_discovered_peer(&self, discovered: DiscoveredPeer) -> Result<(), PeerManagerError> {
        // Create a fake PeerId from the node_id string
        let peer_id_str = discovered.node_id.clone();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        peer_id_str.hash(&mut hasher);
        let hash = hasher.finish();
        let peer_id = PeerId::from_bytes(&hash.to_le_bytes().repeat(4)[..32]).expect("Valid peer id");
        
        // Build multiaddrs from discovered peer
        let mut multiaddrs = Vec::new();
        
        if let Some(tcp_socket) = discovered.tcp_socket {
            if let Ok(addr) = format!("/ip4/{}/tcp/{}", tcp_socket.ip(), tcp_socket.port())
                .parse::<Multiaddr>() {
                multiaddrs.push(addr);
            }
        }
        
        if multiaddrs.is_empty() {
            return Err(PeerManagerError::NoValidAddress);
        }
        
        // Check if already known
        if self.peers.contains_key(&peer_id) {
            trace!("Peer {} already known, skipping", peer_id);
            return Ok(());
        }
        
        // Check if banned
        if self.is_banned(&peer_id) {
            debug!("Peer {} is banned, skipping", peer_id);
            return Ok(());
        }
        
        // Check peer limit
        if self.peers.len() >= self.config.max_peers * 2 {
            // Evict lowest reputation peer
            self.evict_lowest_reputation_peer();
        }
        
        // Create peer info
        let peer_info = PeerInfo::new(peer_id, multiaddrs.clone());
        
        // Store peer
        self.peers.insert(peer_id, peer_info);
        self.peer_cache.write().put(peer_id, ());
        
        // Update metrics
        {
            let mut m = self.metrics.write();
            m.total_peers_discovered += 1;
        }
        
        info!("📥 Added discovered peer: {} ({} addresses)", peer_id_str, multiaddrs.len());
        
        // Auto-dial if we need more peers
        if self.connected_peers.read().len() < self.config.target_peers {
            self.request_dial(peer_id, multiaddrs[0].clone())?;
        }
        
        Ok(())
    }
    
    /// Request dial to peer
    pub fn request_dial(&self, peer_id: PeerId, address: Multiaddr) -> Result<(), PeerManagerError> {
        // Update peer state
        if let Some(mut peer) = self.peers.get_mut(&peer_id) {
            peer.mark_dialing();
        }
        
        // Update metrics
        {
            let mut m = self.metrics.write();
            m.total_connections_attempted += 1;
            m.current_dialing += 1;
        }
        
        // Send dial event
        self.event_tx.send(PeerEvent::Dial { peer_id, address })
            .map_err(|_| PeerManagerError::ChannelClosed)?;
        
        debug!("📞 Requesting dial to peer: {}", peer_id);
        
        Ok(())
    }
    
    /// Handle peer connected event
    pub fn handle_peer_connected(&self, peer_id: PeerId, address: Multiaddr) {
        // Update peer state
        if let Some(mut peer) = self.peers.get_mut(&peer_id) {
            peer.mark_connected();
        }
        
        // Add to connected set
        self.connected_peers.write().insert(peer_id);
        
        // Update metrics
        {
            let mut m = self.metrics.write();
            m.total_connections_succeeded += 1;
            m.current_connected += 1;
            if m.current_dialing > 0 {
                m.current_dialing -= 1;
            }
        }
        
        info!("✅ Peer connected: {} at {}", peer_id, address);
    }
    
    /// Handle peer disconnected event
    pub fn handle_peer_disconnected(&self, peer_id: PeerId) {
        // Update peer state
        if let Some(mut peer) = self.peers.get_mut(&peer_id) {
            peer.mark_disconnected();
        }
        
        // Remove from connected set
        self.connected_peers.write().remove(&peer_id);
        
        // Update metrics
        {
            let mut m = self.metrics.write();
            m.total_disconnections += 1;
            if m.current_connected > 0 {
                m.current_connected -= 1;
            }
        }
        
        info!("❌ Peer disconnected: {}", peer_id);
    }
    
    /// Handle dial failed event
    pub fn handle_dial_failed(&self, peer_id: PeerId, error: String) {
        // Update peer state
        if let Some(mut peer) = self.peers.get_mut(&peer_id) {
            peer.mark_failed();
        }
        
        // Update metrics
        {
            let mut m = self.metrics.write();
            m.total_connections_failed += 1;
            if m.current_dialing > 0 {
                m.current_dialing -= 1;
            }
        }
        
        warn!("⚠️  Dial failed to peer {}: {}", peer_id, error);
    }
    
    /// Ban peer
    pub fn ban_peer(&self, peer_id: PeerId, reason: String) {
        // Mark as banned
        if let Some(mut peer) = self.peers.get_mut(&peer_id) {
            peer.mark_banned();
        }
        
        // Add to banned list with expiry
        let unban_at = Instant::now() + Duration::from_secs(3600); // 1 hour ban
        self.banned_peers.insert(peer_id, unban_at);
        
        // Disconnect if connected
        self.connected_peers.write().remove(&peer_id);
        
        // Update metrics
        {
            let mut m = self.metrics.write();
            m.total_bans += 1;
            m.current_banned += 1;
        }
        
        warn!("🚫 Banned peer {}: {}", peer_id, reason);
        
        // Send disconnect event
        let _ = self.event_tx.send(PeerEvent::Disconnect { peer_id });
    }
    
    /// Check if peer is banned
    pub fn is_banned(&self, peer_id: &PeerId) -> bool {
        if let Some(entry) = self.banned_peers.get(peer_id) {
            if *entry.value() > Instant::now() {
                return true;
            } else {
                // Ban expired, remove
                drop(entry);
                self.banned_peers.remove(peer_id);
                
                let mut m = self.metrics.write();
                if m.current_banned > 0 {
                    m.current_banned -= 1;
                }
            }
        }
        false
    }
    
    /// Update peer reputation (call this for good/bad behavior)
    pub fn update_reputation(&self, peer_id: &PeerId, delta: i32) {
        if let Some(mut peer) = self.peers.get_mut(peer_id) {
            if delta > 0 {
                peer.reputation.add(delta);
            } else {
                peer.reputation.subtract(-delta);
            }
            
            // Auto-ban if reputation too low
            if peer.reputation.should_ban() {
                drop(peer); // Release lock before banning
                self.ban_peer(*peer_id, "Low reputation score".into());
            }
        }
    }
    
    /// Evict lowest reputation peer
    fn evict_lowest_reputation_peer(&self) {
        let mut lowest_score = i32::MAX;
        let mut lowest_peer = None;
        
        for entry in self.peers.iter() {
            let peer = entry.value();
            // Only evict disconnected peers
            if peer.state == PeerState::Disconnected || peer.state == PeerState::Failed {
                if peer.reputation.score() < lowest_score {
                    lowest_score = peer.reputation.score();
                    lowest_peer = Some(*entry.key());
                }
            }
        }
        
        if let Some(peer_id) = lowest_peer {
            self.peers.remove(&peer_id);
            self.peer_cache.write().pop(&peer_id);
            debug!("Evicted low-reputation peer: {}", peer_id);
        }
    }
    
    /// Start heartbeat loop (peer maintenance)
    pub async fn start_heartbeat(&self) {
        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();
        let peers = self.peers.clone();
        let connected_peers = self.connected_peers.clone();
        let banned_peers = self.banned_peers.clone();
        let event_tx = self.event_tx.clone();
        let target_peers = self.config.target_peers;
        
        tokio::spawn(async move {
            let mut heartbeat = interval(Duration::from_secs(30));
            
            loop {
                tokio::select! {
                    _ = heartbeat.tick() => {
                        // 1. Unban expired peers
                        let now = Instant::now();
                        banned_peers.retain(|_, unban_at| *unban_at > now);
                        
                        // 2. Check if we need more connections
                        let current_connected = connected_peers.read().len();
                        
                        if current_connected < target_peers {
                            let needed = target_peers - current_connected;
                            debug!("Need {} more peer connections", needed);
                            
                            // Find disconnected peers to retry
                            let mut retry_candidates: Vec<_> = peers
                                .iter()
                                .filter(|entry| {
                                    let peer = entry.value();
                                    peer.should_retry(5) && !banned_peers.contains_key(entry.key())
                                })
                                .map(|entry| (*entry.key(), entry.value().multiaddrs[0].clone()))
                                .take(needed)
                                .collect();
                            
                            // Dial candidates
                            for (peer_id, addr) in retry_candidates {
                                debug!("🔄 Retrying connection to peer: {}", peer_id);
                                let _ = event_tx.send(PeerEvent::Dial { peer_id, address: addr });
                            }
                        }
                        
                        // 3. Log stats
                        trace!(
                            "Peer stats: connected={}, total={}, banned={}",
                            current_connected,
                            peers.len(),
                            banned_peers.len()
                        );
                    }
                    
                    _ = shutdown_rx.recv() => {
                        info!("🛑 Peer manager heartbeat shutting down");
                        break;
                    }
                }
            }
        });
    }
    
    /// Get event receiver (consume once)
    pub fn take_event_receiver(&self) -> Option<mpsc::UnboundedReceiver<PeerEvent>> {
        self.event_rx.write().take()
    }
    
    /// Get metrics
    pub fn metrics(&self) -> PeerMetrics {
        self.metrics.read().clone()
    }
    
    /// Get connected peer count
    pub fn connected_count(&self) -> usize {
        self.connected_peers.read().len()
    }
    
    /// Get all connected peers
    pub fn connected_peers(&self) -> Vec<PeerId> {
        self.connected_peers.read().iter().copied().collect()
    }
    
    /// Get peer info
    pub fn get_peer(&self, peer_id: &PeerId) -> Option<PeerInfo> {
        self.peers.get(peer_id).map(|p| p.clone())
    }
    
    /// Shutdown manager
    pub async fn shutdown(&self) {
        if let Some(tx) = self.shutdown_tx.clone() {
            let _ = tx.send(());
        }
        info!("🛑 Peer manager shutting down");
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PeerManagerError {
    #[error("Key conversion error: {0}")]
    KeyConversion(String),
    
    #[error("Unsupported key type")]
    UnsupportedKeyType,
    
    #[error("Multiaddr parse error: {0}")]
    MultiaddrParse(String),
    
    #[error("No valid address found for peer")]
    NoValidAddress,
    
    #[error("Event channel closed")]
    ChannelClosed,
}
