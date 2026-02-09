use libp2p::gossipsub::{Behaviour as Gossipsub, Event as GossipsubEvent, IdentTopic, MessageAuthenticity, Config as GossipsubConfig};
use libp2p::identify::{Behaviour as Identify, Config as IdentifyConfig, Event as IdentifyEvent};
use libp2p::kad::{self, store::MemoryStore, Event as KademliaEvent};
use libp2p::mdns;
use libp2p::swarm::NetworkBehaviour;
use libp2p::identity::Keypair;
use libp2p::PeerId;

use crate::network::config::NetworkConfig;

/// Compute a 512-bit BLAKE3 node identity from a PeerId.
///
/// This binds the libp2p identity to the AXIOM 512-bit hash space so
/// that Kademlia keys and Gossipsub message IDs are aligned with the
/// protocol-wide BLAKE3-512 standard.
pub fn node_identity_512(peer_id: &PeerId) -> [u8; 64] {
    crate::axiom_hash_512(peer_id.to_bytes().as_slice())
}

// ---------------------------------------------------------------------------
// Hybrid Network Behaviour
// ---------------------------------------------------------------------------

/// Production-ready hybrid `NetworkBehaviour` that orchestrates Gossipsub
/// (pub/sub messaging), Kademlia DHT (structured peer & data storage),
/// Identify (peer metadata exchange), and mDNS (local peer discovery).
///
/// Discv5 (UDP) runs as a separate service (`Discv5Service`) and bridges
/// discovered peers into this behaviour via `Swarm::dial`.
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "AxiomEvent")]
pub struct AxiomHybridBehaviour {
    /// Gossipsub for pub/sub block, transaction, and pulse propagation.
    pub gossipsub: Gossipsub,
    /// Kademlia DHT for structured peer discovery and data storage.
    pub kademlia: kad::Behaviour<MemoryStore>,
    /// Identify for exchanging peer metadata (protocol version, agent).
    pub identify: Identify,
    /// mDNS for automatic local-network peer discovery.
    pub mdns: mdns::tokio::Behaviour,
}

/// Backward-compatible alias so existing code that references
/// `AxiomBehaviour` continues to compile.
pub type AxiomBehaviour = AxiomHybridBehaviour;

// ---------------------------------------------------------------------------
// Event Enum
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum AxiomEvent {
    Gossipsub(GossipsubEvent),
    Kademlia(KademliaEvent),
    Identify(IdentifyEvent),
    Mdns(mdns::Event),
}

impl From<GossipsubEvent> for AxiomEvent {
    fn from(event: GossipsubEvent) -> Self {
        AxiomEvent::Gossipsub(event)
    }
}

impl From<KademliaEvent> for AxiomEvent {
    fn from(event: KademliaEvent) -> Self {
        AxiomEvent::Kademlia(event)
    }
}

impl From<IdentifyEvent> for AxiomEvent {
    fn from(event: IdentifyEvent) -> Self {
        AxiomEvent::Identify(event)
    }
}

impl From<mdns::Event> for AxiomEvent {
    fn from(event: mdns::Event) -> Self {
        AxiomEvent::Mdns(event)
    }
}

// ---------------------------------------------------------------------------
// Implementation
// ---------------------------------------------------------------------------

impl AxiomHybridBehaviour {
    /// Create a new hybrid behaviour with default settings.
    pub fn new(keypair: &Keypair) -> Result<Self, Box<dyn std::error::Error>> {
        let peer_id = keypair.public().to_peer_id();

        let gossipsub_config = GossipsubConfig::default();
        let gossipsub = Gossipsub::new(MessageAuthenticity::Signed(keypair.clone()), gossipsub_config)?;

        let kad_store = MemoryStore::new(peer_id);
        let mut kademlia = kad::Behaviour::new(peer_id, kad_store);
        kademlia.set_mode(Some(kad::Mode::Server));

        let identify = Identify::new(IdentifyConfig::new("/axiom/1.0.0".to_string(), keypair.public()));

        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;

        Ok(Self { gossipsub, kademlia, identify, mdns })
    }

