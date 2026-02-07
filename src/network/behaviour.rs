use libp2p::{
    core::ConnectedPoint,
    gossipsub::{self, IdentTopic},
    identify,
    identity::Keypair,
    kad::{self, store::MemoryStore, Kademlia, KademliaConfig},
    mdns,
    ping,
    swarm::{NetworkBehaviour, SwarmEvent},
    Multiaddr, PeerId,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tracing::{debug, info, trace, warn};

use super::config::NetworkConfig;

/// AXIOM protocol topics for Gossipsub
#[derive(Debug, Clone)]
pub struct AxiomTopics {
    /// Block propagation topic
    pub blocks: IdentTopic,
    
    /// Transaction propagation topic
    pub transactions: IdentTopic,
    
    /// Chain state sync topic
    pub sync: IdentTopic,
    
    /// Network heartbeat topic
    pub heartbeat: IdentTopic,
}

impl AxiomTopics {
    pub fn new(chain_id: &str) -> Self {
        Self {
            blocks: IdentTopic::new(format!("axiom/{}/blocks/1", chain_id)),
            transactions: IdentTopic::new(format!("axiom/{}/transactions/1", chain_id)),
            sync: IdentTopic::new(format!("axiom/{}/sync/1", chain_id)),
            heartbeat: IdentTopic::new(format!("axiom/{}/heartbeat/1", chain_id)),
        }
    }
    
    pub fn all(&self) -> Vec<&IdentTopic> {
        vec![&self.blocks, &self.transactions, &self.sync, &self.heartbeat]
    }
}

/// Composite network behavior combining all protocols
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "AxiomBehaviourEvent")]
pub struct AxiomBehaviour {
    /// Gossipsub for message propagation (blocks, transactions)
    pub gossipsub: gossipsub::Behaviour,
    
    /// mDNS for local network peer discovery
    pub mdns: mdns::tokio::Behaviour,
    
    /// Kademlia DHT for peer routing and data storage
    pub kademlia: Kademlia<MemoryStore>,
    
    /// Identify protocol for peer information exchange
    pub identify: identify::Behaviour,
    
    /// Ping for connection keepalive and latency measurement
    pub ping: ping::Behaviour,
}

/// Events from network behaviour
#[derive(Debug)]
pub enum AxiomBehaviourEvent {
    Gossipsub(gossipsub::Event),
    Mdns(mdns::Event),
    Kademlia(kad::Event),
    Identify(identify::Event),
    Ping(ping::Event),
}

impl From<gossipsub::Event> for AxiomBehaviourEvent {
    fn from(event: gossipsub::Event) -> Self {
        AxiomBehaviourEvent::Gossipsub(event)
    }
}

impl From<mdns::Event> for AxiomBehaviourEvent {
    fn from(event: mdns::Event) -> Self {
        AxiomBehaviourEvent::Mdns(event)
    }
}

impl From<kad::Event> for AxiomBehaviourEvent {
    fn from(event: kad::Event) -> Self {
        AxiomBehaviourEvent::Kademlia(event)
    }
}

impl From<identify::Event> for AxiomBehaviourEvent {
    fn from(event: identify::Event) -> Self {
        AxiomBehaviourEvent::Identify(event)
    }
}

impl From<ping::Event> for AxiomBehaviourEvent {
    fn from(event: ping::Event) -> Self {
        AxiomBehaviourEvent::Ping(event)
    }
}

impl AxiomBehaviour {
    /// Create new network behaviour with production configuration
    pub fn new(
        keypair: &Keypair,
        peer_id: PeerId,
        config: &NetworkConfig,
    ) -> Result<Self, BehaviourError> {
        //Configure Gossipsub with production settings
        let gossipsub = Self::build_gossipsub(keypair, config)?;
        
        // Configure mDNS for local discovery
        let mdns = mdns::tokio::Behaviour::new(
            mdns::Config::default(),
            peer_id,
        ).map_err(|e| BehaviourError::Mdns(e))?;
        
        // Configure Kademlia DHT
        let kademlia = Self::build_kademlia(peer_id, config);
        
        // Configure Identify protocol
        let identify = Self::build_identify(keypair);
        
        // Configure Ping
        let ping = Self::build_ping();
        
        Ok(Self {
            gossipsub,
            mdns,
            kademlia,
            identify,
            ping,
        })
    }
    
