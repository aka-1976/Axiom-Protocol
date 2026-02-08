use libp2p::swarm::SwarmEvent;
use libp2p::PeerId;
use crate::network::{AxiomEvent, PeerManager};
use tracing::{info, warn, debug};

pub struct EventHandler {
    peer_manager: PeerManager,
}

impl EventHandler {
    pub fn new(max_peers: usize) -> Self {
        Self {
            peer_manager: PeerManager::new(max_peers),
        }
    }
    
    pub async fn handle_swarm_event(&mut self, event: SwarmEvent<AxiomEvent>) {
        match event {
            SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                info!("✅ Connected to peer: {} at {:?}", peer_id, endpoint);
                if self.peer_manager.add_peer(peer_id) {
                    info!("   Added to peer manager (total: {})", self.peer_manager.peer_count());
                }
            }
            
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                warn!("❌ Disconnected from peer: {} (cause: {:?})", peer_id, cause);
                if let Some(removed_peer) = self.peer_manager.remove_peer(&peer_id) {
                    info!("   Removed peer (was connected for {:?})", removed_peer.connection_duration());
                }
            }
            
            SwarmEvent::Behaviour(axiom_event) => {
                self.handle_axiom_event(axiom_event).await;
            }
            
            SwarmEvent::IncomingConnection { .. } => {
                debug!("📥 Incoming connection");
            }
            
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                if let Some(peer_id) = peer_id {
                    warn!("⚠️ Outgoing connection error to {}: {}", peer_id, error);
                }
            }
            
            _ => {}
        }
    }
    
    async fn handle_axiom_event(&mut self, event: AxiomEvent) {
        match event {
            AxiomEvent::Gossipsub(gossip_event) => {
                use libp2p::gossipsub::Event;
                match gossip_event {
                    Event::Message { propagation_source, message_id, message } => {
                        debug!("📡 Received gossipsub message from {}", propagation_source);
                        self.peer_manager.record_message_received(&propagation_source);
                        self.peer_manager.update_peer_activity(&propagation_source);
                    }
                    Event::Subscribed { peer_id, topic } => {
                        info!("📢 Peer {} subscribed to topic: {}", peer_id, topic);
                    }
                    Event::Unsubscribed { peer_id, topic } => {
                        info!("📢 Peer {} unsubscribed from topic: {}", peer_id, topic);
                    }
                    _ => {
                        debug!("📡 Other gossipsub event");
                    }
                }
            }
            
            AxiomEvent::Identify(identify_event) => {
                use libp2p::identify::Event;
                match identify_event {
                    Event::Received { peer_id, info, .. } => {
                        info!("🔍 Identified peer: {}", peer_id);
                        info!("   Protocol: {}", info.protocol_version);
                        info!("   Agent: {}", info.agent_version);
                    }
                    Event::Sent { peer_id, .. } => {
                        debug!("🔍 Sent identify info to {}", peer_id);
                    }
                    Event::Pushed { peer_id, .. } => {
                        debug!("🔍 Pushed identify update to {}", peer_id);
                    }
                    Event::Error { peer_id, error, .. } => {
                        warn!("🔍 Identify error with {}: {}", peer_id, error);
                    }
                }
            }
        }
    }
    
    pub fn peer_count(&self) -> usize {
        self.peer_manager.peer_count()
    }
    
    pub fn connected_peers(&self) -> Vec<PeerId> {
        self.peer_manager.all_peers()
    }
}
