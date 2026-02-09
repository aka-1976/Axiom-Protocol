//! Integration test for AXIOM Protocol network stack

use axiom_core::network::{
    config::NetworkConfig,
    peer_manager::PeerManager,
    behaviour::AxiomHybridBehaviour,
    event_handler::EventHandler,
};
use libp2p::identity::Keypair;

#[test]
fn test_network_stack_initialization() {
    let config = NetworkConfig::default();
    config.validate().expect("Default config must be valid");

    let peer_manager = PeerManager::new(config.max_peers);
    assert_eq!(peer_manager.peer_count(), 0);

    let event_handler = EventHandler::new(config.max_peers);
    assert_eq!(event_handler.peer_count(), 0);
}

#[tokio::test]
async fn test_hybrid_behaviour_initialization() {
    let keypair = Keypair::generate_ed25519();
    let config = NetworkConfig::default();
    let behaviour = AxiomHybridBehaviour::new_with_config(&keypair, &config);
    assert!(behaviour.is_ok(), "Hybrid behaviour must initialize from config");

    let mut b = behaviour.unwrap();
    assert_eq!(b.connected_peers(), 0);
    assert!(b.subscribe_to_topic("axiom/test").is_ok());
}