    /// Create a new hybrid behaviour using `NetworkConfig` values.
    pub fn new_with_config(keypair: &Keypair, config: &NetworkConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let peer_id = keypair.public().to_peer_id();

        let gossipsub_config = GossipsubConfig::default();
        let gossipsub = Gossipsub::new(MessageAuthenticity::Signed(keypair.clone()), gossipsub_config)?;

        let kad_store = MemoryStore::new(peer_id);
        let mut kademlia = kad::Behaviour::new(peer_id, kad_store);
        kademlia.set_mode(Some(kad::Mode::Server));

        let identify = Identify::new(IdentifyConfig::new("/axiom/1.0.0".to_string(), keypair.public()));

        // Note: mDNS cannot be fully disabled via config; when `enable_mdns`
        // is false we disable IPv6 to reduce chatter but IPv4 mDNS still runs.
        let mdns_config = if config.enable_mdns {
            mdns::Config::default()
        } else {
            let mut c = mdns::Config::default();
            c.enable_ipv6 = false;
            c
        };
        let mdns = mdns::tokio::Behaviour::new(mdns_config, peer_id)?;

        Ok(Self { gossipsub, kademlia, identify, mdns })
    }

    pub fn subscribe_to_topic(&mut self, topic_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let topic = IdentTopic::new(topic_name);
        self.gossipsub.subscribe(&topic)?;
        Ok(())
    }

    pub fn publish_message(&mut self, topic_name: &str, data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        let topic = IdentTopic::new(topic_name);
        self.gossipsub.publish(topic, data)?;
        Ok(())
    }

    pub fn connected_peers(&self) -> usize {
        self.gossipsub.all_peers().count()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_identity_512_determinism() {
        let keypair = Keypair::generate_ed25519();
        let peer_id = keypair.public().to_peer_id();

        let id_a = node_identity_512(&peer_id);
        let id_b = node_identity_512(&peer_id);

        assert_eq!(id_a, id_b, "Same PeerId must produce identical 512-bit identity");
        assert_eq!(id_a.len(), 64, "Identity must be 64 bytes (512 bits)");
    }

    #[test]
    fn test_node_identity_512_uniqueness() {
        let kp1 = Keypair::generate_ed25519();
        let kp2 = Keypair::generate_ed25519();

        let id1 = node_identity_512(&kp1.public().to_peer_id());
        let id2 = node_identity_512(&kp2.public().to_peer_id());

        assert_ne!(id1, id2, "Different PeerIds must produce different identities");
    }

    #[tokio::test]
    async fn test_hybrid_behaviour_construction() {
        let keypair = Keypair::generate_ed25519();
        let behaviour = AxiomHybridBehaviour::new(&keypair);
        assert!(behaviour.is_ok(), "Hybrid behaviour must construct without error");
    }

    #[tokio::test]
    async fn test_hybrid_behaviour_with_config() {
        let keypair = Keypair::generate_ed25519();
        let config = NetworkConfig::default();
        let behaviour = AxiomHybridBehaviour::new_with_config(&keypair, &config);
        assert!(behaviour.is_ok(), "Hybrid behaviour with config must construct without error");
    }

    #[tokio::test]
    async fn test_topic_subscription() {
        let keypair = Keypair::generate_ed25519();
        let mut behaviour = AxiomHybridBehaviour::new(&keypair).unwrap();
        let result = behaviour.subscribe_to_topic("axiom/test/topic");
        assert!(result.is_ok(), "Topic subscription must succeed");
    }

    #[tokio::test]
    async fn test_backward_compatible_alias() {
        // AxiomBehaviour is a type alias for AxiomHybridBehaviour
        let keypair = Keypair::generate_ed25519();
        let _behaviour: Result<AxiomBehaviour, _> = AxiomBehaviour::new(&keypair);
    }
}
