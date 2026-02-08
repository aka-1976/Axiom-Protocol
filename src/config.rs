// src/config.rs - AXIOM Protocol Production Configuration
// Complete configuration management for mainnet deployment

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::{AxiomError, Result};

/// Main configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
#[derive(Default)]
pub struct AxiomConfig {
    /// Node identification
    pub node: NodeConfig,
    /// Network settings
    pub network: NetworkConfig,
    /// Consensus parameters
    pub consensus: ConsensusConfig,
    /// Mining configuration
    pub mining: MiningConfig,
    /// Storage settings
    pub storage: StorageConfig,
    /// AI/security settings
    pub ai: AIConfig,
    /// RPC server settings
    pub rpc: RpcConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeConfig {
    /// Node name/identifier
    pub name: String,
    /// Node type (full, light, miner)
    pub node_type: NodeType,
    /// Enable metrics collection
    pub metrics_enabled: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    Full,
    Light,
    Miner,
    Archive,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkConfig {
    /// P2P listen address
    pub listen_address: String,
    /// Bootstrap peers for initial connection
    pub bootstrap_peers: Vec<String>,
    /// Maximum number of peers
    pub max_peers: usize,
    /// Maximum inbound connections
    pub max_inbound_peers: usize,
    /// Maximum outbound connections  
    pub max_outbound_peers: usize,
    /// Enable mDNS local discovery
    pub enable_mdns: bool,
    /// Enable Discv5 peer discovery
    pub enable_discv5: bool,
    /// Connection timeout (seconds)
    pub connection_timeout: u64,
    /// Gossipsub heartbeat interval (seconds)
    pub gossip_heartbeat: u64,
    /// Network ID (mainnet=1)
    pub network_id: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConsensusConfig {
    /// VDF time steps
    pub vdf_steps: u64,
    /// Initial PoW difficulty
    pub pow_difficulty: u64,
    /// Target block time (seconds)
    pub block_time_seconds: u64,
    /// Difficulty adjustment interval (blocks)
    pub difficulty_adjustment_interval: u64,
    /// Maximum block size (bytes)
    pub max_block_size: usize,
    /// Maximum transactions per block
    pub max_transactions_per_block: usize,
    /// Minimum transaction fee
    pub min_transaction_fee: u64,
    /// Block confirmation depth
    pub confirmation_depth: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MiningConfig {
    /// Enable mining
    pub enabled: bool,
    /// Number of mining threads
    pub threads: usize,
    /// Miner address (receives block rewards)
    pub miner_address: Option<String>,
    /// Mining intensity (1-100)
    pub intensity: u8,
    /// Pause mining if <N peers
    pub min_peers_to_mine: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    /// Data directory path
    pub data_dir: PathBuf,
    /// Database cache size (MB)
    pub cache_size_mb: usize,
    /// Enable database compression
    pub compression: bool,
    /// Pruning mode
    pub pruning: PruningMode,
    /// Maximum database size (GB)
    pub max_db_size_gb: u64,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PruningMode {
    Full,
    Archive,
    Light,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AIConfig {
    /// Enable Neural Guardian
    pub neural_guardian_enabled: bool,
    /// Threat detection threshold
    pub threat_threshold: f32,
    /// Model update interval (seconds)
    pub model_update_interval: u64,
    /// Enable AI oracle network
    pub oracle_enabled: bool,
    /// Minimum oracle stake
    pub min_oracle_stake: u64,
    /// Oracle consensus threshold
    pub oracle_consensus_threshold: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RpcConfig {
    /// Enable RPC server
    pub enabled: bool,
    /// RPC listen address
    pub listen_address: String,
    /// CORS allowed origins
    pub cors_allowed_origins: Vec<String>,
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// Request timeout (seconds)
    pub request_timeout: u64,
    /// Enable WebSocket
    pub websocket_enabled: bool,
    /// Rate limiting (requests per minute)
    pub rate_limit: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
    /// Enable file logging
    pub file_enabled: bool,
    /// Log file path
    pub log_file: PathBuf,
    /// Maximum log file size (MB)
    pub max_file_size_mb: u64,
    /// Number of old logs to keep
    pub max_backups: usize,
    /// Enable JSON format
    pub json_format: bool,
    /// Enable colored output
    pub colored: bool,
}

// ==================== DEFAULT CONFIGURATIONS ====================


impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            name: format!("axiom-node-{}", rand::random::<u16>()),
            node_type: NodeType::Full,
            metrics_enabled: true,
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_address: "/ip4/0.0.0.0/tcp/8545".to_string(),
            bootstrap_peers: vec![],
            max_peers: 50,
            max_inbound_peers: 30,
            max_outbound_peers: 20,
            enable_mdns: true,
            enable_discv5: true,
            connection_timeout: 30,
            gossip_heartbeat: 1,
            network_id: 1,
        }
    }
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            vdf_steps: 3_600_000,
            pow_difficulty: 1000,
            block_time_seconds: 1800,
            difficulty_adjustment_interval: 2016,
            max_block_size: 1_000_000,
            max_transactions_per_block: 10_000,
            min_transaction_fee: 100_000_000,
            confirmation_depth: 6,
        }
    }
}

impl Default for MiningConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            threads: num_cpus::get(),
            miner_address: None,
            intensity: 80,
            min_peers_to_mine: 3,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./axiom-data"),
            cache_size_mb: 1024,
            compression: true,
            pruning: PruningMode::Full,
            max_db_size_gb: 0,
        }
    }
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            neural_guardian_enabled: true,
            threat_threshold: 0.7,
            model_update_interval: 86400,
            oracle_enabled: false,
            min_oracle_stake: 50_000_000_000,
            oracle_consensus_threshold: 3,
        }
    }
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            listen_address: "127.0.0.1:8546".to_string(),
            cors_allowed_origins: vec!["*".to_string()],
            max_connections: 100,
            request_timeout: 30,
            websocket_enabled: true,
            rate_limit: 60,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file_enabled: true,
            log_file: PathBuf::from("axiom-node.log"),
            max_file_size_mb: 100,
            max_backups: 10,
            json_format: false,
            colored: true,
        }
    }
}

