/// Network Configuration Module
/// Handles peer discovery, bootstrap peer management, and consensus synchronization

use serde::{Deserialize, Serialize};
use log;

/// Network configuration for all node types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// TCP port to listen on (default: 7000)
    pub listen_port: u16,
    
    /// Bootstrap peer addresses (multiaddr format)
    pub bootstrap_peers: Vec<String>,
    
    /// Maximum peers to connect to
    pub max_peers: u16,
    
    /// Minimum peers required for consensus
    pub min_peers: u16,
    
    /// Peer reconnection timeout (seconds)
    pub reconnect_timeout: u64,
    
    /// Peer discovery interval (seconds)
    pub discovery_interval: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_port: 7000,
            bootstrap_peers: vec![
                // Will be populated from environment or config file
            ],
            max_peers: 50,
            min_peers: 4,
            reconnect_timeout: 30,
            discovery_interval: 60,
        }
    }
}

impl NetworkConfig {
    /// Create config for genesis miner (5 nodes)
    pub fn for_genesis_miner(node_id: u8) -> Self {
        let mut config = Self::default();
        
        // Genesis miners use the same public bootstrap peers as the main network.
        // Operators must update these addresses to match their actual infrastructure
        // or set the AXIOM_BOOTSTRAP_PEERS environment variable.
        config.bootstrap_peers = match std::env::var("AXIOM_BOOTSTRAP_PEERS") {
            Ok(val) => val.split(',').map(|s| s.trim().to_string()).collect(),
            Err(_) => vec![
                "/ip4/34.10.172.20/tcp/7000".to_string(),   // Primary seed
                "/ip4/34.160.111.145/tcp/7001".to_string(),  // Secondary seed
                "/ip4/51.15.23.200/tcp/7002".to_string(),    // EU seed
                "/ip4/3.8.120.113/tcp/7003".to_string(),     // US seed
            ],
        };
        
        // Dynamic port assignment for genesis miners: 7000 + node_id
        config.listen_port = 7000 + (node_id as u16).min(3); // Ensures ports stay in 7000-7003 range
        
        // Stricter requirements for genesis phase
        config.min_peers = 3; // Need at least 3 out of 4
        config.max_peers = 50;
        
        log::info!("Genesis Miner {}: Using dedicated bootstrap configuration", node_id);
        
        config
    }
    
    /// Create config for regular validator node
    pub fn for_validator() -> Self {
        let mut config = Self::default();
        
        // Regular validators connect to known bootstrap nodes
        config.bootstrap_peers = vec![
            "/ip4/34.160.111.145/tcp/7000".to_string(),
            "/ip4/51.15.23.200/tcp/7000".to_string(),
            "/ip4/3.8.120.113/tcp/7000".to_string(),
        ];
        
        config.listen_port = 7000;
        config.min_peers = 2; // More relaxed for regular nodes
        
        log::info!("Validator Node: Using standard bootstrap configuration");
        
        config
    }
    
    /// Load from environment variable AXIOM_BOOTSTRAP_PEERS
    pub fn from_environment(listen_port: u16) -> Self {
        let mut config = Self::default();
        config.listen_port = listen_port;
        
        if let Ok(peers_str) = std::env::var("AXIOM_BOOTSTRAP_PEERS") {
            config.bootstrap_peers = peers_str
                .split(',')
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim().to_string())
                .collect();
            
            log::info!("Loaded {} bootstrap peers from AXIOM_BOOTSTRAP_PEERS", config.bootstrap_peers.len());
        }
        
        config
    }
    
    /// Load from TOML config file
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        
        log::info!("Loaded network config from {}", path);
        log::info!("Bootstrap peers: {}", config.bootstrap_peers.len());
        
        Ok(config)
    }
    
    /// Validate configuration consistency
    pub fn validate(&self) -> Result<(), String> {
        // Listen port must be valid
        if self.listen_port == 0 {
            return Err("listen_port cannot be 0".to_string());
        }
        
        // Min peers must be less than max
        if self.min_peers > self.max_peers {
            return Err(format!(
                "min_peers ({}) cannot be greater than max_peers ({})",
                self.min_peers, self.max_peers
            ));
        }
        
        // Should have at least one bootstrap peer
        if self.bootstrap_peers.is_empty() {
            log::warn!("No bootstrap peers configured - relying on mDNS discovery");
        }
        
        Ok(())
    }
    
    /// Get a summary of the configuration
    pub fn summary(&self) -> String {
        format!(
            "NetworkConfig {{\n  \
             listen_port: {},\n  \
             bootstrap_peers: {},\n  \
             max_peers: {},\n  \
             min_peers: {},\n  \
             reconnect_timeout: {}s,\n  \
             discovery_interval: {}s\n\
             }}",
            self.listen_port,
            self.bootstrap_peers.len(),
            self.max_peers,
            self.min_peers,
            self.reconnect_timeout,
            self.discovery_interval
        )
    }
}

/// Peer connection state tracker
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerState {
    pub peer_id: String,
    pub address: String,
    pub height: u64,
    pub connected: bool,
    pub last_seen: u64,
    pub trust_score: f32,
}

/// Network health metrics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkHealth {
    pub total_peers: usize,
    pub connected_peers: usize,
    pub avg_peer_height: u64,
    pub local_height: u64,
    pub synced: bool,
    pub forks_detected: u32,
}

impl Default for NetworkHealth {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkHealth {
    pub fn new() -> Self {
        Self {
            total_peers: 0,
            connected_peers: 0,
            avg_peer_height: 0,
            local_height: 0,
            synced: false,
            forks_detected: 0,
        }
    }
    
    pub fn summary(&self) -> String {
        let status = if self.synced { "✅ SYNCED" } else { "⏳ SYNCING" };
        
        format!(
            "NetworkHealth {{\n  \
             Status: {}\n  \
             Connected Peers: {}/{}\n  \
             Heights: Local={} | Peer Avg={}\n  \
             Forks Detected: {}\n\
             }}",
            status, 
            self.connected_peers, 
            self.total_peers,
            self.local_height,
            self.avg_peer_height,
            self.forks_detected
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = NetworkConfig::default();
        assert_eq!(config.listen_port, 7000);
        assert_eq!(config.min_peers, 4);
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_genesis_config() {
        let config = NetworkConfig::for_genesis_miner(1);
        assert_eq!(config.bootstrap_peers.len(), 4);
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_validator_config() {
        let config = NetworkConfig::for_validator();
        assert!(!config.bootstrap_peers.is_empty());
        assert_eq!(config.min_peers, 2);
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_validation_error() {
        let mut config = NetworkConfig::default();
        config.min_peers = 10;
        config.max_peers = 5;
        assert!(config.validate().is_err());
    }
}
