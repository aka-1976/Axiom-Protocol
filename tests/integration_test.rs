//! Integration test for AXIOM Protocol network stack

use axiom_protocol::network::{config::NetworkConfig, discv5_service::Discv5Service, peer_manager::PeerManager, behaviour::AxiomBehaviour, gossip_handler::GossipHandler, event_handler::EventHandler};
use axiom_protocol::metrics::MetricsCollector;

#[test]
fn test_network_stack_initialization() {
    let config = NetworkConfig::default();
    let metrics = MetricsCollector::new();
    let discv5 = Discv5Service::new(&config, &metrics);
    let peer_manager = PeerManager::new(&config, &metrics);
    let behaviour = AxiomBehaviour::new(&config, &metrics);
    let gossip_handler = GossipHandler::new(&config, &metrics);
    let event_handler = EventHandler::new();
    // Assert that all components are initialized
    assert!(discv5.is_initialized());
    assert!(peer_manager.is_initialized());
    assert!(behaviour.is_initialized());
    assert!(gossip_handler.is_initialized());
    assert!(event_handler.is_initialized());
}
