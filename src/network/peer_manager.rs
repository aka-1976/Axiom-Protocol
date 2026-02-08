use libp2p::PeerId;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: PeerId,
    pub connected_at: Instant,
    pub last_seen: Instant,
    pub reputation: i32,
    pub messages_received: u64,
    pub messages_sent: u64,
}

impl PeerInfo {
    pub fn new(peer_id: PeerId) -> Self {
        let now = Instant::now();
        Self {
            peer_id,
            connected_at: now,
            last_seen: now,
            reputation: 100,
            messages_received: 0,
            messages_sent: 0,
        }
    }
    
    pub fn update_last_seen(&mut self) {
        self.last_seen = Instant::now();
    }
    
    pub fn increment_reputation(&mut self, amount: i32) {
        self.reputation = (self.reputation + amount).clamp(0, 200);
    }
    
    pub fn decrement_reputation(&mut self, amount: i32) {
        self.reputation = (self.reputation - amount).clamp(0, 200);
    }
    
    pub fn is_healthy(&self) -> bool {
        self.reputation > 50 && self.last_seen.elapsed() < Duration::from_secs(300)
    }
    
    pub fn connection_duration(&self) -> Duration {
        self.connected_at.elapsed()
    }
}

pub struct PeerManager {
    peers: HashMap<PeerId, PeerInfo>,
    max_peers: usize,
    banned_peers: HashMap<PeerId, Instant>,
    ban_duration: Duration,
}

impl PeerManager {
    pub fn new(max_peers: usize) -> Self {
        Self {
            peers: HashMap::new(),
            max_peers,
            banned_peers: HashMap::new(),
            ban_duration: Duration::from_secs(3600),
        }
    }
    
    pub fn add_peer(&mut self, peer_id: PeerId) -> bool {
        if self.is_banned(&peer_id) {
            return false;
        }
        
        if self.peers.len() >= self.max_peers {
            return false;
        }
        
        if !self.peers.contains_key(&peer_id) {
            self.peers.insert(peer_id, PeerInfo::new(peer_id));
            true
        } else {
            false
        }
    }
    
    pub fn remove_peer(&mut self, peer_id: &PeerId) -> Option<PeerInfo> {
        self.peers.remove(peer_id)
    }
    
    pub fn get_peer(&self, peer_id: &PeerId) -> Option<&PeerInfo> {
        self.peers.get(peer_id)
    }
    
    pub fn get_peer_mut(&mut self, peer_id: &PeerId) -> Option<&mut PeerInfo> {
        self.peers.get_mut(peer_id)
    }
    
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }
    
    pub fn healthy_peer_count(&self) -> usize {
        self.peers.values().filter(|p| p.is_healthy()).count()
    }
    
    pub fn all_peers(&self) -> Vec<PeerId> {
        self.peers.keys().copied().collect()
    }
    
    pub fn ban_peer(&mut self, peer_id: PeerId) {
        self.banned_peers.insert(peer_id, Instant::now());
        self.remove_peer(&peer_id);
    }
    
    pub fn is_banned(&self, peer_id: &PeerId) -> bool {
        if let Some(banned_at) = self.banned_peers.get(peer_id) {
            banned_at.elapsed() < self.ban_duration
        } else {
            false
        }
    }
    
    pub fn update_peer_activity(&mut self, peer_id: &PeerId) {
        if let Some(peer) = self.get_peer_mut(peer_id) {
            peer.update_last_seen();
        }
    }
    
    pub fn record_message_sent(&mut self, peer_id: &PeerId) {
        if let Some(peer) = self.get_peer_mut(peer_id) {
            peer.messages_sent += 1;
        }
    }
    
    pub fn record_message_received(&mut self, peer_id: &PeerId) {
        if let Some(peer) = self.get_peer_mut(peer_id) {
            peer.messages_received += 1;
        }
    }
}