impl AxiomConfig {
    /// Load configuration from file
    pub fn load_from_file(path: &str) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| AxiomError::ConfigNotFound(e.to_string()))?;
        
        toml::from_str(&contents)
            .map_err(|e| AxiomError::ConfigParseError(e.to_string()))
    }
    
    /// Load configuration with defaults
    pub fn load() -> Result<Self> {
        for path in &["axiom.toml", "./config/axiom.toml", "/etc/axiom/axiom.toml"] {
            if std::path::Path::new(path).exists() {
                return Self::load_from_file(path);
            }
        }
        Ok(Self::default())
    }
    
    /// Save configuration to file
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let contents = toml::to_string_pretty(self)
            .map_err(|e| AxiomError::SerializationError(e.to_string()))?;
        
        std::fs::write(path, contents)?;
        Ok(())
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.network.max_peers == 0 {
            return Err(AxiomError::InvalidConfig("max_peers must be > 0".to_string()));
        }
        
        if self.consensus.vdf_steps == 0 {
            return Err(AxiomError::InvalidConfig("vdf_steps must be > 0".to_string()));
        }
        
        if self.mining.enabled && self.mining.miner_address.is_none() {
            return Err(AxiomError::InvalidConfig(
                "miner_address required when mining enabled".to_string()
            ));
        }
        
        if !self.storage.data_dir.exists() {
            std::fs::create_dir_all(&self.storage.data_dir)?;
        }
        
        Ok(())
    }
    
    /// Create validator configuration (mainnet with archive mode)
    pub fn validator() -> Self {
        let mut config = Self::default();
        config.network.network_id = 1;
        config.node.node_type = NodeType::Archive;
        config.storage.pruning = PruningMode::Archive;
        config.consensus.vdf_steps = 3_600_000;
        config.consensus.block_time_seconds = 1800;
        config.consensus.pow_difficulty = 1000;
        config.storage.data_dir = PathBuf::from("./axiom-validator-data");
        config
    }
    
    /// Create light client configuration (mainnet with pruning)
    pub fn light_client() -> Self {
        let mut config = Self::default();
        config.network.network_id = 1;
        config.node.node_type = NodeType::Light;
        config.network.max_peers = 20;
        config.consensus.vdf_steps = 3_600_000;
        config.consensus.block_time_seconds = 1800;
        config.consensus.pow_difficulty = 1000;
        config.mining.enabled = false;
        config.storage.data_dir = PathBuf::from("./axiom-light-data");
        config.storage.pruning = PruningMode::Light;
        config.logging.level = "info".to_string();
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = AxiomConfig::default();
        assert_eq!(config.network.network_id, 1);
        assert!(config.consensus.vdf_steps > 0);
    }
    
    #[test]
    fn test_config_validation() {
        let config = AxiomConfig::default();
        assert!(config.validate().is_ok());
    }
}