    /// Build production Gossipsub configuration
    fn build_gossipsub(
        keypair: &Keypair,
        config: &NetworkConfig,
    ) -> Result<gossipsub::Behaviour, BehaviourError> {
        // Message validation strategy
        let message_authenticity = gossipsub::MessageAuthenticity::Signed(keypair.clone());
        
        // Production Gossipsub config (Ethereum-inspired)
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            // Mesh parameters (how many peers to gossip to)
            .mesh_n(config.gossip_config.mesh_n)
            .mesh_n_low(config.gossip_config.mesh_n_low)
            .mesh_n_high(config.gossip_config.mesh_n_high)
            
            // Gossip parameters
            .gossip_lazy(6)
            .gossip_factor(0.25)
            
            // Heartbeat
            .heartbeat_interval(config.gossip_config.heartbeat_interval)
            
            // Message history (for deduplication)
            .history_length(config.gossip_config.history_length)
            .history_gossip(config.gossip_config.history_gossip)
            
            // Message validation
            .validation_mode(gossipsub::ValidationMode::Strict)
            .duplicate_cache_time(Duration::from_secs(60))
            
            // Message size limits (prevent DoS)
            .max_transmit_size(2 * 1024 * 1024) // 2MB max message
            
            // Flood publishing for critical messages
            .flood_publish(false)
            
            // Protocol ID
            .protocol_id_prefix("/axiom/gossipsub")
            
            .build()
            .map_err(|e| BehaviourError::GossipsubConfig(e.to_string()))?;
        
        let mut gossipsub = gossipsub::Behaviour::new(
            message_authenticity,
            gossipsub_config,
        ).map_err(|e| BehaviourError::GossipsubInit(e.to_string()))?;
        
        // Subscribe to AXIOM topics
        let topics = AxiomTopics::new("mainnet");
        for topic in topics.all() {
            gossipsub.subscribe(topic)?;
            info!("📡 Subscribed to topic: {}", topic);
        }
        
        Ok(gossipsub)
    }
    
    /// Build Kademlia DHT configuration
    fn build_kademlia(peer_id: PeerId, _config: &NetworkConfig) -> Kademlia<MemoryStore> {
        let mut kad_config = KademliaConfig::default();
        
        // Query configuration
        kad_config.set_query_timeout(Duration::from_secs(60));
        kad_config.set_replication_factor(
            std::num::NonZeroUsize::new(20).unwrap()
        );
        
        // Protocol name
        kad_config.set_protocol_names(vec![
            std::borrow::Cow::Borrowed(b"/axiom/kad/1.0.0")
        ]);
        
        // Provider record TTL
        kad_config.set_provider_record_ttl(Some(Duration::from_secs(3600 * 24))); // 24 hours
        
        // Record TTL
        kad_config.set_record_ttl(Some(Duration::from_secs(3600 * 24 * 7))); // 7 days
        
        let store = MemoryStore::new(peer_id);
        Kademlia::with_config(peer_id, store, kad_config)
    }
    
    /// Build Identify protocol
    fn build_identify(keypair: &Keypair) -> identify::Behaviour {
        let public_key = keypair.public();
        
        identify::Behaviour::new(identify::Config::new(
            "/axiom/3.0.0".to_string(),
            public_key,
        )
        .with_agent_version("axiom-rust/3.0.0".to_string())
        .with_interval(Duration::from_secs(300)))
    }
    
    /// Build Ping protocol
    fn build_ping() -> ping::Behaviour {
        ping::Behaviour::new(
            ping::Config::new()
                .with_interval(Duration::from_secs(30))
                .with_timeout(Duration::from_secs(10))
                .with_max_failures(std::num::NonZeroU32::new(3).unwrap()),
        )
    }
    
    /// Publish message to Gossipsub topic
    pub fn publish_message(
        &mut self,
        topic: &IdentTopic,
        data: Vec<u8>,
    ) -> Result<gossipsub::MessageId, BehaviourError> {
        self.gossipsub
            .publish(topic.clone(), data)
            .map_err(|e| BehaviourError::PublishFailed(e.to_string()))
    }
    
    /// Add peer to Kademlia routing table
    pub fn add_kad_peer(&mut self, peer_id: PeerId, multiaddr: Multiaddr) {
        self.kademlia.add_address(&peer_id, multiaddr.clone());
        debug!("Added peer to Kademlia: {} at {}", peer_id, multiaddr);
    }
    
    /// Bootstrap Kademlia (discover more peers via DHT)
    pub fn bootstrap_kad(&mut self) -> Result<(), BehaviourError> {
        self.kademlia
            .bootstrap()
            .map_err(|e| BehaviourError::KademliaBootstrap(e.to_string()))?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BehaviourError {
    #[error("Gossipsub configuration error: {0}")]
    GossipsubConfig(String),
    
    #[error("Gossipsub initialization error: {0}")]
    GossipsubInit(String),
    
    #[error("Gossipsub subscription error: {0}")]
    GossipsubSubscription(#[from] gossipsub::SubscriptionError),
    
    #[error("mDNS error: {0}")]
    Mdns(#[from] std::io::Error),
    
    #[error("Publish failed: {0}")]
    PublishFailed(String),
    
    #[error("Kademlia bootstrap error: {0}")]
    KademliaBootstrap(String),
}
