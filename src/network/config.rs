use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::PathBuf;
use std::time::Duration;
use std::net::SocketAddr;

/// Peer discovery strategies for the AXIOM network.
///
/// Multi-vector bootstrap eliminates single-point-of-failure by allowing
/// nodes to discover peers through several independent mechanisms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryStrategy {
    /// A static list of geographically diverse bootstrap multiaddresses.
    /// At least 3 addresses are recommended for resilience.
    StaticList(Vec<String>),

    /// Kademlia DHT in Server Mode — allows new nodes to find each other
    /// without a central relay by participating in the distributed hash table.
    KademliaDHT {
        /// Protocol identifier used for Kademlia routing.
        protocol: String,
    },

    /// DNS-based seed discovery via SRV records.
    /// Resolves `_axiom-seed._tcp.<domain>` to obtain bootstrap addresses.
    DnsDiscovery {
        /// The domain to query, e.g. `"axiom-protocol.org"`.
        domain: String,
    },
}

impl Default for DiscoveryStrategy {
    fn default() -> Self {
        DiscoveryStrategy::StaticList(vec![
            "/ip4/34.10.172.20/tcp/6000".to_string(),
            "/ip4/34.160.111.145/tcp/7000".to_string(),
            "/ip4/51.15.23.200/tcp/7000".to_string(),
            "/ip4/3.8.120.113/tcp/7000".to_string(),
        ])
    }
}

impl DiscoveryStrategy {
    /// Resolve the strategy into a list of multiaddress strings that can
    /// be dialed by the libp2p swarm.
    ///
    /// * `StaticList` returns the addresses as-is.
    /// * `KademliaDHT` returns an empty list (peers are discovered via
    ///   the DHT protocol itself once connected to any single node).
    /// * `DnsDiscovery` performs a DNS SRV lookup for
    ///   `_axiom-seed._tcp.<domain>` and converts the results to
    ///   multiaddresses.
    pub fn resolve(&self) -> Vec<String> {
        match self {
            DiscoveryStrategy::StaticList(addrs) => addrs.clone(),
            DiscoveryStrategy::KademliaDHT { .. } => {
                // DHT does not produce static bootstrap addresses;
                // peers are found once *any* connection is established.
                vec![]
            }
            DiscoveryStrategy::DnsDiscovery { domain } => {
                resolve_dns_seeds(domain)
            }
        }
    }
}

/// Attempt to resolve `_axiom-seed._tcp.<domain>` via the system DNS
/// resolver and convert the results into libp2p multiaddresses.
///
/// Falls back to an empty list if resolution fails (non-fatal: the node
/// can still bootstrap via static seeds or mDNS).
fn resolve_dns_seeds(domain: &str) -> Vec<String> {
    let srv_name = format!("_axiom-seed._tcp.{}", domain);

    // Use the standard library DNS resolution (blocking) for SRV-like
    // lookup. `std::net::ToSocketAddrs` resolves A/AAAA records; true
    // SRV record support requires a dedicated DNS library which we
    // avoid for minimal dependency. Instead we resolve the SRV hostname
    // directly and use the default AXIOM TCP port.
    match std::net::ToSocketAddrs::to_socket_addrs(&(srv_name.as_str(), 7000)) {
        Ok(addrs) => addrs
            .map(|sa| format!("/ip4/{}/tcp/{}", sa.ip(), sa.port()))
            .collect(),
        Err(_) => {
            // DNS resolution failed — not fatal, other strategies can
            // still provide bootstrap peers.
            vec![]
        }
    }
}

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
    
    /// Enable mDNS for peer discovery
    pub enable_mdns: bool,

    /// Multi-vector bootstrap strategies (Static List, Kademlia DHT,
    /// DNS Discovery). Strategies are tried in order; addresses from all
    /// successful strategies are merged.
    pub discovery_strategies: Vec<DiscoveryStrategy>,
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
            heartbeat_interval: Duration::from_secs(10),
            mesh_n: 8,
            mesh_n_low: 6,
            mesh_n_high: 12,
            history_length: 6,
            history_gossip: 3,
            enable_mdns: true,
            discovery_strategies: vec![
                DiscoveryStrategy::default(), // Static list of 4 diverse IPs
                DiscoveryStrategy::KademliaDHT {
                    protocol: "/axiom/kad/1.0.0".to_string(),
                },
                DiscoveryStrategy::DnsDiscovery {
                    domain: "axiom-protocol.org".to_string(),
                },
            ],
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
    
    pub fn tcp_listen_addr(&self) -> SocketAddr {
        format!("0.0.0.0:{}", self.tcp_port).parse().unwrap()
    }
    
    pub fn udp_listen_addr(&self) -> SocketAddr {
        format!("0.0.0.0:{}", self.udp_port).parse().unwrap()
    }

    /// Resolve all configured discovery strategies into a flat list of
    /// bootstrap multiaddresses suitable for `Swarm::dial`.
    pub fn resolve_all_bootstrap_addrs(&self) -> Vec<String> {
        let mut addrs = Vec::new();
        for strategy in &self.discovery_strategies {
            addrs.extend(strategy.resolve());
        }
        addrs.sort();
        addrs.dedup();
        addrs
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_has_three_strategies() {
        let config = NetworkConfig::default();
        assert_eq!(config.discovery_strategies.len(), 3,
            "Default config must include Static, KademliaDHT, and DnsDiscovery strategies");
    }

    #[test]
    fn test_static_list_resolves() {
        let strategy = DiscoveryStrategy::StaticList(vec![
            "/ip4/1.2.3.4/tcp/6000".to_string(),
            "/ip4/5.6.7.8/tcp/7000".to_string(),
            "/ip4/9.10.11.12/tcp/7000".to_string(),
        ]);
        let addrs = strategy.resolve();
        assert_eq!(addrs.len(), 3);
        assert_eq!(addrs[0], "/ip4/1.2.3.4/tcp/6000");
    }

    #[test]
    fn test_kademlia_dht_resolves_empty() {
        let strategy = DiscoveryStrategy::KademliaDHT {
            protocol: "/axiom/kad/1.0.0".to_string(),
        };
        let addrs = strategy.resolve();
        assert!(addrs.is_empty(), "KademliaDHT produces no static addresses");
    }

    #[test]
    fn test_dns_discovery_fallback() {
        // Non-existent domain should gracefully return empty
        let strategy = DiscoveryStrategy::DnsDiscovery {
            domain: "nonexistent.invalid".to_string(),
        };
        let addrs = strategy.resolve();
        assert!(addrs.is_empty(), "DNS failure should return empty list");
    }

    #[test]
    fn test_resolve_all_deduplicates() {
        let mut config = NetworkConfig::default();
        config.discovery_strategies = vec![
            DiscoveryStrategy::StaticList(vec![
                "/ip4/1.2.3.4/tcp/6000".to_string(),
                "/ip4/5.6.7.8/tcp/7000".to_string(),
            ]),
            DiscoveryStrategy::StaticList(vec![
                "/ip4/1.2.3.4/tcp/6000".to_string(), // duplicate
                "/ip4/9.10.11.12/tcp/7000".to_string(),
            ]),
        ];
        let addrs = config.resolve_all_bootstrap_addrs();
        assert_eq!(addrs.len(), 3, "Duplicates should be removed");
    }

    #[test]
    fn test_default_static_list_has_at_least_3() {
        let strategy = DiscoveryStrategy::default();
        if let DiscoveryStrategy::StaticList(addrs) = strategy {
            assert!(addrs.len() >= 3,
                "Default static list must have at least 3 geographically diverse IPs");
        } else {
            panic!("Default strategy should be StaticList");
        }
    }
}
