use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::PathBuf;
use std::time::Duration;

/// Production network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// TCP port for libp2p (Gossipsub, Kad, etc.)
    pub tcp_port: u16,
    
    /// UDP port for Discv5 discovery
    pub udp_port: u16,
    
    /// External IP address (if known, otherwise auto-detected)
    pub external_ip: Option<IpAddr>,
    
    /// Bootstrap ENRs (Ethereum Node Records)
    pub boot_enrs: Vec<String>,
    
    /// Target number of peers to maintain
    pub target_peers: usize,
    
    /// Maximum number of peers
    pub max_peers: usize,
    
    /// Minimum peers before considering synced
    pub min_peers_for_sync: usize,
    
    /// Peer connection timeout
    pub connection_timeout: Duration,
    
    /// Discv5 lookup interval
    pub discovery_interval: Duration,
    
    /// Gossipsub configuration
    pub gossip_config: GossipConfig,
    
    /// Data directory for ENR key storage
    pub data_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipConfig {
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
    
    /// Number of peers to gossip to
    pub mesh_n: usize,
    
    /// Low watermark for mesh peers
    pub mesh_n_low: usize,
    
    /// High watermark for mesh peers
    pub mesh_n_high: usize,
    
    /// History length for message deduplication
    pub history_length: usize,
    
    /// History gossip length
    pub history_gossip: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            tcp_port: 7000,
            udp_port: 9000,
            external_ip: None,
            boot_enrs: vec![],
            target_peers: 50,
            max_peers: 100,
            min_peers_for_sync: 2,
            connection_timeout: Duration::from_secs(30),
            discovery_interval: Duration::from_secs(30),
            gossip_config: GossipConfig::default(),
            data_dir: PathBuf::from(".axiom"),
        }
    }
}

impl Default for GossipConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval: Duration::from_secs(1),
            mesh_n: 8,
            mesh_n_low: 6,
            mesh_n_high: 12,
            history_length: 6,
            history_gossip: 3,
        }
    }
}

impl NetworkConfig {
    /// Create mainnet configuration
    pub fn mainnet() -> Self {
        Self {
            tcp_port: 443,
            udp_port: 9000,
            boot_enrs: vec![
                // Production bootstrap ENRs will be added here
                // Format: "enr:-IS4QHCYrYZbAKWCBRlAy5zzaDZXJBGkcnh4..."
            ],
            target_peers: 50,
            max_peers: 100,
            min_peers_for_sync: 3,
            ..Default::default()
        }
    }
    
    /// Create testnet configuration
    pub fn testnet() -> Self {
        Self {
            tcp_port: 17000,
            udp_port: 19000,
            boot_enrs: vec![],
            target_peers: 20,
            max_peers: 50,
            min_peers_for_sync: 1,
            ..Default::default()
        }
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.tcp_port == 0 {
            return Err(ConfigError::InvalidPort("TCP port cannot be 0".into()));
        }
        
        if self.udp_port == 0 {
            return Err(ConfigError::InvalidPort("UDP port cannot be 0".into()));
        }
        
        if self.tcp_port == self.udp_port {
            return Err(ConfigError::InvalidPort("TCP and UDP ports must differ".into()));
        }
        
        if self.max_peers < self.target_peers {
            return Err(ConfigError::InvalidPeerConfig(
                "max_peers must be >= target_peers".into()
            ));
        }
        
        if self.target_peers < self.min_peers_for_sync {
            return Err(ConfigError::InvalidPeerConfig(
                "target_peers must be >= min_peers_for_sync".into()
            ));
        }
        
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid port configuration: {0}")]
    InvalidPort(String),
    
    #[error("Invalid peer configuration: {0}")]
    InvalidPeerConfig(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
