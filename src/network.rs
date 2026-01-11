use libp2p::{gossipsub, mdns, kad, identify, swarm::{NetworkBehaviour, Swarm}};
use std::error::Error;
use libp2p::identity;

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "TimechainBehaviourEvent")]
pub struct TimechainBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub identify: identify::Behaviour,
}

#[derive(Debug)]
pub enum TimechainBehaviourEvent {
    Gossipsub(gossipsub::Event),
    Mdns(mdns::Event),
    Kademlia(kad::Event),
    Identify(identify::Event),
}

// Convert sub-events into our main event enum
impl From<gossipsub::Event> for TimechainBehaviourEvent {
    fn from(event: gossipsub::Event) -> Self { Self::Gossipsub(event) }
}
impl From<mdns::Event> for TimechainBehaviourEvent {
    fn from(event: mdns::Event) -> Self { Self::Mdns(event) }
}
impl From<kad::Event> for TimechainBehaviourEvent {
    fn from(event: kad::Event) -> Self { Self::Kademlia(event) }
}
impl From<identify::Event> for TimechainBehaviourEvent {
    fn from(event: identify::Event) -> Self { Self::Identify(event) }
}

// Ensure this is PUB so main.rs can call it
pub async fn init_network() -> Result<Swarm<TimechainBehaviour>, Box<dyn Error + Send + Sync>> {
    let local_key = identity::Keypair::generate_ed25519();
    
    let swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_tcp(libp2p::tcp::Config::default(), libp2p::noise::Config::new, libp2p::yamux::Config::default)?
        .with_behaviour(|key| {
            Ok(TimechainBehaviour {
                gossipsub: gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub::Config::default(),
                )?,
                mdns: mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?,
                kademlia: kad::Behaviour::new(key.public().to_peer_id(), kad::store::MemoryStore::new(key.public().to_peer_id())),
                identify: identify::Behaviour::new(identify::Config::new("qubit/1.0.0".into(), key.public())),
            })
        })?
        .build();

    Ok(swarm)
}
