use libp2p::PeerId;
use std::sync::Arc;
use tracing::{debug, info};

use crate::network::{peer_manager::PeerManager, AxiomBehaviourEvent};

/// Network event handler (processes all libp2p events)
pub struct EventHandler {
    peer_manager: Arc<PeerManager>,
}

impl EventHandler {
    pub fn new(peer_manager: Arc<PeerManager>) -> Self {
        Self { peer_manager }
    }
    
    /// Handle protocol-specific behaviour events
    pub async fn handle_behaviour_event(
        &self,
        _event: AxiomBehaviourEvent,
    ) -> Result<(), EventHandlerError> {
        match _event {
            AxiomBehaviourEvent::Gossipsub(_) => {
                debug!("Gossipsub event received");
            }
            AxiomBehaviourEvent::Mdns(_) => {
                debug!("mDNS event received");
            }
            AxiomBehaviourEvent::Kademlia(_) => {
                debug!("Kademlia event received");
            }
            AxiomBehaviourEvent::Identify(_) => {
                debug!("Identify event received");
            }
            AxiomBehaviourEvent::Ping(_) => {
                debug!("Ping event received");
            }
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EventHandlerError {
    #[error("Message processing error: {0}")]
    MessageProcessing(String),
    
    #[error("Peer manager error: {0}")]
    PeerManager(String),
}
